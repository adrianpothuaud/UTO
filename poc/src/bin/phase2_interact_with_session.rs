//! # Phase 2 POC — Interact with Web and Mobile Sessions
//!
//! This script demonstrates the UTO communication layer for both the **web**
//! world (ChromeDriver + Chrome) and the **mobile** world (Appium + Android
//! emulator).  It assumes the relevant driver is already running, or can be
//! found/provisioned automatically.
//!
//! ## Usage
//!
//! ```sh
//! # Web demo (default)
//! cargo run -p uto-poc --bin phase2_interact_with_session
//!
//! # Mobile demo
//! UTO_DEMO=mobile cargo run -p uto-poc --bin phase2_interact_with_session
//! ```
//!
//! ## Prerequisites
//!
//! **Web demo:**
//! * Google Chrome installed on the host machine.
//!
//! **Mobile demo:**
//! * Appium installed globally (`npm install -g appium`).
//! * An Android emulator running (e.g. `emulator-5554`).
//! * `ANDROID_HOME` / `ANDROID_SDK_ROOT` set, or the SDK in a default path.

use uto_core::{
    driver,
    env::{
        mobile_setup::{prepare_mobile_environment, MobileSetupOptions},
        platform::find_chrome_version,
        provisioning::find_or_provision_chromedriver,
    },
    session::{
        mobile::{MobileCapabilities, MobileSession},
        web::WebSession,
        UtoSession,
    },
};

#[tokio::main]
async fn main() {
    let _ = uto_logger::init("phase2-poc");

    let demo = std::env::var("UTO_DEMO").unwrap_or_else(|_| "web".to_string());

    match demo.as_str() {
        "mobile" => run_mobile_demo().await,
        _ => run_web_demo().await,
    }
}

// ---------------------------------------------------------------------------
// Web demo
// ---------------------------------------------------------------------------

/// Demonstrates the web communication layer:
/// 1. Discover Chrome version.
/// 2. Provision (download if needed) the matching ChromeDriver.
/// 3. Start ChromeDriver.
/// 4. Open a session, navigate, read the title.
/// 5. Clean shutdown.
async fn run_web_demo() {
    log::info!("=== UTO Phase 2 — Web Session Demo ===");

    // Step 1 — discover Chrome.
    let chrome_version = match find_chrome_version() {
        Ok(v) => {
            log::info!("Found Chrome {v}");
            v
        }
        Err(e) => {
            log::error!("Chrome discovery failed: {e}");
            return;
        }
    };

    // Step 2 — provision ChromeDriver.
    let chromedriver_path = match find_or_provision_chromedriver(&chrome_version).await {
        Ok(p) => {
            log::info!("ChromeDriver ready at {}", p.display());
            p
        }
        Err(e) => {
            log::error!("ChromeDriver provisioning failed: {e}");
            return;
        }
    };

    // Step 3 — start ChromeDriver.
    let driver_proc = match driver::start_chromedriver(&chromedriver_path).await {
        Ok(p) => {
            log::info!("ChromeDriver running on port {}", p.port);
            p
        }
        Err(e) => {
            log::error!("Failed to start ChromeDriver: {e}");
            return;
        }
    };

    // Step 4 — create a web session and exercise the communication layer.
    match WebSession::new(&driver_proc.url).await {
        Ok(session) => {
            let result = web_interaction(&session).await;
            if let Err(e) = result {
                log::error!("Web interaction error: {e}");
            }
            if let Err(e) = Box::new(session).close().await {
                log::warn!("Session close error: {e}");
            }
        }
        Err(e) => {
            log::error!("Session creation failed: {e}");
        }
    }

    // Step 5 — clean shutdown.
    if let Err(e) = driver_proc.stop() {
        log::warn!("ChromeDriver stop error: {e}");
    }

    log::info!("Web demo complete.");
}

