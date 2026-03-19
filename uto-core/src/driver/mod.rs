use std::path::PathBuf;
use std::time::Duration;

use command_group::{CommandGroup, GroupChild};
use portpicker::pick_unused_port;

use crate::error::{UtoError, UtoResult};

// ---------------------------------------------------------------------------
// Driver variant
// ---------------------------------------------------------------------------

/// The type of WebDriver-compatible server being managed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverKind {
    /// ChromeDriver — serves the W3C WebDriver protocol for Chrome.
    ChromeDriver,
    /// Appium — serves the W3C WebDriver protocol for mobile platforms.
    Appium,
}

// ---------------------------------------------------------------------------
// DriverProcess
// ---------------------------------------------------------------------------

/// A running WebDriver-compatible server process managed by UTO.
///
/// The process is spawned inside its own OS **process group** (Unix) or
/// **Job Object** (Windows) so that it is automatically killed when the
/// parent UTO process exits — even in the face of a panic or SIGKILL.
///
/// Call [`DriverProcess::stop`] for a clean, explicit shutdown.
pub struct DriverProcess {
    /// The type of driver.
    pub kind: DriverKind,
    /// The TCP port the driver is listening on.
    pub port: u16,
    /// The full URL of the driver's WebDriver endpoint.
    pub url: String,
    /// Handle to the underlying OS process group.
    child: GroupChild,
}

impl DriverProcess {
    /// Attempts a graceful shutdown of the driver process.
    pub fn stop(mut self) -> UtoResult<()> {
        self.child
            .kill()
            .map_err(|e| UtoError::DriverStopFailed(format!("{e}")))?;
        self.child
            .wait()
            .map_err(|e| UtoError::DriverStopFailed(format!("{e}")))?;
        log::info!("{:?} stopped (port {})", self.kind, self.port);
        Ok(())
    }
}

