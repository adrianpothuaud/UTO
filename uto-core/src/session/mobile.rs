use async_trait::async_trait;
use serde_json::{json, Value};
use thirtyfour::{By, Capabilities, WebDriver};

use crate::error::{UtoError, UtoResult};

use super::{ElementHandle, UtoElement, UtoSession};

// ---------------------------------------------------------------------------
// Mobile platform
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// MobileCapabilities
// ---------------------------------------------------------------------------

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
}

// ---------------------------------------------------------------------------
// MobileSession
// ---------------------------------------------------------------------------

/// A UTO session that communicates with an **Appium** server via the W3C
/// WebDriver protocol.
///
/// Appium speaks the same W3C WebDriver protocol as ChromeDriver, so `thirtyfour`
/// is used as the underlying transport.  The key difference lies in the
/// capabilities: instead of `browserName`, Appium expects
/// `platformName`, `appium:deviceName`, and `appium:automationName`.
///
/// # Example
///
/// ```rust,no_run
/// # use uto_core::session::mobile::{MobileCapabilities, MobileSession};
/// # use uto_core::session::UtoSession;
/// # #[tokio::main]
/// # async fn main() -> uto_core::error::UtoResult<()> {
/// let caps = MobileCapabilities::android("emulator-5554")
///     .with_app("/path/to/app.apk");
/// let session = MobileSession::new("http://localhost:4723/wd/hub", caps).await?;
/// let title = session.title().await?;
/// println!("Activity: {title}");
/// Box::new(session).close().await?;
/// # Ok(())
/// # }
/// ```
pub struct MobileSession {
    driver: WebDriver,
}

impl MobileSession {
    /// Creates a new Appium mobile session by connecting to the server at
    /// `appium_url` with the given `capabilities`.
    ///
    /// Appium must already be running. Use [`crate::driver::start_appium`] to
    /// start a managed Appium process.
    pub async fn new(appium_url: &str, capabilities: MobileCapabilities) -> UtoResult<Self> {
        // Build the W3C alwaysMatch capability payload.
        let caps_json = capabilities.to_json();

        // `Capabilities` in thirtyfour is `serde_json::Map<String, Value>`.
        // Convert the JSON object directly; use an empty map if conversion fails.
        let caps: Capabilities = caps_json
            .as_object()
            .cloned()
            .unwrap_or_default();

        let driver: WebDriver = WebDriver::new(appium_url, caps).await.map_err(|e| {
            UtoError::SessionCreationFailed(format!("Appium at {appium_url}: {e}"))
        })?;

        log::info!(
            "Mobile session created ({} device '{}' via Appium at {appium_url})",
            capabilities.platform.platform_name(),
            capabilities.device_name
        );
        Ok(Self { driver })
    }

    // -------------------------------------------------------------------
    // Mobile-specific extensions
    // -------------------------------------------------------------------

    /// Performs a swipe gesture from `(start_x, start_y)` to
    /// `(end_x, end_y)`.
    ///
    /// `duration_ms` is reserved for future use. The current implementation
    /// uses `thirtyfour`'s `ActionChain` which does not expose per-move
    /// duration; a raw W3C Actions JSON implementation will honour this
    /// parameter in a future release.
    ///
    /// Coordinates are in device-independent pixels.
    pub async fn swipe(
        &self,
        start_x: i64,
        start_y: i64,
        end_x: i64,
        end_y: i64,
        _duration_ms: u64,
    ) -> UtoResult<()> {
        // Use the W3C Actions API via thirtyfour's ActionChain.
        // Appium maps pointer (mouse) actions to touch events on mobile.
        self.driver
            .action_chain()
            .move_to(start_x, start_y)
            .click_and_hold()
            .move_to(end_x, end_y)
            .release()
            .perform()
            .await
            .map_err(|e| {
                UtoError::SessionCommandFailed(format!("swipe(): {e}"))
            })?;

        Ok(())
    }

    /// Taps the screen at the given coordinates.
    pub async fn tap(&self, x: i64, y: i64) -> UtoResult<()> {
        // Use the W3C Actions API via thirtyfour's ActionChain.
        self.driver
            .action_chain()
            .move_to(x, y)
            .click()
            .perform()
            .await
            .map_err(|e| {
                UtoError::SessionCommandFailed(format!("tap({x}, {y}): {e}"))
            })?;

        Ok(())
    }

    /// Returns the page source (XML accessibility tree dump from Appium).
    pub async fn page_source(&self) -> UtoResult<String> {
        self.driver.source().await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("page_source(): {e}"))
        })
    }
}

// ---------------------------------------------------------------------------
// UtoSession implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl UtoSession for MobileSession {
    async fn goto(&self, url: &str) -> UtoResult<()> {
        // For native apps this sends the deep-link via an Appium command.
        self.driver.goto(url).await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("goto({url}): {e}"))
        })
    }

    async fn title(&self) -> UtoResult<String> {
        // Returns the current activity / view controller name.
        self.driver.title().await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("title(): {e}"))
        })
    }

    async fn find_element(&self, selector: &str) -> UtoResult<UtoElement> {
        // Appium supports XPath, accessibility ID, and other locator strategies.
        // We default to XPath which works across both Android and iOS.
        let elem = self
            .driver
            .find(By::XPath(selector))
            .await
            .map_err(|e| {
                UtoError::SessionCommandFailed(format!("find_element({selector}): {e}"))
            })?;

        let label = elem.text().await.unwrap_or_default();

        Ok(UtoElement {
            label,
            selector: selector.to_string(),
            handle: ElementHandle::Mobile(elem),
        })
    }

    async fn click(&self, element: &UtoElement) -> UtoResult<()> {
        let elem = match &element.handle {
            ElementHandle::Mobile(e) => e,
            ElementHandle::Web(_) => {
                return Err(UtoError::SessionCommandFailed(
                    "click(): web element passed to mobile session".to_string(),
                ))
            }
        };
        elem.click().await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("click({}): {e}", element.selector))
        })
    }

    async fn type_text(&self, element: &UtoElement, text: &str) -> UtoResult<()> {
        let elem = match &element.handle {
            ElementHandle::Mobile(e) => e,
            ElementHandle::Web(_) => {
                return Err(UtoError::SessionCommandFailed(
                    "type_text(): web element passed to mobile session".to_string(),
                ))
            }
        };
        elem.clear().await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("clear({}): {e}", element.selector))
        })?;
        elem.send_keys(text).await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("send_keys({}): {e}", element.selector))
        })
    }

    async fn get_text(&self, element: &UtoElement) -> UtoResult<String> {
        let elem = match &element.handle {
            ElementHandle::Mobile(e) => e,
            ElementHandle::Web(_) => {
                return Err(UtoError::SessionCommandFailed(
                    "get_text(): web element passed to mobile session".to_string(),
                ))
            }
        };
        elem.text().await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("get_text({}): {e}", element.selector))
        })
    }

    async fn screenshot(&self) -> UtoResult<Vec<u8>> {
        self.driver.screenshot_as_png().await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("screenshot(): {e}"))
        })
    }

    async fn close(self: Box<Self>) -> UtoResult<()> {
        self.driver.quit().await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("close(): {e}"))
        })?;
        log::info!("Mobile session closed");
        Ok(())
    }
}