/// Exercises the UTO session API against a running Chrome browser.
async fn web_interaction(session: &WebSession) -> uto_core::error::UtoResult<()> {
    session.goto("https://example.com").await?;
    log::info!("Navigated to https://example.com");

    let title = session.title().await?;
    log::info!("Page title: '{title}'");

    let heading = session.find_element("h1").await?;
    let text = session.get_text(&heading).await?;
    log::info!("Heading text: '{text}'");

    let png = session.screenshot().await?;
    log::info!("Screenshot captured ({} bytes)", png.len());

    Ok(())
}

// ---------------------------------------------------------------------------
// Mobile demo
// ---------------------------------------------------------------------------

/// Demonstrates the mobile communication layer:
/// 1. Auto-prepare Android + Appium prerequisites.
/// 2. Start Appium.
/// 3. Open a mobile session with Android capabilities.
/// 4. Read the current activity / accessibility tree.
/// 5. Clean shutdown.
async fn run_mobile_demo() {
    log::info!("=== UTO Phase 2 — Mobile Session Demo ===");

    // Step 1 — auto-prepare Android + Appium runtime prerequisites.
    let setup_options = MobileSetupOptions {
        require_online_device: true,
        ..MobileSetupOptions::default()
    };

    let mobile_setup = match prepare_mobile_environment(&setup_options) {
        Ok(result) => {
            log::info!("Android SDK found at {}", result.android_sdk.root.display());
            log::info!("adb available at {}", result.android_sdk.adb_path.display());
            log::info!("Appium found at {}", result.appium_path.display());
            if let Some(serial) = &result.device_serial {
                log::info!("Android device/emulator online: {serial}");
            }
            for action in &result.actions {
                log::info!("[AUTO-FIX] {action}");
            }
            result
        }
        Err(e) => {
            log::error!("Mobile setup failed: {e}");
            return;
        }
    };

    // Step 2 — start Appium.
    let driver_proc = match driver::start_appium(&mobile_setup.appium_path).await {
        Ok(p) => {
            log::info!("Appium running on port {}", p.port);
            p
        }
        Err(e) => {
            log::error!("Failed to start Appium: {e}");
            return;
        }
    };

    // Step 3 — create a mobile session targeting the prepared Android device.
    let device_name = mobile_setup
        .device_serial
        .unwrap_or_else(|| "emulator-5554".to_string());

    let caps = MobileCapabilities::android(device_name);

    match MobileSession::new(&driver_proc.url, caps).await {
        Ok(session) => {
            let result = mobile_interaction(&session).await;
            if let Err(e) = result {
                log::error!("Mobile interaction error: {e}");
            }
            if let Err(e) = Box::new(session).close().await {
                log::warn!("Session close error: {e}");
            }
        }
        Err(e) => {
            log::error!("Mobile session creation failed: {e}");
        }
    }

    // Step 4 — clean shutdown.
    if let Err(e) = driver_proc.stop() {
        log::warn!("Appium stop error: {e}");
    }

    log::info!("Mobile demo complete.");
}

/// Exercises the UTO session API against a running Android emulator.
async fn mobile_interaction(session: &MobileSession) -> uto_core::error::UtoResult<()> {
    // Launch Android Settings.
    log::info!("Launching Android Settings...");
    session
        .launch_activity("com.android.settings", ".Settings")
        .await?;

    // Give the Settings app a moment to fully render.
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Take a screenshot of the Settings app.
    let png = session.screenshot().await?;
    log::info!("Screenshot captured ({} bytes)", png.len());

    // Capture the page source (accessibility tree) to show Settings structure.
    let source = session.page_source().await?;
    log::info!(
        "Page source (Settings accessibility tree): {} bytes",
        source.len()
    );

    // Lightweight assertion for demo confidence.
    let source_lower = source.to_ascii_lowercase();
    if source_lower.contains("settings") || source_lower.contains("com.android.settings") {
        log::info!("Settings launch confirmed via accessibility source");
    } else {
        log::warn!(
            "Settings launch not explicitly confirmed from source text; app may still be open"
        );
    }

    Ok(())
}
