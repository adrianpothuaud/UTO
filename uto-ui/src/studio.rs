//! UTO Studio — Phase 6 visual test authoring and recording backend.
//!
//! This module provides the server-side scaffold for Studio mode, which is
//! activated via `uto ui --studio`.  It exposes REST endpoints for managing
//! recording sessions and accumulating recorded steps, then generates
//! `uto-test` Rust code from the captured interaction sequence.
//!
//! ## Endpoints
//!
//! | Method | Path                  | Description                                      |
//! |--------|-----------------------|--------------------------------------------------|
//! | GET    | `/api/studio/status`  | Returns recording state and accumulated steps    |
//! | POST   | `/api/studio/start`   | Starts a new recording session                   |
//! | POST   | `/api/studio/stop`    | Stops the session and returns generated Rust code |
//! | POST   | `/api/studio/step`    | Appends a recorded step to the active session    |
//!
//! ## Recording model
//!
//! Each recorded step is a [`RecordedStep`] that maps to a single `uto-test`
//! intent API call.  Steps are accumulated in [`StudioState`] and rendered
//! into a complete Rust test function by [`generate_test_code`].
//!
//! ## Phase 6 Roadmap
//!
//! The current implementation is the **scaffold** for Phase 6 Iteration 6.1.
//! Subsequent iterations will add:
//!
//! - **6.2** Vision inspector overlay — bounding box / confidence overlays via
//!   WebSocket events pushed from a live WebDriver session.
//! - **6.3** File output — write the generated Rust code back to the project
//!   `tests/` directory as a named test function.
//! - **6.4** Assertion builder — pause recording and add `assert_visible` /
//!   `assert_text` / `assert_gone` steps interactively.

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The kind of interaction captured as a single recorded step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StepKind {
    /// User navigated to a URL.
    Navigate,
    /// User clicked a recognized element.
    Click,
    /// User typed into a recognized element.
    Fill,
    /// User asserted an element is visible.
    AssertVisible,
    /// User asserted an element text matches a value.
    AssertText,
    /// User asserted an element is gone / not visible.
    AssertGone,
    /// Custom / free-form step (used for future extensibility).
    Custom,
}

/// A single interaction step captured during a Studio recording session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedStep {
    /// The type of interaction.
    pub kind: StepKind,
    /// The primary target label or URL for the step.
    pub target: String,
    /// Optional value (e.g. text to type, expected assertion text).
    pub value: Option<String>,
    /// Wall-clock offset from session start in milliseconds.
    pub ts_ms: u64,
}

/// Mutable state for an active Studio recording session.
#[derive(Debug, Default)]
pub struct StudioState {
    /// Whether a recording session is currently active.
    pub recording: bool,
    /// Accumulated recorded steps (appended during an active session).
    pub steps: Vec<RecordedStep>,
    /// Name to use for the generated test function.
    pub test_name: String,
    /// Session start timestamp (Unix millis, 0 when not active).
    pub started_at_ms: u64,
}

// ---------------------------------------------------------------------------
// Code generation
// ---------------------------------------------------------------------------

