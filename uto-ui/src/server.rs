//! Axum-based HTTP + WebSocket server for the UTO UI mode.
//!
//! ## Endpoints
//!
//! | Method | Path         | Description                                               |
//! |--------|--------------|-----------------------------------------------------------|
//! | GET    | `/`          | Serve embedded SPA (`index.html`)                         |
//! | GET    | `/api/status`| JSON `{ project, status }` — server health / project info |
//! | GET    | `/api/report`| JSON report artifact loaded at startup, or `null`         |
//! | GET    | `/ws`        | WebSocket upgrade — live event stream / run relay         |

use std::path::PathBuf;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use tokio::sync::broadcast;

/// Embedded SPA — compiled into the binary at build time.
const INDEX_HTML: &str = include_str!("assets/index.html");

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Configuration options for the UI server.
#[derive(Debug, Clone)]
pub struct UiOptions {
    /// Path to the UTO project directory (used for project name display and future run integration).
    pub project: PathBuf,
    /// Local port for the HTTP + WebSocket server. Default: `4000`.
    pub port: u16,
    /// Automatically open the browser after the server starts. Default: `false`.
    pub open: bool,
    /// Enable watch mode (auto re-run on file change). Default: `false`.
    /// Note: filesystem watcher integration is planned for Iteration 5.4.
    pub watch: bool,
    /// Path to a saved `uto-suite/v1` or `uto-report/v1` JSON artifact to replay. Default: `None`.
    pub report: Option<PathBuf>,
}

impl Default for UiOptions {
    fn default() -> Self {
        Self {
            project: PathBuf::from("."),
            port: 4000,
            open: false,
            watch: false,
            report: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    /// Display name derived from project directory or `uto.json`.
    project_name: String,
    /// Pre-loaded report artifact (from `--report` flag), or `None`.
    report: Option<serde_json::Value>,
    /// Broadcast channel for streaming live events to WebSocket clients.
    tx: broadcast::Sender<String>,
}

// ---------------------------------------------------------------------------
// Router factory (also used by tests)
// ---------------------------------------------------------------------------

fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(serve_index))
        .route("/api/status", get(api_status))
        .route("/api/report", get(api_report))
        .route("/ws", get(ws_handler))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

/// Start the UTO UI server asynchronously.
///
/// Binds an HTTP + WebSocket server on `localhost:<port>`, serves the embedded
/// SPA, and optionally replays a saved report artifact over WebSocket on
/// client connect. Blocks until `SIGINT` / `Ctrl+C` is received.
pub async fn start_server(opts: UiOptions) -> Result<(), String> {
    // Load the report artifact if one was specified.
    let report = if let Some(ref path) = opts.report {
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
    let state = AppState {
        project_name,
        report,
        tx,
    };

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
    if opts.watch {
        println!("  Watch mode: planned for Iteration 5.4");
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
        "status": "ready",
    }))
}

async fn api_report(State(state): State<AppState>) -> impl IntoResponse {
    match state.report {
        Some(r) => Json(r).into_response(),
        None => Json(serde_json::Value::Null).into_response(),
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();

    // If a report artifact was loaded at startup, push it to the new client.
    if let Some(report) = &state.report {
        let msg = serde_json::json!({ "type": "report", "payload": report });
        if socket
            .send(Message::Text(msg.to_string().into()))
            .await
            .is_err()
        {
            return;
        }
    }

    // Relay broadcast events and listen for client control messages.
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
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
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

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install Ctrl+C handler");
    println!("\nStopping UTO UI server…");
}

fn open_browser(url: &str) {
    let result = if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(url).spawn().ok()
    } else if cfg!(target_os = "windows") {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()
            .ok()
    } else {
        std::process::Command::new("xdg-open").arg(url).spawn().ok()
    };

    if result.is_none() {
        log::warn!("Could not open browser automatically. Navigate to {url}");
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
            report: None,
            tx,
        }
    }

    fn test_state_with_report(report: serde_json::Value) -> AppState {
        let (tx, _) = broadcast::channel(16);
        AppState {
            project_name: "report-project".to_string(),
            report: Some(report),
            tx,
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
