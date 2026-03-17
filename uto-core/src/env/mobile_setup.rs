use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::error::{UtoError, UtoResult};

use super::platform::{find_android_sdk, find_appium, AndroidSdk};

/// Configuration for Android/Appium environment preparation.
#[derive(Debug, Clone)]
pub struct MobileSetupOptions {
    /// Optional preferred Android Virtual Device (AVD) name.
    pub preferred_avd: Option<String>,
    /// If true, attempt to boot an emulator when no device is connected.
    pub auto_start_emulator: bool,
    /// If true, return an error when no online Android device is available.
    pub require_online_device: bool,
    /// If true, install Appium via npm when not found in PATH.
    pub auto_install_appium: bool,
    /// If true, install the Appium UiAutomator2 driver when missing.
    pub auto_install_uiautomator2: bool,
    /// Maximum time to wait for an emulator/device to come online.
    pub emulator_boot_timeout: Duration,
}

impl Default for MobileSetupOptions {
    fn default() -> Self {
        Self {
            preferred_avd: None,
            auto_start_emulator: true,
            require_online_device: false,
            auto_install_appium: true,
            auto_install_uiautomator2: true,
            emulator_boot_timeout: Duration::from_secs(120),
        }
    }
}

/// The result of preparing Android/Appium prerequisites.
#[derive(Debug, Clone)]
pub struct MobileSetupResult {
    /// Discovered Android SDK location.
    pub android_sdk: AndroidSdk,
    /// Resolved Appium binary path.
    pub appium_path: PathBuf,
    /// Connected device serial (for example `emulator-5554`) if available.
    pub device_serial: Option<String>,
    /// Human-readable list of setup actions taken.
    pub actions: Vec<String>,
}

/// Prepares the Android/Appium environment using a discover-or-deploy strategy.
///
/// The flow is:
/// 1. Discover Android SDK and start `adb`.
/// 2. Ensure an online device (boot emulator if configured).
/// 3. Discover Appium or install it through npm.
/// 4. Ensure the UiAutomator2 Appium driver is installed.
pub fn prepare_mobile_environment(options: &MobileSetupOptions) -> UtoResult<MobileSetupResult> {
    let mut actions = Vec::new();

    let sdk = find_android_sdk().ok_or(UtoError::AndroidSdkNotFound)?;

    run_command(
        Command::new(&sdk.adb_path).arg("start-server"),
        "start adb server",
    )?;
    actions.push("Started adb server".to_string());

    let mut device_serial = first_online_device(&sdk.adb_path)?;

    if device_serial.is_none() && options.auto_start_emulator {
        match try_start_emulator_and_wait(&sdk, options) {
            Ok(Some(serial)) => {
                actions.push(format!(
                    "Android device online after emulator start: {serial}"
                ));
                device_serial = Some(serial);
            }
            Ok(None) => {}
            Err(e) if !options.require_online_device => {
                actions.push(format!("Emulator auto-start skipped: {e}"));
            }
            Err(e) => return Err(e),
        }
    }

    if options.require_online_device && device_serial.is_none() {
        return Err(UtoError::EnvironmentSetupFailed(
            "No online Android device/emulator found after setup. Start an emulator manually or connect a device with USB debugging enabled."
                .to_string(),
        ));
    }

    let appium_path = ensure_appium_available(options, &mut actions)?;

    if options.auto_install_uiautomator2 {
        ensure_uiautomator2_driver(&appium_path, &mut actions)?;
    }

    // Run driver health check and auto-repair if needed.
    ensure_appium_drivers_healthy(&appium_path, &mut actions)?;

    Ok(MobileSetupResult {
        android_sdk: sdk,
        appium_path,
        device_serial,
        actions,
    })
}

fn ensure_appium_available(
    options: &MobileSetupOptions,
    actions: &mut Vec<String>,
) -> UtoResult<PathBuf> {
    if let Some(path) = find_appium() {
        return Ok(path);
    }

    if !options.auto_install_appium {
        return Err(UtoError::AppiumNotFound);
    }

    let npm_path = which::which("npm").map_err(|_| {
        UtoError::EnvironmentSetupFailed(
            "Appium is not installed and npm was not found in PATH. Install Node.js/npm, then rerun setup."
                .to_string(),
        )
    })?;

    run_command(
        Command::new(npm_path).args(["install", "-g", "appium"]),
        "install appium via npm",
    )?;
    actions.push("Installed Appium via npm".to_string());

    find_appium().ok_or(UtoError::AppiumNotFound)
}

