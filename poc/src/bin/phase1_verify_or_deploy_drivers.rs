//! # Phase 1 POC — Verify or Deploy Drivers for Web and Mobile
//!
//! This script validates the **Zero-Config Infrastructure** layer of UTO.
//! It checks whether the required drivers are already available on the
//! system; if not, it provisions them automatically.
//!
//! ## Actions performed
//!
//! ### Web
//! 1. Detect the installed Chrome version.
//! 2. Look for a matching ChromeDriver in the local cache (`.uto/cache/`).
//! 3. If not found, download and install the correct ChromeDriver from the
//!    Chrome for Testing JSON API.
//!
//! ### Mobile
//! 1. Locate the Android SDK via `ANDROID_HOME` / `ANDROID_SDK_ROOT` or
//!    common default paths.
//! 2. Verify that `adb` is reachable inside the SDK.
//! 3. Check whether `appium` is available in `PATH`.
//! 4. If Appium is missing, print actionable installation instructions.
//!
//! ## Usage
//!
//! ```sh
//! cargo run -p uto-poc --bin phase1_verify_or_deploy_drivers
//! ```

use std::process;

use uto_core::env::{
    platform::{find_android_sdk, find_appium, find_chrome_version},
    provisioning::find_or_provision_chromedriver,
};

#[tokio::main]
async fn main() {
    // INFO by default, configurable via RUST_LOG.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("=== UTO Phase 1 — Verify / Deploy Drivers ===");

    let mut all_ok = true;

    // ------------------------------------------------------------------
    // Web: Chrome + ChromeDriver
    // ------------------------------------------------------------------
    log::info!("--- Web: Chrome ---");
    let chrome_version = match find_chrome_version() {
        Ok(v) => {
            log::info!("[OK] Chrome detected: version {v}");
            Some(v)
        }
        Err(e) => {
            log::error!("[FAIL] Chrome not found: {e}");
            all_ok = false;
            None
        }
    };

    if let Some(version) = chrome_version {
        log::info!("--- Web: ChromeDriver ---");
        match find_or_provision_chromedriver(&version).await {
            Ok(path) => {
                log::info!("[OK] ChromeDriver ready at {}", path.display());
            }
            Err(e) => {
                log::error!("[FAIL] ChromeDriver provisioning failed: {e}");
                all_ok = false;
            }
        }
    }

    // ------------------------------------------------------------------
    // Mobile: Android SDK + adb
    // ------------------------------------------------------------------
    log::info!("--- Mobile: Android SDK ---");
    match find_android_sdk() {
        Some(sdk) => {
            log::info!("[OK] Android SDK found at {}", sdk.root.display());
            log::info!("[OK] adb available at {}", sdk.adb_path.display());
        }
        None => {
            log::warn!(
                "[WARN] Android SDK not found. \
                 Set the ANDROID_HOME environment variable or install the SDK. \
                 Mobile automation will not work without it."
            );
        }
    }

    // ------------------------------------------------------------------
    // Mobile: Appium
    // ------------------------------------------------------------------
    log::info!("--- Mobile: Appium ---");
    match find_appium() {
        Some(path) => {
            log::info!("[OK] Appium found at {}", path.display());
        }
        None => {
            log::warn!(
                "[WARN] Appium not found in PATH. \
                 Install it with: npm install -g appium\n\
                 Then add the UiAutomator2 driver: appium driver install uiautomator2\n\
                 (For iOS: appium driver install xcuitest)"
            );
        }
    }

    // ------------------------------------------------------------------
    // Summary
    // ------------------------------------------------------------------
    log::info!("=== Summary ===");
    if all_ok {
        log::info!("All required drivers are available. Ready to run Phase 2.");
    } else {
        log::error!(
            "One or more required drivers are missing. \
             Please review the messages above and fix the issues before running Phase 2."
        );
        process::exit(1);
    }
}
