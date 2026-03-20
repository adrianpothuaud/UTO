//! Axum-based HTTP + WebSocket server for the UTO UI mode.
//!
//! ## Endpoints
//!
//! | Method | Path         | Description                                               |
//! |--------|--------------|-----------------------------------------------------------|
//! | GET    | `/`          | Serve embedded SPA (`index.html`)                         |
//! | GET    | `/api/status`| JSON `{ project, status }` — server health / project info |
//! | GET    | `/api/report`| JSON report artifact loaded at startup, or `null`         |
//! | GET    | `/api/tests` | JSON discovered test list from project `tests/` sources    |
//! | GET    | `/ws`        | WebSocket upgrade — live event stream / run relay         |

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use tokio::sync::{broadcast, Mutex, RwLock};
use tower_http::services::ServeDir;

/// Embedded SPA — compiled into the binary at build time.
const INDEX_HTML: &str = include_str!("assets/index.html");

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Configuration options for the UI server.
#[derive(Debug, Clone)]
pub struct UiOptions {
    /// Path to the UTO project directory (used for project name display and run integration).
    pub project: PathBuf,
    /// Local port for the HTTP + WebSocket server. Default: `4000`.
    pub port: u16,
    /// Automatically open the browser after the server starts. Default: `false`.
    pub open: bool,
    /// Enable watch mode (auto re-run on file change). Default: `false`.
    pub watch: bool,
    /// Path to a saved `uto-suite/v1` or `uto-report/v1` JSON artifact to replay. Default: `None`.
    pub report: Option<PathBuf>,
    /// Enable UTO Studio mode (Phase 6 — visual test authoring and recording). Default: `false`.
    pub studio: bool,
}

impl Default for UiOptions {
    fn default() -> Self {
        Self {
            project: PathBuf::from("."),
            port: 4000,
            open: false,
            watch: false,
            report: None,
            studio: false,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub(crate) struct AppState {
    /// Display name derived from project directory or `uto.json`.
    pub(crate) project_name: String,
    /// Project directory — used when spawning a live run subprocess.
    pub(crate) project: PathBuf,
    /// Shared, mutable report artifact.  Updated after each live run.
    pub(crate) report: Arc<RwLock<Option<serde_json::Value>>>,
    /// Broadcast channel for streaming live events to all WebSocket clients.
    pub(crate) tx: broadcast::Sender<String>,
    /// Whether a run subprocess is currently active.
    pub(crate) run_active: Arc<AtomicBool>,
    /// Kill handle for the active run subprocess (if any).
    pub(crate) kill_handle: Arc<Mutex<Option<crate::runner::KillHandle>>>,
    /// Whether Studio mode (Phase 6) is enabled.
    pub(crate) studio_mode: bool,
    /// Studio state — shared mutable recording context.
    pub(crate) studio: Arc<RwLock<crate::studio::StudioState>>,
}

// ---------------------------------------------------------------------------
// Router factory (also used by tests)
// ---------------------------------------------------------------------------

fn create_router(state: AppState) -> Router {
    // Serve static files from the project's .uto/reports directory
    let reports_dir = state.project.join(".uto/reports");
    let serve_reports = ServeDir::new(reports_dir);

    Router::new()
        .route("/", get(serve_index))
        .route("/api/status", get(api_status))
        .route("/api/report", get(api_report))
        .route("/api/tests", get(api_tests))
        .route("/api/studio/status", get(crate::studio::api_studio_status))
        .route(
            "/api/studio/start",
            axum::routing::post(crate::studio::api_studio_start),
        )
        .route(
            "/api/studio/stop",
            axum::routing::post(crate::studio::api_studio_stop),
        )
        .route(
            "/api/studio/step",
            axum::routing::post(crate::studio::api_studio_add_step),
        )
        .route("/ws", get(ws_handler))
        .nest_service("/.uto/reports", serve_reports)
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Start the UTO UI server asynchronously.
///
/// Binds an HTTP + WebSocket server on `localhost:<port>`, serves the embedded
/// SPA, optionally replays a saved report artifact over WebSocket on client
/// connect, and optionally watches the project `tests/` directory for changes.
/// Blocks until `SIGINT` / `Ctrl+C` is received.
pub async fn start_server(opts: UiOptions) -> Result<(), String> {
    // Load the initial report artifact (from `--report`), if provided.
    let initial_report = if let Some(ref path) = opts.report {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read report '{}': {e}", path.display()))?;
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid JSON in '{}': {e}", path.display()))?;
        Some(value)
    } else {
        None
    };

    let project_name = derive_project_name(&opts.project);

    let (tx, _rx) = broadcast::channel(256);
    let studio_mode = opts.studio;
    let state = AppState {
        project_name,
        project: opts.project.clone(),
        report: Arc::new(RwLock::new(initial_report)),
        tx,
        run_active: Arc::new(AtomicBool::new(false)),
        kill_handle: Arc::new(Mutex::new(None)),
        studio_mode,
        studio: Arc::new(RwLock::new(crate::studio::StudioState::default())),
    };

    if studio_mode {
        log::info!("UTO Studio mode enabled (Phase 6)");
        println!("  Studio mode  →  enabled");
    }

    // Start the filesystem watcher when `--watch` is enabled.
    if opts.watch {
        // Watch the tests/ sub-directory; fall back to the project root.
        let watch_path = {
            let tests_dir = opts.project.join("tests");
            if tests_dir.exists() {
                tests_dir
            } else {
                opts.project.clone()
            }
        };
        let watch_state = state.clone();
        let rt = tokio::runtime::Handle::current();
        match crate::watcher::start_watcher(watch_path.clone(), move || {
            let s = watch_state.clone();
            rt.spawn(async move { handle_trigger_run(s, None).await });
        }) {
            Ok(()) => {
                log::info!("Watch mode active — watching '{}'", watch_path.display());
                println!("  Watch mode  →  watching '{}'", watch_path.display());
            }
            Err(e) => {
                log::warn!("Watch mode unavailable: {e}");
                println!("  Watch mode unavailable: {e}");
            }
        }
    }

    let app = create_router(state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], opts.port));
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("Failed to bind to port {}: {e}", opts.port))?;

    let url = format!("http://localhost:{}", opts.port);
    log::info!("UTO UI server listening on {url}");
    println!("  UTO UI  →  {url}");
    println!("  Press Ctrl+C to stop");

    if opts.open {
        open_browser(&url);
    }

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| format!("Server error: {e}"))?;

