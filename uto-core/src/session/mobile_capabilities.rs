use serde_json::{json, Value};

/// The target mobile operating system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MobilePlatform {
    /// Android device or emulator controlled via UiAutomator2.
    Android,
    /// iOS device or simulator controlled via XCUITest.
    Ios,
}

impl MobilePlatform {
    /// Returns the W3C `platformName` string expected by Appium.
    pub fn platform_name(&self) -> &'static str {
        match self {
            MobilePlatform::Android => "Android",
            MobilePlatform::Ios => "iOS",
        }
    }

    /// Returns the default Appium `automationName` for this platform.
    pub fn automation_name(&self) -> &'static str {
        match self {
            MobilePlatform::Android => "UiAutomator2",
            MobilePlatform::Ios => "XCUITest",
        }
    }
}

/// Desired capabilities for an Appium mobile session.
///
/// These map to the Appium W3C capabilities and are serialised into the
/// session-creation request.
#[derive(Debug, Clone)]
pub struct MobileCapabilities {
    /// The target mobile platform.
    pub platform: MobilePlatform,
    /// The target device name or UDID (e.g. `"emulator-5554"`, `"iPhone 14"`).
    pub device_name: String,
    /// Absolute path to the app under test. Leave empty to target the device's
    /// default browser.
    pub app_path: Option<String>,
    /// Platform version string (e.g. `"13.0"`, `"16.4"`).
    pub platform_version: Option<String>,
    /// Custom extra capabilities forwarded verbatim to Appium.
    pub extra: serde_json::Map<String, Value>,
}

impl MobileCapabilities {
    /// Creates a minimal Android capability set targeting `device_name`.
    pub fn android(device_name: impl Into<String>) -> Self {
        Self {
            platform: MobilePlatform::Android,
            device_name: device_name.into(),
            app_path: None,
            platform_version: None,
            extra: serde_json::Map::new(),
        }
    }

    /// Creates a minimal iOS capability set targeting `device_name`.
    pub fn ios(device_name: impl Into<String>) -> Self {
        Self {
            platform: MobilePlatform::Ios,
            device_name: device_name.into(),
            app_path: None,
            platform_version: None,
            extra: serde_json::Map::new(),
        }
    }

    /// Sets the app path.
    pub fn with_app(mut self, path: impl Into<String>) -> Self {
        self.app_path = Some(path.into());
        self
    }

    /// Sets the platform version.
    pub fn with_platform_version(mut self, version: impl Into<String>) -> Self {
        self.platform_version = Some(version.into());
        self
    }

    /// Adds a custom extra capability.
    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    /// Builds the Appium W3C `alwaysMatch` capability object.
    pub(crate) fn to_json(&self) -> Value {
        let mut caps = json!({
            "platformName": self.platform.platform_name(),
            "appium:deviceName": self.device_name,
            "appium:automationName": self.platform.automation_name(),
        });

        if let Some(ver) = &self.platform_version {
            caps["appium:platformVersion"] = json!(ver);
        }
        if let Some(app) = &self.app_path {
            caps["appium:app"] = json!(app);
        }
        for (k, v) in &self.extra {
            caps[k] = v.clone();
        }

        caps
    }

    /// Public version of [`to_json`] intended for tests that live outside
    /// this crate (e.g. in `uto-core/tests/`).
    pub fn to_json_pub(&self) -> Value {
        self.to_json()
    }
}