fn ensure_uiautomator2_driver(appium_path: &Path, actions: &mut Vec<String>) -> UtoResult<()> {
    let installed_drivers = list_installed_appium_drivers(appium_path)?;

    if let Some(driver) = installed_drivers.get("uiautomator2") {
        actions.push(format!(
            "UiAutomator2 driver verified: {}{}",
            driver.automation_name.as_deref().unwrap_or("uiautomator2"),
            driver
                .version
                .as_deref()
                .map(|version| format!(" {version}"))
                .unwrap_or_default()
        ));
        return Ok(());
    }

    run_command(
        Command::new(appium_path).args(["driver", "install", "uiautomator2"]),
        "install appium uiautomator2 driver",
    )?;
    actions.push("Installed Appium UiAutomator2 driver".to_string());

    if !appium_driver_is_installed(appium_path, "uiautomator2")? {
        return Err(UtoError::EnvironmentSetupFailed(
            "UiAutomator2 driver install command completed but driver is still missing."
                .to_string(),
        ));
    }

    Ok(())
}

fn appium_driver_is_installed(appium_path: &Path, driver_name: &str) -> UtoResult<bool> {
    let out = run_command_capture(
        Command::new(appium_path).args(["driver", "list", "--installed"]),
        "list installed appium drivers",
    )?;
    let text = combined_output(&out).to_ascii_lowercase();
    Ok(text.contains(&driver_name.to_ascii_lowercase()))
}

/// Performs a comprehensive Appium driver health check and auto-repair.
///
/// This function:
/// 1. Loads the Appium 3 driver inventory via JSON output.
/// 2. Ensures the UiAutomator2 driver is installed.
/// 3. Applies a safe `appium driver update uiautomator2` when Appium reports one.
/// 4. Runs `appium driver doctor uiautomator2` and fails only on required issues.
///
/// This is cross-platform and works on macOS, Linux, and Windows.
fn ensure_appium_drivers_healthy(appium_path: &Path, actions: &mut Vec<String>) -> UtoResult<()> {
    let mut installed_drivers = list_installed_appium_drivers(appium_path)?;

    let Some(driver) = installed_drivers.get("uiautomator2").cloned() else {
        run_command(
            Command::new(appium_path).args(["driver", "install", "uiautomator2"]),
            "install appium uiautomator2 driver",
        )?;
        actions.push("Installed Appium UiAutomator2 driver".to_string());
        installed_drivers = list_installed_appium_drivers(appium_path)?;
        let Some(driver) = installed_drivers.get("uiautomator2") else {
            return Err(UtoError::EnvironmentSetupFailed(
                "UiAutomator2 driver install completed but Appium still does not report it as installed."
                    .to_string(),
            ));
        };
        log::info!(
            "UiAutomator2 driver installed: version={:?}, appium_range={:?}",
            driver.version,
            driver.appium_version
        );
        run_uiautomator2_doctor(appium_path, actions)?;
        return Ok(());
    };

    log::debug!(
        "UiAutomator2 driver inventory: version={:?}, update={:?}, unsafe_update={:?}, appium_range={:?}",
        driver.version,
        driver.update_version,
        driver.unsafe_update_version,
        driver.appium_version
    );

    if let Some(update_version) = &driver.update_version {
        run_command(
            Command::new(appium_path).args(["driver", "update", "uiautomator2"]),
            "update appium uiautomator2 driver",
        )?;
        actions.push(format!(
            "Updated Appium UiAutomator2 driver to {update_version}"
        ));
    } else if !driver.up_to_date && driver.unsafe_update_version.is_some() {
        actions.push(format!(
            "UiAutomator2 driver {} has only a major update available; keeping the installed version for compatibility",
            driver.version.as_deref().unwrap_or("unknown")
        ));
    }

    run_uiautomator2_doctor(appium_path, actions)?;

    Ok(())
}

fn run_uiautomator2_doctor(appium_path: &Path, actions: &mut Vec<String>) -> UtoResult<()> {
    let out = run_command_capture(
        Command::new(appium_path).args(["driver", "doctor", "uiautomator2"]),
        "run appium uiautomator2 doctor",
    )?;

    let doctor_output = combined_output(&out);
    let summary = parse_doctor_summary(&doctor_output).ok_or_else(|| {
        UtoError::EnvironmentSetupFailed(format!(
            "Appium UiAutomator2 doctor ran, but its summary could not be parsed: {doctor_output}"
        ))
    })?;

    if summary.required_fixes > 0 {
        return Err(UtoError::EnvironmentSetupFailed(format!(
            "Appium UiAutomator2 doctor reported {} required fix(es): {doctor_output}",
            summary.required_fixes
        )));
    }

    actions.push(format!(
        "UiAutomator2 doctor passed with {} required fix(es) and {} optional fix(es)",
        summary.required_fixes, summary.optional_fixes
    ));

    Ok(())
}