    Ok(())
}

/// Blocking wrapper around [`start_server`].
///
/// Creates a `tokio` multi-thread runtime and blocks until the server shuts
/// down. Designed for use from a synchronous `main` (e.g. `uto-cli`).
pub fn start_server_sync(opts: UiOptions) -> Result<(), String> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create async runtime: {e}"))?
        .block_on(start_server(opts))
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

async fn serve_index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn api_status(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "project": state.project_name,
        "status": if state.run_active.load(Ordering::Relaxed) { "running" } else { "ready" },
        "studio": state.studio_mode,
    }))
}

async fn api_report(State(state): State<AppState>) -> impl IntoResponse {
    let guard = state.report.read().await;
    match &*guard {
        Some(r) => Json(r.clone()).into_response(),
        None => Json(serde_json::Value::Null).into_response(),
    }
}

async fn api_tests(State(state): State<AppState>) -> impl IntoResponse {
    match discover_project_tests(&state.project) {
        Ok(tests) => Json(serde_json::json!({ "tests": tests })).into_response(),
        Err(e) => Json(serde_json::json!({ "tests": [], "error": e })).into_response(),
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    // Push the pre-loaded report (if any) to the new client immediately.
    {
        let guard = state.report.read().await;
        if let Some(report) = &*guard {
            let msg = serde_json::json!({ "type": "report", "payload": report });
            if socket
                .send(Message::Text(msg.to_string().into()))
                .await
                .is_err()
            {
                return;
            }
        }
    }

    // Relay broadcast events and handle client control messages.
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(msg) => {
                        if socket.send(Message::Text(msg.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        handle_ws_message(text.as_str(), state.clone()).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }
}

/// Dispatch an incoming WebSocket text message to the appropriate handler.
async fn handle_ws_message(text: &str, state: AppState) {
    if let Ok(msg) = serde_json::from_str::<serde_json::Value>(text) {
        match msg.get("type").and_then(|t| t.as_str()) {
            Some("trigger_run") => {
                let selection = parse_selection(&msg);
                handle_trigger_run(state, selection).await
            }
            Some("stop_run") => handle_stop_run(state).await,
            _ => {}
        }
    }
}

/// Start a live run subprocess if none is currently active.
async fn handle_trigger_run(state: AppState, selection: Option<crate::runner::RunSelection>) {
    // Atomically claim the run slot; notify clients if already occupied.
    if state
        .run_active
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        let _ = state.tx.send(
            serde_json::json!({
                "type": "run_ignored",
                "payload": { "reason": "run_already_active" }
            })
            .to_string(),
        );
        return;
    }

    let handle = crate::runner::start_run(
        state.project.clone(),
        state.tx.clone(),
        state.report.clone(),
        state.run_active.clone(),
        selection,
    )
    .await;

    *state.kill_handle.lock().await = Some(handle);
}

/// Stop the currently active run subprocess (if any).
async fn handle_stop_run(state: AppState) {
    if let Some(h) = state.kill_handle.lock().await.take() {
        h.kill();
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Derive a human-readable project name from the project directory.
///
/// Reads `uto.json` if present; otherwise falls back to the directory name.
fn derive_project_name(project: &std::path::Path) -> String {
    let config_path = project.join("uto.json");
    if config_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&config_path) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(name) = v.get("project_name").and_then(|n| n.as_str()) {
                    if !name.trim().is_empty() {
                        return name.to_string();
                    }
                }
            }
        }
    }
    project
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("uto-project")
        .to_string()
}