/// Render a list of recorded steps into a complete Rust test function.
///
/// The generated function uses the `uto-test` helper API and can be pasted
/// directly into a `tests/` file in any UTO project.
///
/// # Example
///
/// ```text
/// #[tokio::test]
/// async fn my_recording() -> uto_core::error::UtoResult<()> {
///     let s = uto_test::startNewSession("chrome").await?;
///     s.goto("https://example.com").await?;
///     s.click_intent("Login").await?;
///     s.fill_intent("Username", "alice").await?;
///     s.assert_visible("Dashboard").await?;
///     s.close().await
/// }
/// ```
pub fn generate_test_code(test_name: &str, steps: &[RecordedStep]) -> String {
    let fn_name = sanitize_fn_name(test_name);
    let mut lines: Vec<String> = Vec::new();

    lines.push("#[tokio::test]".to_string());
    lines.push(format!(
        "async fn {fn_name}() -> uto_core::error::UtoResult<()> {{"
    ));
    lines.push(r#"    let s = uto_test::startNewSession("chrome").await?;"#.to_string());

    for step in steps {
        let stmt = match step.kind {
            StepKind::Navigate => {
                format!(r#"    s.goto("{}").await?;"#, escape_string(&step.target))
            }
            StepKind::Click => {
                format!(
                    r#"    s.click_intent("{}").await?;"#,
                    escape_string(&step.target)
                )
            }
            StepKind::Fill => {
                let value = step.value.as_deref().unwrap_or("");
                format!(
                    r#"    s.fill_intent("{}", "{}").await?;"#,
                    escape_string(&step.target),
                    escape_string(value)
                )
            }
            StepKind::AssertVisible => {
                format!(
                    r#"    s.assert_visible("{}").await?;"#,
                    escape_string(&step.target)
                )
            }
            StepKind::AssertText => {
                let value = step.value.as_deref().unwrap_or("");
                format!(
                    r#"    s.assert_text("{}", "{}").await?;"#,
                    escape_string(&step.target),
                    escape_string(value)
                )
            }
            StepKind::AssertGone => {
                format!(
                    r#"    s.assert_gone("{}").await?;"#,
                    escape_string(&step.target)
                )
            }
            StepKind::Custom => {
                format!("    // custom: {}", escape_string(&step.target))
            }
        };
        lines.push(stmt);
    }

    lines.push("    s.close().await".to_string());
    lines.push("}".to_string());
    lines.join("\n")
}

/// Convert a free-form name into a valid Rust identifier (snake_case).
fn sanitize_fn_name(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    // Collapse consecutive underscores and trim leading/trailing ones.
    let cleaned: String = slug
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    if cleaned.is_empty() {
        "recorded_test".to_string()
    } else {
        cleaned
    }
}

/// Escape double-quotes and backslashes in a string literal.
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

// ---------------------------------------------------------------------------
// Request / response bodies
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct StartSessionRequest {
    /// Optional custom name for the generated test function.
    pub test_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StudioStatusResponse {
    pub recording: bool,
    pub step_count: usize,
    pub test_name: String,
    pub started_at_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct StopSessionResponse {
    pub recording: bool,
    pub step_count: usize,
    pub generated_code: String,
}

// ---------------------------------------------------------------------------
// Axum handler types
// ---------------------------------------------------------------------------

/// Shared studio state handle used by Axum handlers.
pub type StudioHandle = Arc<RwLock<StudioState>>;

// ---------------------------------------------------------------------------
// Axum handlers
// ---------------------------------------------------------------------------

/// `GET /api/studio/status` — return current recording state.
pub(crate) async fn api_studio_status(State(state): State<crate::server::AppState>) -> impl IntoResponse {
    let guard = state.studio.read().await;
    Json(StudioStatusResponse {
        recording: guard.recording,
        step_count: guard.steps.len(),
        test_name: guard.test_name.clone(),
        started_at_ms: guard.started_at_ms,
    })
}

/// `POST /api/studio/start` — start a new recording session.
///
/// Resets any previously accumulated steps.  If a session is already active
/// this is a no-op and the response will still indicate `recording: true`.
/// The request body is optional; if absent the test function is named
/// `"recorded_test"`.
pub(crate) async fn api_studio_start(
    State(state): State<crate::server::AppState>,
    body: Result<Json<StartSessionRequest>, axum::extract::rejection::JsonRejection>,
) -> impl IntoResponse {
    let test_name = body
        .ok()
        .and_then(|b| b.test_name.clone())
        .unwrap_or_else(|| "recorded_test".to_string());
    let mut guard = state.studio.write().await;
    if !guard.recording {
        guard.recording = true;
        guard.steps.clear();
        guard.test_name = test_name;
        guard.started_at_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        log::info!("Studio recording started — test_name='{}'", guard.test_name);
    }
    Json(StudioStatusResponse {
        recording: guard.recording,
        step_count: guard.steps.len(),
        test_name: guard.test_name.clone(),
        started_at_ms: guard.started_at_ms,
    })
}

/// `POST /api/studio/stop` — stop the active recording session.
///
/// Returns the accumulated steps and the generated Rust test code.
pub(crate) async fn api_studio_stop(State(state): State<crate::server::AppState>) -> impl IntoResponse {
    let mut guard = state.studio.write().await;
    guard.recording = false;
    let code = generate_test_code(&guard.test_name, &guard.steps);
    log::info!(
        "Studio recording stopped — {} step(s) captured",
        guard.steps.len()
    );
    Json(StopSessionResponse {
        recording: false,
        step_count: guard.steps.len(),
        generated_code: code,
    })
}

/// `POST /api/studio/step` — append a recorded step to the active session.
///
/// If no session is active the step is silently discarded (the caller is
/// responsible for checking studio status first).
pub(crate) async fn api_studio_add_step(
    State(state): State<crate::server::AppState>,
    Json(step): Json<RecordedStep>,
) -> impl IntoResponse {
    let mut guard = state.studio.write().await;
    if guard.recording {
        log::debug!("Studio step recorded: {:?} → '{}'", step.kind, step.target);
        guard.steps.push(step);
    }
    Json(serde_json::json!({
        "recording": guard.recording,
        "step_count": guard.steps.len(),
    }))
}

// ---------------------------------------------------------------------------
// Public re-export for server.rs access
// ---------------------------------------------------------------------------

/// Re-export `AppState` reference used by studio handler signatures.
///
/// Handlers are declared in this module but referenced from [`server::create_router`].
/// They receive `State<AppState>` from Axum — the AppState type lives in
/// `server.rs` to keep the server boundary clean, and studio just adds fields
/// to it.  This module receives the full `AppState` rather than just the studio
/// sub-state to avoid awkward double-state nesting.

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_code_empty_steps() {
        let code = generate_test_code("my test", &[]);
        assert!(code.contains("async fn my_test()"));
        assert!(code.contains("startNewSession"));
        assert!(code.contains("s.close().await"));
    }

    #[test]
    fn generate_code_navigate_and_click() {
        let steps = vec![
            RecordedStep {
                kind: StepKind::Navigate,
                target: "https://example.com".to_string(),
                value: None,
                ts_ms: 0,
            },
            RecordedStep {
                kind: StepKind::Click,
                target: "Login".to_string(),
                value: None,
                ts_ms: 500,
            },
        ];
        let code = generate_test_code("login_flow", &steps);
        assert!(code.contains(r#"s.goto("https://example.com")"#));
        assert!(code.contains(r#"s.click_intent("Login")"#));
    }

    #[test]
    fn generate_code_fill_and_assert() {
        let steps = vec![
            RecordedStep {
                kind: StepKind::Fill,
                target: "Username".to_string(),
                value: Some("alice".to_string()),
                ts_ms: 0,
            },
            RecordedStep {
                kind: StepKind::AssertVisible,
                target: "Dashboard".to_string(),
                value: None,
                ts_ms: 1000,
            },
        ];
        let code = generate_test_code("fill_test", &steps);
        assert!(code.contains(r#"s.fill_intent("Username", "alice")"#));
        assert!(code.contains(r#"s.assert_visible("Dashboard")"#));
    }

    #[test]
    fn generate_code_assert_text() {
        let steps = vec![RecordedStep {
            kind: StepKind::AssertText,
            target: "Status".to_string(),
            value: Some("Active".to_string()),
            ts_ms: 0,
        }];
        let code = generate_test_code("status_check", &steps);
        assert!(code.contains(r#"s.assert_text("Status", "Active")"#));
    }

    #[test]
    fn generate_code_assert_gone() {
        let steps = vec![RecordedStep {
            kind: StepKind::AssertGone,
            target: "Loading spinner".to_string(),
            value: None,
            ts_ms: 0,
        }];
        let code = generate_test_code("spinner_gone", &steps);
        assert!(code.contains(r#"s.assert_gone("Loading spinner")"#));
    }

    #[test]
    fn sanitize_fn_name_basic() {
        assert_eq!(sanitize_fn_name("My Test Flow"), "my_test_flow");
        assert_eq!(sanitize_fn_name("login flow!"), "login_flow");
        assert_eq!(sanitize_fn_name(""), "recorded_test");
        assert_eq!(sanitize_fn_name("  "), "recorded_test");
    }

    #[test]
    fn escape_string_special_chars() {
        assert_eq!(escape_string(r#"say "hello""#), r#"say \"hello\""#);
        assert_eq!(escape_string(r"path\to\file"), r"path\\to\\file");
    }
}