fn list_installed_appium_drivers(appium_path: &Path) -> UtoResult<HashMap<String, AppiumDriver>> {
    let out = run_command_capture(
        Command::new(appium_path).args(["driver", "list", "--installed", "--updates", "--json"]),
        "list installed appium drivers as json",
    )?;

    parse_installed_drivers_json(&combined_output(&out))
}

/// Represents an installed Appium driver with its metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AppiumDriver {
    /// Driver version (e.g., "6.9.3"), if detectable.
    version: Option<String>,
    /// Human-facing automation name, e.g. `UiAutomator2`.
    automation_name: Option<String>,
    /// Whether Appium reports the driver as installed.
    installed: bool,
    /// Safe update version, when available.
    update_version: Option<String>,
    /// Major or otherwise unsafe update version, when available.
    unsafe_update_version: Option<String>,
    /// Whether the driver is fully up to date.
    up_to_date: bool,
    /// Declared Appium version range for this driver.
    appium_version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DriverDoctorSummary {
    required_fixes: u32,
    optional_fixes: u32,
}

fn parse_installed_drivers_json(output: &str) -> UtoResult<HashMap<String, AppiumDriver>> {
    let drivers: HashMap<String, AppiumDriver> = serde_json::from_str(output).map_err(|e| {
        UtoError::EnvironmentSetupFailed(format!(
            "Failed to parse Appium driver inventory JSON: {e}. Output: {output}"
        ))
    })?;

    Ok(drivers
        .into_iter()
        .filter(|(_, driver)| driver.installed)
        .collect())
}

fn parse_doctor_summary(output: &str) -> Option<DriverDoctorSummary> {
    let summary_line = output
        .lines()
        .find(|line| line.contains("Diagnostic completed,"))?;

    let required_fixes = summary_line
        .split("Diagnostic completed,")
        .nth(1)?
        .split("required fixes needed")
        .next()?
        .split_whitespace()
        .last()?
        .parse()
        .ok()?;

    let optional_fixes = summary_line
        .split("required fixes needed,")
        .nth(1)?
        .split("optional fixes possible")
        .next()?
        .split_whitespace()
        .last()?
        .parse()
        .ok()?;

    Some(DriverDoctorSummary {
        required_fixes,
        optional_fixes,
    })
}

fn choose_avd_name(sdk: &AndroidSdk, preferred: Option<&str>) -> UtoResult<Option<String>> {
    let Some(emulator_bin) = emulator_binary_path(sdk) else {
        return Ok(None);
    };

    let out = run_command_capture(
        Command::new(emulator_bin).arg("-list-avds"),
        "list Android AVDs",
    )?;

    let avds = parse_avd_list(&String::from_utf8_lossy(&out.stdout));
    if avds.is_empty() {
        return Ok(None);
    }

    if let Some(preferred_name) = preferred {
        if avds.iter().any(|a| a == preferred_name) {
            return Ok(Some(preferred_name.to_string()));
        }
    }

    Ok(Some(avds[0].clone()))
}

fn try_start_emulator_and_wait(
    sdk: &AndroidSdk,
    options: &MobileSetupOptions,
) -> UtoResult<Option<String>> {
    let Some(avd_name) = choose_avd_name(sdk, options.preferred_avd.as_deref())? else {
        return Ok(None);
    };

    start_emulator(sdk, &avd_name)?;
    wait_for_online_device(&sdk.adb_path, options.emulator_boot_timeout)
}

fn start_emulator(sdk: &AndroidSdk, avd_name: &str) -> UtoResult<()> {
    let emulator_bin = emulator_binary_path(sdk)
        .ok_or_else(|| UtoError::BinaryNotFound("Android emulator binary".to_string()))?;

    Command::new(emulator_bin)
        .args(["-avd", avd_name, "-no-snapshot-save"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| {
            UtoError::EnvironmentSetupFailed(format!("Failed to start emulator '{avd_name}': {e}"))
        })?;

    Ok(())
}

fn wait_for_online_device(adb_path: &Path, timeout: Duration) -> UtoResult<Option<String>> {
    let deadline = Instant::now() + timeout;

    loop {
        if let Some(serial) = first_online_device(adb_path)? {
            return Ok(Some(serial));
        }

        if Instant::now() >= deadline {
            return Ok(None);
        }

        thread::sleep(Duration::from_millis(1000));
    }
}

fn first_online_device(adb_path: &Path) -> UtoResult<Option<String>> {
    let out = run_command_capture(Command::new(adb_path).arg("devices"), "list adb devices")?;
    let devices = parse_adb_devices(&String::from_utf8_lossy(&out.stdout));
    Ok(devices
        .iter()
        .find(|(_, state)| state == "device")
        .map(|(serial, _)| serial.clone()))
}

fn emulator_binary_path(sdk: &AndroidSdk) -> Option<PathBuf> {
    let binary = if cfg!(target_os = "windows") {
        "emulator.exe"
    } else {
        "emulator"
    };

    let path = sdk.root.join("emulator").join(binary);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn run_command(cmd: &mut Command, label: &str) -> UtoResult<()> {
    let out = run_command_capture(cmd, label)?;
    if out.status.success() {
        Ok(())
    } else {
        Err(UtoError::EnvironmentSetupFailed(format!(
            "Failed to {label}: {}",
            combined_output(&out)
        )))
    }
}

fn run_command_capture(cmd: &mut Command, label: &str) -> UtoResult<Output> {
    cmd.output()
        .map_err(|e| UtoError::EnvironmentSetupFailed(format!("Failed to {label}: {e}")))
}

fn combined_output(out: &Output) -> String {
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();

    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => "no command output".to_string(),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("{stdout} | {stderr}"),
    }
}

fn parse_adb_devices(output: &str) -> Vec<(String, String)> {
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }

            let mut parts = trimmed.split_whitespace();
            let serial = parts.next()?.to_string();
            let state = parts.next()?.to_string();
            Some((serial, state))
        })
        .collect()
}