fn parse_selection(msg: &serde_json::Value) -> Option<crate::runner::RunSelection> {
    let payload = msg.get("payload")?;
    let test_bin = payload.get("test_bin")?.as_str()?.trim();
    let test_name = payload.get("test_name")?.as_str()?.trim();
    if test_bin.is_empty() || test_name.is_empty() {
        return None;
    }
    Some(crate::runner::RunSelection {
        test_bin: test_bin.to_string(),
        test_name: test_name.to_string(),
    })
}

fn parse_test_target_from_attr(line: &str) -> Option<String> {
    if line.contains("target") && line.contains("\"web\"") {
        return Some("web".to_string());
    }
    if line.contains("target") && line.contains("\"mobile\"") {
        return Some("mobile".to_string());
    }
    None
}

fn infer_target_from_name(name: &str) -> Option<String> {
    if name.starts_with("web_") {
        return Some("web".to_string());
    }
    if name.starts_with("mobile_") {
        return Some("mobile".to_string());
    }
    None
}

fn extract_fn_name(line: &str) -> Option<String> {
    let prefixes = ["pub async fn ", "async fn ", "pub fn ", "fn "];
    let prefix = prefixes.into_iter().find(|p| line.starts_with(p))?;
    let rest = &line[prefix.len()..];
    let mut name = String::new();
    for ch in rest.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            name.push(ch);
        } else {
            break;
        }
    }
    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}

fn parse_tests_from_source(test_bin: &str, source: &str) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    let mut pending_test_attr = false;
    let mut pending_target: Option<String> = None;

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#[") {
            if trimmed.contains("tokio::test") || trimmed.starts_with("#[test") {
                pending_test_attr = true;
            }
            if trimmed.contains("uto_test") {
                pending_target = parse_test_target_from_attr(trimmed);
            }
            continue;
        }

        if let Some(test_name) = extract_fn_name(trimmed) {
            if pending_test_attr || pending_target.is_some() {
                let target = pending_target
                    .clone()
                    .or_else(|| infer_target_from_name(&test_name));
                out.push(serde_json::json!({
                    "test_bin": test_bin,
                    "test_name": test_name,
                    "target": target,
                }));
            }
            pending_test_attr = false;
            pending_target = None;
            continue;
        }

        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            pending_test_attr = false;
        }
    }

    out
}

fn discover_project_tests(project: &std::path::Path) -> Result<Vec<serde_json::Value>, String> {
    let tests_root = project.join("tests");
    if !tests_root.exists() {
        return Ok(Vec::new());
    }

    let mut tests = Vec::new();
    for entry in std::fs::read_dir(&tests_root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }

        let test_bin = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid test filename: {}", path.display()))?
            .to_string();
        let source = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read test source '{}': {e}", path.display()))?;
        tests.extend(parse_tests_from_source(&test_bin, &source));
    }

    tests.sort_by(|a, b| {
        let a_bin = a
            .get("test_bin")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let b_bin = b
            .get("test_bin")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let a_name = a
            .get("test_name")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let b_name = b
            .get("test_name")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        a_bin.cmp(b_bin).then_with(|| a_name.cmp(b_name))
    });

    Ok(tests)
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C signal handler. Check OS signal permissions or file a bug report at https://github.com/adrianpothuaud/UTO");
    println!("\nStopping UTO UI server…");
}

