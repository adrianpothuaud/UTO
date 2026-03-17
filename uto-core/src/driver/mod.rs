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
        self.child.kill().map_err(|e| {
            UtoError::DriverStopFailed(format!("{e}"))
        })?;
        self.child.wait().map_err(|e| {
            UtoError::DriverStopFailed(format!("{e}"))
        })?;
        log::info!("{:?} stopped (port {})", self.kind, self.port);
        Ok(())
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
pub async fn start_chromedriver(binary_path: &PathBuf) -> UtoResult<DriverProcess> {
    let port = pick_unused_port().ok_or(UtoError::NoFreePort)?;
    let port_arg = format!("--port={port}");

    log::info!(
        "Starting chromedriver on port {} from {}",
        port,
        binary_path.display()
    );

    let child = std::process::Command::new(binary_path)
        .arg(&port_arg)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
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

    let child = std::process::Command::new(binary_path)
        .args(["--port", &port.to_string(), "--base-path", "/wd/hub"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .group_spawn()
        .map_err(|e| UtoError::DriverStartFailed(format!("{e}")))?;

    let url = format!("http://localhost:{port}/wd/hub");

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
async fn wait_for_driver_ready(base_url: &str, timeout: Duration) -> UtoResult<()> {
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