fn parse_avd_list(output: &str) -> Vec<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        parse_adb_devices, parse_avd_list, parse_doctor_summary, parse_installed_drivers_json,
    };

    #[test]
    fn parse_adb_devices_extracts_serial_and_state() {
        let out = "List of devices attached\nemulator-5554\tdevice\n0123456789ABCDEF\toffline\n";
        let devices = parse_adb_devices(out);

        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].0, "emulator-5554");
        assert_eq!(devices[0].1, "device");
        assert_eq!(devices[1].0, "0123456789ABCDEF");
        assert_eq!(devices[1].1, "offline");
    }

    #[test]
    fn parse_avd_list_returns_non_empty_lines() {
        let out = "Pixel_7_API_34\n\nsmall_phone_api_31\n";
        let avds = parse_avd_list(out);

        assert_eq!(avds, vec!["Pixel_7_API_34", "small_phone_api_31"]);
    }

    #[test]
    fn parse_installed_drivers_json_extracts_metadata() {
        let output = r#"{
    "uiautomator2": {
        "version": "6.9.3",
        "automationName": "UiAutomator2",
        "installed": true,
        "updateVersion": null,
        "unsafeUpdateVersion": "7.0.0",
        "upToDate": false,
        "appiumVersion": "^3.0.0-rc.2"
    },
    "xcuitest": {
        "version": "10.32.1",
        "automationName": "XCUITest",
        "installed": true,
        "updateVersion": null,
        "unsafeUpdateVersion": null,
        "upToDate": true,
        "appiumVersion": "^3.0.0-rc.2"
    }
}"#;
        let drivers = parse_installed_drivers_json(output).expect("json should parse");

        assert_eq!(drivers.len(), 2);
        assert_eq!(drivers["uiautomator2"].version.as_deref(), Some("6.9.3"));
        assert_eq!(
            drivers["uiautomator2"].automation_name.as_deref(),
            Some("UiAutomator2")
        );
        assert!(!drivers["uiautomator2"].up_to_date);
        assert_eq!(
            drivers["uiautomator2"].unsafe_update_version.as_deref(),
            Some("7.0.0")
        );
    }

    #[test]
    fn parse_installed_drivers_json_filters_non_installed_entries() {
        let output = r#"{
    "uiautomator2": {
        "version": "6.9.3",
        "automationName": "UiAutomator2",
        "installed": true,
        "updateVersion": null,
        "unsafeUpdateVersion": null,
        "upToDate": true,
        "appiumVersion": "^3.0.0-rc.2"
    },
    "mac2": {
        "version": "1.0.0",
        "automationName": "Mac2",
        "installed": false,
        "updateVersion": null,
        "unsafeUpdateVersion": null,
        "upToDate": false,
        "appiumVersion": "^3.0.0-rc.2"
    }
}"#;
        let drivers = parse_installed_drivers_json(output).expect("json should parse");

        assert_eq!(drivers.len(), 1);
        assert!(drivers.contains_key("uiautomator2"));
        assert!(!drivers.contains_key("mac2"));
    }

    #[test]
    fn parse_doctor_summary_extracts_required_and_optional_fix_counts() {
        let output = "info Doctor ### Diagnostic completed, 0 required fixes needed, 2 optional fixes possible. ###";
        let summary = parse_doctor_summary(output).expect("summary should parse");

        assert_eq!(summary.required_fixes, 0);
        assert_eq!(summary.optional_fixes, 2);
    }
}
