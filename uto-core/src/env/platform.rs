use std::path::PathBuf;
use std::process::Command;

use crate::error::{UtoError, UtoResult};

// ---------------------------------------------------------------------------
// Chrome
// ---------------------------------------------------------------------------

/// Returns the installed Google Chrome version string (e.g. `"124.0.6367.60"`).
///
/// The function tries several well-known binary locations and falls back to
/// querying the binary with `--version`.
pub fn find_chrome_version() -> UtoResult<String> {
    let candidates = chrome_binary_candidates();

    for candidate in &candidates {
        if let Some(version) = query_chrome_version(candidate) {
            return Ok(version);
        }
    }

    Err(UtoError::BrowserNotFound(
        "Google Chrome not found. Install Chrome and try again.".to_string(),
    ))
}

/// Returns candidate paths for the Chrome binary depending on the current OS.
fn chrome_binary_candidates() -> Vec<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        vec![
            PathBuf::from(
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            ),
            PathBuf::from(
                "/Applications/Chromium.app/Contents/MacOS/Chromium",
            ),
        ]
    }

    #[cfg(target_os = "linux")]
    {
        vec![
            PathBuf::from("/usr/bin/google-chrome"),
            PathBuf::from("/usr/bin/google-chrome-stable"),
            PathBuf::from("/usr/bin/chromium"),
            PathBuf::from("/usr/bin/chromium-browser"),
            PathBuf::from("/snap/bin/chromium"),
        ]
    }

    #[cfg(target_os = "windows")]
    {
        let mut paths = Vec::new();
        for base in &[
            std::env::var("PROGRAMFILES").unwrap_or_default(),
            std::env::var("PROGRAMFILES(X86)").unwrap_or_default(),
            std::env::var("LOCALAPPDATA").unwrap_or_default(),
        ] {
            if !base.is_empty() {
                paths.push(PathBuf::from(base).join("Google\\Chrome\\Application\\chrome.exe"));
            }
        }
        paths
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        vec![]
    }
}

/// Invokes `<binary> --version` and parses the version number from the output.
fn query_chrome_version(binary: &PathBuf) -> Option<String> {
    if !binary.exists() {
        return None;
    }

    let output = Command::new(binary).arg("--version").output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Expected format: "Google Chrome 124.0.6367.60" or "Chromium 124.0.6367.60"
    parse_version_from_output(&stdout)
}

/// Extracts the dotted version number from a `"<name> X.Y.Z.W"` string.
fn parse_version_from_output(output: &str) -> Option<String> {
    output
        .split_whitespace()
        .find(|token| token.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
        .map(|v| v.trim().to_string())
}

// ---------------------------------------------------------------------------
// Android SDK
// ---------------------------------------------------------------------------

/// Represents a discovered Android SDK installation.
#[derive(Debug, Clone)]
pub struct AndroidSdk {
    /// Root directory of the Android SDK (value of `ANDROID_HOME` / `ANDROID_SDK_ROOT`).
    pub root: PathBuf,
    /// Path to the `adb` binary inside the SDK.
    pub adb_path: PathBuf,
}

/// Discovers an Android SDK installation.
///
/// Search order:
/// 1. `ANDROID_HOME` environment variable
/// 2. `ANDROID_SDK_ROOT` environment variable
/// 3. Common platform-specific default locations
///
/// Returns `None` if no SDK with a valid `adb` binary is found.
pub fn find_android_sdk() -> Option<AndroidSdk> {
    let candidates = android_sdk_candidates();

    for root in candidates {
        let adb = root.join("platform-tools").join(adb_binary_name());
        if adb.exists() {
            return Some(AndroidSdk { root, adb_path: adb });
        }
    }
    None
}

/// Returns candidate root directories for the Android SDK.
fn android_sdk_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // Prefer explicit environment variables
    for var in &["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Ok(path) = std::env::var(var) {
            candidates.push(PathBuf::from(path));
        }
    }

    // Common default locations
    #[cfg(target_os = "macos")]
    candidates.push(
        dirs::home_dir()
            .unwrap_or_default()
            .join("Library/Android/sdk"),
    );

    #[cfg(target_os = "linux")]
    candidates.push(
        dirs::home_dir()
            .unwrap_or_default()
            .join("Android/Sdk"),
    );

    #[cfg(target_os = "windows")]
    candidates.push(
        dirs::data_local_dir()
            .unwrap_or_default()
            .join("Android\\Sdk"),
    );

    candidates
}

/// Returns the platform-specific name of the `adb` binary.
fn adb_binary_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "adb.exe"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "adb"
    }
}

// ---------------------------------------------------------------------------
// Appium
// ---------------------------------------------------------------------------

/// Returns the path to the `appium` binary if it is available in PATH.
pub fn find_appium() -> Option<PathBuf> {
    which::which("appium").ok()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_version_number_from_chrome_output() {
        let line = "Google Chrome 124.0.6367.60";
        assert_eq!(
            parse_version_from_output(line),
            Some("124.0.6367.60".to_string())
        );
    }

    #[test]
    fn parse_version_number_from_chromium_output() {
        let line = "Chromium 120.0.6099.109 built on Debian 12, running on Debian 12";
        assert_eq!(
            parse_version_from_output(line),
            Some("120.0.6099.109".to_string())
        );
    }

    #[test]
    fn parse_version_returns_none_for_empty_string() {
        assert_eq!(parse_version_from_output(""), None);
    }
}
