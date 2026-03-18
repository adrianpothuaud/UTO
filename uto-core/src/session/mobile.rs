use async_trait::async_trait;
use serde_json::json;
use thirtyfour::{By, Capabilities, WebDriver};

use crate::error::{UtoError, UtoResult};

use super::mobile_accessibility::build_mobile_select_xpaths;
pub use super::mobile_capabilities::{MobileCapabilities, MobilePlatform};
use super::mobile_connection::{
    appium_alternate_base_url, connect_appium_driver, is_appium_base_path_mismatch_error,
};
use super::{ElementHandle, UtoElement, UtoSession};

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
/// let session = MobileSession::new("http://localhost:4723", caps).await?;
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
        use super::appium_probe::probe_appium;

        let appium_url = appium_url.trim_end_matches('/');

        // Preflight: probe Appium to detect configuration issues early.
        if let Ok(probe) = probe_appium(appium_url).await {
            log::debug!(
                "Appium probe: status_ok={}, session_ok={}, version={:?}, drivers={:?}",
                probe.status_endpoint_ok,
                probe.session_endpoint_ok,
                probe.appium_version,
                probe.available_drivers
            );
        }

        // Build the W3C alwaysMatch capability payload.
        let caps_json = capabilities.to_json();

        // `Capabilities` in thirtyfour is `serde_json::Map<String, Value>`.
        // Convert the JSON object directly; use an empty map if conversion fails.
        let caps: Capabilities = caps_json.as_object().cloned().unwrap_or_default();

        let primary_error = match connect_appium_driver(appium_url, &caps).await {
            Ok(driver) => {
                log::info!(
                    "Mobile session created ({} device '{}' via Appium at {appium_url})",
                    capabilities.platform.platform_name(),
                    capabilities.device_name
                );
                return Ok(Self { driver });
            }
            Err(e) => e,
        };

        let primary_message = primary_error.to_string();

        if is_appium_base_path_mismatch_error(&primary_message) {
            if let Some(alternate_url) = appium_alternate_base_url(appium_url) {
                if alternate_url != appium_url {
                    log::warn!(
                        "Appium session creation failed at {} with a possible base-path mismatch; retrying at {}",
                        appium_url,
                        alternate_url
                    );

                    match connect_appium_driver(&alternate_url, &caps).await {
                        Ok(driver) => {
                            log::info!(
                                "Mobile session created ({} device '{}' via Appium at {})",
                                capabilities.platform.platform_name(),
                                capabilities.device_name,
                                alternate_url
                            );
                            return Ok(Self { driver });
                        }
                        Err(secondary_error) => {
                            return Err(UtoError::SessionCreationFailed(format!(
                                "Appium at {appium_url} (retry at {alternate_url}) failed: primary error: {primary_message}; retry error: {secondary_error}"
                            )));
                        }
                    }
                }
            }
        }

        Err(UtoError::SessionCreationFailed(format!(
            "Appium at {appium_url}: {primary_message}"
        )))
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
            .map_err(|e| UtoError::SessionCommandFailed(format!("swipe(): {e}")))?;

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
            .map_err(|e| UtoError::SessionCommandFailed(format!("tap({x}, {y}): {e}")))?;

        Ok(())
    }

    /// Returns the page source (XML accessibility tree dump from Appium).
    pub async fn page_source(&self) -> UtoResult<String> {
        self.driver
            .source()
            .await
            .map_err(|e| UtoError::SessionCommandFailed(format!("page_source(): {e}")))
    }

    /// Launches an Android activity using Appium's `startActivity` mobile command.
    ///
    /// # Arguments
    ///
    /// * `package` - The Android package name (e.g. `"com.android.settings"`).
    /// * `activity` - The activity name, with or without the package prefix
    ///   (e.g. `".Settings"` or `"com.android.settings.Settings"`).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use uto_core::session::mobile::{MobileCapabilities, MobileSession};
    /// # use uto_core::session::UtoSession;
    /// # #[tokio::main]
    /// # async fn main() -> uto_core::error::UtoResult<()> {
    /// # let session = MobileSession::new("http://localhost:4723",
    /// #     MobileCapabilities::android("emulator-5554")).await?;
    /// session.launch_activity("com.android.settings", ".Settings").await?;
    /// # Box::new(session).close().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn launch_activity(&self, package: &str, activity: &str) -> UtoResult<()> {
        let cmd = json!({
            "action": "android.intent.action.MAIN",
            "flags": 0x10200000,
            "component": format!("{}/{}", package, activity)
        });

        self.driver
            .execute("mobile:startActivity", vec![cmd])
            .await
            .map_err(|e| {
                UtoError::SessionCommandFailed(format!(
                    "launch_activity({package}, {activity}): {e}"
                ))
            })?;

        // Give the activity time to launch.
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// UtoSession implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl UtoSession for MobileSession {
    async fn goto(&self, url: &str) -> UtoResult<()> {
        // For native apps this sends the deep-link via an Appium command.
        self.driver
            .goto(url)
            .await
            .map_err(|e| UtoError::SessionCommandFailed(format!("goto({url}): {e}")))
    }

    async fn title(&self) -> UtoResult<String> {
        // Returns the current activity / view controller name.
        self.driver
            .title()
            .await
            .map_err(|e| UtoError::SessionCommandFailed(format!("title(): {e}")))
    }

    async fn find_element(&self, selector: &str) -> UtoResult<UtoElement> {
        // Appium supports XPath, accessibility ID, and other locator strategies.
        // We default to XPath which works across both Android and iOS.
        let elem = self.driver.find(By::XPath(selector)).await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("find_element({selector}): {e}"))
        })?;

        let label = elem.text().await.unwrap_or_default();

        Ok(UtoElement {
            label,
            selector: selector.to_string(),
            handle: ElementHandle::Mobile(elem),
        })
    }

    async fn select(&self, label: &str) -> UtoResult<UtoElement> {
        let xpaths = build_mobile_select_xpaths(label);

        let mut last_error: Option<String> = None;
        let mut selected: Option<(thirtyfour::WebElement, String)> = None;

        for xpath in &xpaths {
            match self.driver.find(By::XPath(xpath)).await {
                Ok(elem) => {
                    selected = Some((elem, xpath.clone()));
                    break;
                }
                Err(err) => {
                    last_error = Some(err.to_string());
                }
            }
        }

        let (elem, selector_used) = selected.ok_or_else(|| {
            UtoError::VisionResolutionFailed(format!(
                "select('{label}') failed on mobile after {} xpath strategies: [{}]. Last error: {}",
                xpaths.len(),
                xpaths.join(" | "),
                last_error.unwrap_or_else(|| "<none>".to_string())
            ))
        })?;

        let resolved_label = elem.text().await.unwrap_or_else(|_| label.to_string());
        Ok(UtoElement {
            label: resolved_label,
            selector: selector_used,
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
        self.driver
            .screenshot_as_png()
            .await
            .map_err(|e| UtoError::SessionCommandFailed(format!("screenshot(): {e}")))
    }

    async fn close(self: Box<Self>) -> UtoResult<()> {
        self.driver
            .quit()
            .await
            .map_err(|e| UtoError::SessionCommandFailed(format!("close(): {e}")))?;
        log::info!("Mobile session closed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Intentionally empty for now: connection-specific tests moved to
    // `session/mobile_connection.rs` and accessibility strategy tests moved to
    // `session/mobile_accessibility.rs`.
}
