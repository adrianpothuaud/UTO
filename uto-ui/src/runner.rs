//! Subprocess bridge for live test run integration.
//!
//! Spawns `uto run --project <dir>` via the current `uto` executable,
//! relays stdout and stderr as `log` WebSocket events, then broadcasts
//! `run_started` / `run_finished` on the shared broadcast channel.

use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{broadcast, oneshot, RwLock};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSelection {
    pub test_bin: String,
    pub test_name: String,
}

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
/// Runs `uto run --project <project> --target web --report-json <path>`.
/// As lines are written to the child's stdout/stderr they are
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
    selection: Option<RunSelection>,
) -> KillHandle {
    let (kill_tx, kill_rx) = oneshot::channel();
    tokio::spawn(run_task(
        project,
        tx,
        report_store,
        run_active,
        kill_rx,
        selection,
    ));
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
    selection: Option<RunSelection>,
) {
    let run_start = Instant::now();
    let run_start_unix_ms = now_unix_ms();

    // Notify all WebSocket clients that a run is starting.
    let _ = tx.send(
        serde_json::json!({
            "type": "run_started",
            "payload": { "ts_ms": 0 }
        })
        .to_string(),
    );

    // Ensure the report directory exists.
    let report_dir = project.join(".uto/reports");
    if let Err(e) = std::fs::create_dir_all(&report_dir) {
        log::warn!(
            "Could not create report directory '{}': {e}",
            report_dir.display()
        );
    }
    let report_file = report_dir.join("last-run.json");
    let live_event_file = report_dir.join("live-run-events.jsonl");
    if live_event_file.exists() {
        let _ = std::fs::remove_file(&live_event_file);
    }

    let current_exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            let _ = tx.send(
                serde_json::json!({
                    "type": "run_finished",
                    "payload": {
                        "ts_ms": elapsed_ms(&run_start),
                        "status": "failed",
                        "error": format!("Failed to resolve current executable: {e}")
                    }
                })
                .to_string(),
            );
            run_active.store(false, Ordering::SeqCst);
            return;
        }
    };

    // Spawn the CLI-owned test execution subprocess.
    let mut cmd = tokio::process::Command::new(current_exe);
    cmd.args(["run", "--project"])
        .arg(&project)
        .args(["--target", "web", "--report-json"])
        .arg(&report_file)
        .args(["--live-events-jsonl"])
        .arg(&live_event_file)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    if let Some(sel) = &selection {
        cmd.arg("--test-bin")
            .arg(&sel.test_bin)
            .arg("--test-name")
            .arg(&sel.test_name);
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(
                serde_json::json!({
                    "type": "run_finished",
                    "payload": {
                        "ts_ms": elapsed_ms(&run_start),
                        "status": "failed",
                        "error": format!(
                            "Failed to start test run: {e}. \
                             Ensure the UTO CLI executable can launch subprocesses."
                        )
                    }
                })
                .to_string(),
            );
            run_active.store(false, Ordering::SeqCst);
            return;
        }
    };

    let (live_stop_tx, live_stop_rx) = oneshot::channel();
    let tx_live = tx.clone();
    let live_events_task = tokio::spawn(async move {
        relay_live_events(live_event_file, tx_live, run_start_unix_ms, live_stop_rx).await;
    });

    // Relay stdout lines as log events.
    let stdout = child.stdout.take().expect("stdout was piped");
    let tx_out = tx.clone();
    let run_start_out = run_start;
    let stdout_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = tx_out.send(
                serde_json::json!({
                    "type": "log",
                    "payload": {
                        "line": line,
                        "ts_ms": elapsed_ms(&run_start_out)
                    }
                })
                .to_string(),
            );
        }
    });

    // Relay stderr lines as log events.
    let stderr = child.stderr.take().expect("stderr was piped");
    let tx_err = tx.clone();
    let run_start_err = run_start;
    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ = tx_err.send(
                serde_json::json!({
                    "type": "log",
                    "payload": {
                        "line": line,
                        "ts_ms": elapsed_ms(&run_start_err)
                    }
                })
                .to_string(),
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
    let _ = live_stop_tx.send(());
    let _ = live_events_task.await;

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
            "payload": {
                "status": status_str,
                "ts_ms": elapsed_ms(&run_start)
            }
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

fn elapsed_ms(start: &Instant) -> u128 {
    start.elapsed().as_millis()
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

async fn relay_live_events(
    path: PathBuf,
    tx: broadcast::Sender<String>,
    run_start_unix_ms: u64,
    mut stop_rx: oneshot::Receiver<()>,
) {
    let mut consumed_bytes = 0usize;
    let mut carry = String::new();

    loop {
        tokio::select! {
            _ = sleep(Duration::from_millis(25)) => {
                if drain_live_event_stream(&path, &tx, run_start_unix_ms, &mut consumed_bytes, &mut carry).await.is_err() {
                    break;
                }
            }
            _ = &mut stop_rx => {
                let _ = drain_live_event_stream(&path, &tx, run_start_unix_ms, &mut consumed_bytes, &mut carry).await;
                if !carry.trim().is_empty() {
                    let _ = forward_live_event_line(carry.trim(), &tx, run_start_unix_ms);
                }
                break;
            }
        }
    }
}

async fn drain_live_event_stream(
    path: &std::path::Path,
    tx: &broadcast::Sender<String>,
    run_start_unix_ms: u64,
    consumed_bytes: &mut usize,
    carry: &mut String,
) -> Result<(), String> {
    let data = match tokio::fs::read(path).await {
        Ok(data) => data,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(err) => {
            return Err(format!(
                "failed to read live event stream '{}': {err}",
                path.display()
            ));
        }
    };

    if data.len() <= *consumed_bytes {
        return Ok(());
    }

    let delta = &data[*consumed_bytes..];
    *consumed_bytes = data.len();
    let delta_text = std::str::from_utf8(delta).map_err(|err| {
        format!(
            "live event stream '{}' contained invalid UTF-8: {err}",
            path.display()
        )
    })?;
    carry.push_str(delta_text);

    while let Some(newline_idx) = carry.find('\n') {
        let line = carry[..newline_idx].trim().to_string();
        let remainder = carry[newline_idx + 1..].to_string();
        *carry = remainder;
        if line.is_empty() {
            continue;
        }
        forward_live_event_line(&line, tx, run_start_unix_ms)?;
    }

    Ok(())
}

fn forward_live_event_line(
    line: &str,
    tx: &broadcast::Sender<String>,
    run_start_unix_ms: u64,
) -> Result<(), String> {
    let envelope: uto_runner::LiveEventEnvelope =
        serde_json::from_str(line).map_err(|err| format!("invalid live event payload: {err}"))?;
    let ts_ms = envelope.ts_unix_ms.saturating_sub(run_start_unix_ms);

    match envelope.payload {
        uto_runner::LiveEventPayload::TestStarted => {
            let _ = tx.send(
                serde_json::json!({
                    "type": "test_started",
                    "payload": {
                        "test_bin": envelope.test_bin,
                        "test_name": envelope.test_name,
                        "target": envelope.target,
                        "ts_ms": ts_ms,
                    }
                })
                .to_string(),
            );
        }
        uto_runner::LiveEventPayload::ReportEvent { event } => {
            let _ = tx.send(
                serde_json::json!({
                    "type": "event",
                    "payload": {
                        "test_bin": envelope.test_bin,
                        "test_name": envelope.test_name,
                        "target": envelope.target,
                        "event": {
                            "stage": event.stage,
                            "status": event.status,
                            "detail": event.detail,
                            "ts_ms": ts_ms,
                        }
                    }
                })
                .to_string(),
            );
        }
        uto_runner::LiveEventPayload::TestFinished {
            status,
            error,
            duration_ms,
        } => {
            let _ = tx.send(
                serde_json::json!({
                    "type": "test_finished",
                    "payload": {
                        "test_bin": envelope.test_bin,
                        "test_name": envelope.test_name,
                        "target": envelope.target,
                        "status": status,
                        "error": error,
                        "duration_ms": duration_ms,
                        "ts_ms": ts_ms,
                    }
                })
                .to_string(),
            );
        }
    }

    Ok(())
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

    #[test]
    fn elapsed_ms_is_non_negative() {
        let start = Instant::now();
        let elapsed = elapsed_ms(&start);
        assert!(
            elapsed <= 10_000,
            "unexpectedly large elapsed value: {elapsed}"
        );
    }
}
