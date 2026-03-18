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
//! 3. Start `adb` and ensure at least one device is discoverable.
//! 4. Check whether `appium` is available in `PATH`; install it via npm if missing.
//! 5. Ensure the Appium UiAutomator2 driver is installed.
//!
//! ## Usage
//!
//! ```sh
//! cargo run -p uto-poc --bin phase1_verify_or_deploy_drivers
//! ```

use std::process;

use uto_core::env::{
    mobile_setup::{prepare_mobile_environment, MobileSetupOptions},
    platform::find_chrome_version,
    provisioning::find_or_provision_chromedriver,
};

#[tokio::main]
async fn main() {
    let _ = uto_logger::init("phase1-poc");

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
    // Mobile: Android + Appium auto-setup
    // ------------------------------------------------------------------
    log::info!("--- Mobile: Android/Appium Setup ---");
    let setup_options = MobileSetupOptions {
        // We validate setup here but do not require a booted device in phase 1.
        require_online_device: false,
        ..MobileSetupOptions::default()
    };

    match prepare_mobile_environment(&setup_options) {
        Ok(result) => {
            log::info!(
                "[OK] Android SDK found at {}",
                result.android_sdk.root.display()
            );
            log::info!(
                "[OK] adb available at {}",
                result.android_sdk.adb_path.display()
            );
            log::info!("[OK] Appium found at {}", result.appium_path.display());
            if let Some(serial) = result.device_serial {
                log::info!("[OK] Android device/emulator online: {serial}");
            } else {
                log::warn!("[WARN] No Android device/emulator online yet.");
            }
            for action in result.actions {
                log::info!("[AUTO-FIX] {action}");
            }
        }
        Err(e) => {
            log::warn!("[WARN] Mobile setup could not be fully prepared: {e}");
            all_ok = false;
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
