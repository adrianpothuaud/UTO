//! Subprocess bridge for live test run integration.
//!
//! Spawns `cargo run --bin uto_project_runner` inside a UTO project directory,
//! relays stdout and stderr as `log` WebSocket events, then broadcasts
//! `run_started` / `run_finished` on the shared broadcast channel.

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{broadcast, oneshot, RwLock};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Opaque handle returned by [`start_run`].
///
/// Call [`KillHandle::kill`] to send a graceful stop signal to the running
/// subprocess.  Dropping the handle without calling `kill` does not stop the
/// subprocess — the run continues to completion.
pub struct KillHandle {
    kill_tx: oneshot::Sender<()>,
}

impl KillHandle {
    /// Signal the subprocess to stop (best-effort SIGKILL / TerminateProcess).
    pub fn kill(self) {
        let _ = self.kill_tx.send(());
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Spawn a live test run subprocess.
///
/// Runs `cargo run --bin uto_project_runner -- --json --report-file <path>` in
/// `project`.  As lines are written to the child's stdout/stderr they are
/// broadcast as `{ "type": "log", "payload": { "line": "…" } }` messages.
/// On completion a `{ "type": "run_finished", "payload": { "status": "…" } }`
/// message is broadcast and `report_store` is updated with the freshly written
/// JSON artifact.
///
/// `run_active` is set to `true` before the task starts and `false` when it
/// finishes, so callers can detect whether a run is in progress.
pub async fn start_run(
    project: PathBuf,
    tx: broadcast::Sender<String>,
    report_store: Arc<RwLock<Option<serde_json::Value>>>,
    run_active: Arc<AtomicBool>,
) -> KillHandle {
    let (kill_tx, kill_rx) = oneshot::channel();
    tokio::spawn(run_task(project, tx, report_store, run_active, kill_rx));
    KillHandle { kill_tx }
}

// ---------------------------------------------------------------------------
// Internal run task
// ---------------------------------------------------------------------------

async fn run_task(
    project: PathBuf,
    tx: broadcast::Sender<String>,
    report_store: Arc<RwLock<Option<serde_json::Value>>>,
    run_active: Arc<AtomicBool>,
    kill_rx: oneshot::Receiver<()>,
) {
    // Notify all WebSocket clients that a run is starting.
    let _ = tx.send(serde_json::json!({ "type": "run_started" }).to_string());

    // Ensure the report directory exists.
    let report_dir = project.join(".uto/reports");
    if let Err(e) = std::fs::create_dir_all(&report_dir) {
        log::warn!(
            "Could not create report directory '{}': {e}",
            report_dir.display()
        );
    }
    let report_file = report_dir.join("last-run.json");

    // Spawn the project runner subprocess.
    let mut child = match tokio::process::Command::new("cargo")
        .current_dir(&project)
        .args([
            "run",
            "--bin",
            "uto_project_runner",
            "--",
            "--json",
            "--report-file",
        ])
        .arg(&report_file)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(
                serde_json::json!({
                    "type": "run_finished",
                    "payload": {
                        "status": "failed",
                        "error": format!(
                            "Failed to start test run: {e}. \
                             Ensure 'uto_project_runner' binary is defined in the project \
                             and 'cargo' is available in PATH."
                        )
                    }
                })
                .to_string(),
            );
            run_active.store(false, Ordering::SeqCst);
            return;
        }
    };

    // Relay stdout lines as log events.
    let stdout = child.stdout.take().expect("stdout was piped");
    let tx_out = tx.clone();
    let stdout_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = tx_out.send(
                serde_json::json!({ "type": "log", "payload": { "line": line } }).to_string(),
            );
        }
    });

    // Relay stderr lines as log events.
    let stderr = child.stderr.take().expect("stderr was piped");
    let tx_err = tx.clone();
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = tx_err.send(
                serde_json::json!({ "type": "log", "payload": { "line": line } }).to_string(),
            );
        }
    });

    // Wait for process exit or an external stop signal.
    // `stopped` distinguishes a deliberate user stop from a natural exit.
    let (exit_ok, stopped) = tokio::select! {
        status = child.wait() => (status.map(|s| s.success()).unwrap_or(false), false),
        _ = kill_rx => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            (false, true)
        }
    };

    // Drain IO relay tasks before reading the report file.
    let _ = tokio::join!(stdout_task, stderr_task);

    // A deliberate stop is reported as "stopped" rather than "failed".
    let status_str = if stopped {
        "stopped".to_string()
    } else {
        // Load the freshly written report artifact (if any) and update shared state.
        let loaded = load_report_file(&report_file);
        if let Some(ref v) = loaded {
            *report_store.write().await = Some(v.clone());
        }
        status_from_report(&loaded, exit_ok)
    };

    // Broadcast run_finished so the SPA can refresh its view.
    let _ = tx.send(
        serde_json::json!({
            "type": "run_finished",
            "payload": { "status": status_str }
        })
        .to_string(),
    );

    // Mark the slot as available for the next run.
    run_active.store(false, Ordering::SeqCst);
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn load_report_file(path: &std::path::Path) -> Option<serde_json::Value> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

fn status_from_report(report: &Option<serde_json::Value>, exit_ok: bool) -> String {
    if let Some(v) = report {
        if let Some(s) = v.get("status").and_then(|s| s.as_str()) {
            return s.to_string();
        }
    }
    if exit_ok { "passed" } else { "failed" }.to_string()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kill_handle_can_be_killed() {
        let (tx, _rx) = oneshot::channel::<()>();
        let handle = KillHandle { kill_tx: tx };
        // Calling kill() should not panic even if the receiver has been dropped.
        handle.kill();
    }

    #[test]
    fn status_from_report_uses_json_status() {
        let report = Some(serde_json::json!({ "status": "partial" }));
        assert_eq!(status_from_report(&report, true), "partial");
    }

    #[test]
    fn status_from_report_falls_back_to_exit_code() {
        assert_eq!(status_from_report(&None, true), "passed");
        assert_eq!(status_from_report(&None, false), "failed");
    }

    #[test]
    fn load_report_file_returns_none_for_missing_file() {
        let path = std::path::Path::new("/tmp/uto-test-nonexistent-report-file.json");
        assert!(load_report_file(path).is_none());
    }

    #[test]
    fn load_report_file_parses_valid_json() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(tmp.path(), r#"{"status":"passed"}"#).unwrap();
        let v = load_report_file(tmp.path()).expect("should parse");
        assert_eq!(v["status"], "passed");
    }
}