fn open_browser(url: &str) {
    let (cmd, args): (&str, &[&str]) = if cfg!(target_os = "macos") {
        ("open", &[url])
    } else if cfg!(target_os = "windows") {
        ("cmd", &["/C", "start", url])
    } else {
        ("xdg-open", &[url])
    };

    let result = std::process::Command::new(cmd).args(args).spawn().ok();

    if result.is_none() {
        log::warn!(
            "Could not open browser automatically (tried `{cmd}`). Navigate to {url} manually."
        );
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn test_state() -> AppState {
        let (tx, _) = broadcast::channel(16);
        AppState {
            project_name: "test-project".to_string(),
            project: PathBuf::from("."),
            report: Arc::new(RwLock::new(None)),
            tx,
            run_active: Arc::new(AtomicBool::new(false)),
            kill_handle: Arc::new(Mutex::new(None)),
            studio_mode: false,
            studio: Arc::new(RwLock::new(crate::studio::StudioState::default())),
        }
    }

    fn test_state_with_report(report: serde_json::Value) -> AppState {
        let (tx, _) = broadcast::channel(16);
        AppState {
            project_name: "report-project".to_string(),
            project: PathBuf::from("."),
            report: Arc::new(RwLock::new(Some(report))),
            tx,
            run_active: Arc::new(AtomicBool::new(false)),
            kill_handle: Arc::new(Mutex::new(None)),
            studio_mode: false,
            studio: Arc::new(RwLock::new(crate::studio::StudioState::default())),
        }
    }

    #[tokio::test]
    async fn ui_options_default_values() {
        let opts = UiOptions::default();
        assert_eq!(opts.port, 4000);
        assert!(!opts.open);
        assert!(!opts.watch);
        assert!(opts.report.is_none());
        assert_eq!(opts.project, std::path::PathBuf::from("."));
    }

    #[tokio::test]
    async fn get_index_returns_200_with_html() {
        let app = create_router(test_state());
        let resp = app
            .oneshot(Request::get("/").body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let html = std::str::from_utf8(&body).unwrap();
        assert!(
            html.contains("UTO UI"),
            "index.html should contain 'UTO UI'"
        );
        assert!(html.contains("<html"), "index.html should be valid HTML");
    }

    #[tokio::test]
    async fn api_status_returns_project_name() {
        let app = create_router(test_state());
        let resp = app
            .oneshot(
                Request::get("/api/status")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["project"], "test-project");
        assert_eq!(json["status"], "ready");
    }

    #[tokio::test]
    async fn api_status_shows_running_when_active() {
        let state = test_state();
        state.run_active.store(true, Ordering::SeqCst);
        let app = create_router(state);
        let resp = app
            .oneshot(
                Request::get("/api/status")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "running");
    }

    #[tokio::test]
    async fn api_report_returns_null_when_no_report_loaded() {
        let app = create_router(test_state());
        let resp = app
            .oneshot(
                Request::get("/api/report")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(
            json.is_null(),
            "should return null when no report is loaded"
        );
    }

    #[tokio::test]
    async fn api_report_returns_report_when_loaded() {
        let fake_report = serde_json::json!({
            "schema_version": "uto-suite/v1",
            "suite_id": "s1",
            "status": "passed",
        });
        let app = create_router(test_state_with_report(fake_report.clone()));
        let resp = app
            .oneshot(
                Request::get("/api/report")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["schema_version"], "uto-suite/v1");
        assert_eq!(json["status"], "passed");
    }

    #[tokio::test]
    async fn api_tests_returns_discovered_tests() {
        let temp = tempfile::tempdir().unwrap();
        let tests_dir = temp.path().join("tests");
        std::fs::create_dir_all(&tests_dir).unwrap();
        std::fs::write(
            tests_dir.join("web_example.rs"),
            "#[tokio::test]\nasync fn web_login() {}\n",
        )
        .unwrap();

        let (tx, _) = broadcast::channel(16);
        let app = create_router(AppState {
            project_name: "p".to_string(),
            project: temp.path().to_path_buf(),
            report: Arc::new(RwLock::new(None)),
            tx,
            run_active: Arc::new(AtomicBool::new(false)),
            kill_handle: Arc::new(Mutex::new(None)),
            studio_mode: false,
            studio: Arc::new(RwLock::new(crate::studio::StudioState::default())),
        });

        let resp = app
            .oneshot(
                Request::get("/api/tests")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["tests"].as_array().unwrap().len(), 1);
        assert_eq!(json["tests"][0]["test_name"], "web_login");
    }

    #[tokio::test]
    async fn handle_trigger_run_is_idempotent_when_active() {
        let state = test_state();
        let mut rx = state.tx.subscribe();
        // Pre-set run_active so that a second trigger_run is ignored.
        state.run_active.store(true, Ordering::SeqCst);
        handle_trigger_run(state, None).await;
        // Should have broadcast run_ignored.
        let msg = rx.try_recv().expect("should have received run_ignored");
        let v: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(v["type"], "run_ignored");
    }

    #[tokio::test]
    async fn handle_stop_run_is_noop_when_no_run_active() {
        let state = test_state();
        // Should not panic when there is nothing to stop.
        handle_stop_run(state).await;
    }

    #[tokio::test]
    async fn derive_project_name_falls_back_to_dir_name() {
        let tmp = tempfile::tempdir().unwrap();
        // No uto.json — should use directory name
        let name = derive_project_name(tmp.path());
        // temp dir names are OS-generated; we just verify the function returns something non-empty.
        assert!(!name.is_empty());
    }

    #[tokio::test]
    async fn derive_project_name_reads_uto_json() {
        let tmp = tempfile::tempdir().unwrap();
        let config = serde_json::json!({ "project_name": "my-uto-app" });
        std::fs::write(tmp.path().join("uto.json"), config.to_string()).unwrap();
        assert_eq!(derive_project_name(tmp.path()), "my-uto-app");
    }
}