impl Drop for DriverProcess {
    /// Best-effort process-group cleanup on drop.
    ///
    /// This ensures driver processes are killed when their owner is dropped
    /// without an explicit `stop()` call (e.g. after a test panic or early return).
    ///
    /// # RUST LEARNING: The Drop trait
    ///
    /// `Drop` is similar to destructors in C++ or `finally` blocks in Java.
    /// It's automatically called when a value goes out of scope:
    ///
    /// ```rust,ignore
    /// {
    ///     let driver = DriverProcess::start(...).await?;
    ///     // ... use driver ...
    /// } // <- Drop::drop() called here automatically
    /// ```
    ///
    /// **Why is this important?**
    /// If a test panics or returns early, Rust guarantees `drop()` runs,
    /// preventing orphaned driver processes that would consume system resources.
    ///
    /// **Error handling in Drop:**
    /// We use `let _ = ...` to ignore errors because `drop()` can't return a Result.
    /// This is a "best-effort" cleanup - we try to kill the process, but if it
    /// fails, there's nothing we can do at this point.
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

// ---------------------------------------------------------------------------
// Starting ChromeDriver
// ---------------------------------------------------------------------------

/// Starts a `chromedriver` process on a randomly chosen free port.
///
/// `binary_path` should point to the `chromedriver` executable that was
/// either discovered on the system or provisioned by [`crate::env::provisioning`].
///
/// The driver is ready to accept connections once this function returns.
///
/// # RUST LEARNING: Error propagation with `?`
///
/// This function uses the `?` operator extensively for clean error handling.
/// Each `?` checks if the operation succeeded (Ok) or failed (Err):
/// - If Ok: unwraps the value and continues
/// - If Err: immediately returns the error, converted to UtoError
pub async fn start_chromedriver(binary_path: &PathBuf) -> UtoResult<DriverProcess> {
    // RUST LEARNING NOTE: Option to Result conversion
    //
    // `pick_unused_port()` returns `Option<u16>`:
    // - `Some(port)` if a free port was found
    // - `None` if no ports are available
    //
    // `.ok_or(...)` converts:
    // - `Some(port)` -> `Ok(port)`
    // - `None` -> `Err(UtoError::NoFreePort)`
    //
    // The `?` then unwraps Ok(port) or returns early with the error.
    let port = pick_unused_port().ok_or(UtoError::NoFreePort)?;
    let port_arg = format!("--port={port}");

    log::info!(
        "Starting chromedriver on port {} from {}",
        port,
        binary_path.display()
    );

    // RUST LEARNING NOTE: Process groups for clean shutdown
    //
    // `.group_spawn()` (from `command_group` crate) is crucial for clean shutdown:
    //
    // - On Unix: Creates a new process group (PGID)
    // - On Windows: Creates a Job Object
    //
    // This ensures that when we kill the driver process, all its child processes
    // (like Chrome instances it spawns) are also terminated. Without this, orphaned
    // processes would survive after tests finish.
    //
    // `.map_err(|e| ...)` transforms the I/O error into a UtoError with context.
    // `format!("{e}")` converts the error to a string for our error message.
    let child = std::process::Command::new(binary_path)
        .arg(&port_arg)
        .stdout(std::process::Stdio::null()) // Discard stdout
        .stderr(std::process::Stdio::null()) // Discard stderr
        .group_spawn()
        .map_err(|e| UtoError::DriverStartFailed(format!("{e}")))?;

    let url = format!("http://localhost:{port}");

    // Give the driver a moment to start accepting connections.
    wait_for_driver_ready(&url, Duration::from_secs(10)).await?;

    Ok(DriverProcess {
        kind: DriverKind::ChromeDriver,
        port,
        url,
        child,
    })
}

// ---------------------------------------------------------------------------
// Starting Appium
// ---------------------------------------------------------------------------

/// Starts an `appium` server process on a randomly chosen free port.
///
/// `binary_path` should point to the `appium` executable discovered via
/// [`crate::env::platform::find_appium`].
///
/// The server is ready to accept sessions once this function returns.
pub async fn start_appium(binary_path: &PathBuf) -> UtoResult<DriverProcess> {
    let port = pick_unused_port().ok_or(UtoError::NoFreePort)?;

    log::info!(
        "Starting Appium on port {} from {}",
        port,
        binary_path.display()
    );

    // `thirtyfour` 0.34 issues commands against root-relative endpoints such as
    // `/session`, so Appium must listen on the default root base path.
    let child = std::process::Command::new(binary_path)
        .args(["--port", &port.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .group_spawn()
        .map_err(|e| UtoError::DriverStartFailed(format!("{e}")))?;

    let url = format!("http://localhost:{port}");

    // Appium may take a few extra seconds to initialise its plugins.
    wait_for_driver_ready(&url, Duration::from_secs(30)).await?;

    Ok(DriverProcess {
        kind: DriverKind::Appium,
        port,
        url,
        child,
    })
}

// ---------------------------------------------------------------------------
// Readiness probe
// ---------------------------------------------------------------------------

/// Polls the driver's `/status` endpoint until it responds successfully or
/// the `timeout` elapses.
pub(crate) async fn wait_for_driver_ready(base_url: &str, timeout: Duration) -> UtoResult<()> {
    let status_url = format!("{base_url}/status");
    let deadline = tokio::time::Instant::now() + timeout;
    let client = reqwest::Client::new();

    loop {
        match client.get(&status_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                log::debug!("Driver at {base_url} is ready");
                return Ok(());
            }
            _ => {
                if tokio::time::Instant::now() >= deadline {
                    return Err(UtoError::DriverStartFailed(format!(
                        "Driver at {base_url} did not become ready within {:?}",
                        timeout
                    )));
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // -----------------------------------------------------------------------
    // wait_for_driver_ready — success
    // -----------------------------------------------------------------------

    /// When the `/status` endpoint returns HTTP 200 the helper must resolve
    /// successfully before the timeout expires.
    #[tokio::test]
    async fn wait_for_driver_ready_succeeds_when_server_responds_ok() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "value": { "ready": true, "message": "" }
            })))
            .mount(&mock_server)
            .await;

        wait_for_driver_ready(&mock_server.uri(), Duration::from_secs(5))
            .await
            .expect("should succeed when /status returns 200");
    }

    // -----------------------------------------------------------------------
    // wait_for_driver_ready — timeout
    // -----------------------------------------------------------------------

    /// When nothing listens on the target URL the helper must return a
    /// `DriverStartFailed` error before the timeout exceeds.
    #[tokio::test]
    async fn wait_for_driver_ready_times_out_when_no_server() {
        // Port 1 is reserved; nothing should be listening there.
        let result = wait_for_driver_ready("http://127.0.0.1:1", Duration::from_millis(300)).await;

        assert!(
            result.is_err(),
            "should fail when driver never becomes ready"
        );
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("did not become ready"),
            "unexpected error message: {msg}"
        );
    }

    // -----------------------------------------------------------------------
    // DriverProcess::stop — kills the child process
    // -----------------------------------------------------------------------

    /// We spawn a long-running process (Unix: `sleep 60`, Windows: `timeout 60`)
    /// and verify that `stop()` terminates it cleanly.
    #[cfg(unix)]
    #[test]
    fn driver_process_stop_terminates_child() {
        use command_group::CommandGroup;

        let child = std::process::Command::new("sleep")
            .arg("60")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .group_spawn()
            .expect("spawn sleep");

        let proc = DriverProcess {
            kind: DriverKind::ChromeDriver,
            port: 0,
            url: "http://localhost:0".to_string(),
            child,
        };

        proc.stop().expect("stop should not error");
    }
}
