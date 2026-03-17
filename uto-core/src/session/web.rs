use async_trait::async_trait;
use thirtyfour::common::capabilities::chromium::ChromiumLikeCapabilities;
use thirtyfour::{By, ChromeCapabilities, WebDriver};

use crate::error::{UtoError, UtoResult};

use super::{ElementHandle, UtoElement, UtoSession};

// ---------------------------------------------------------------------------
// WebSession
// ---------------------------------------------------------------------------

/// A UTO session that communicates with **ChromeDriver** via the W3C WebDriver
/// protocol.
///
/// # Example
///
/// ```rust,no_run
/// # use uto_core::session::web::WebSession;
/// # use uto_core::session::UtoSession;
/// # #[tokio::main]
/// # async fn main() -> uto_core::error::UtoResult<()> {
/// let session = WebSession::new("http://localhost:9515").await?;
/// session.goto("https://example.com").await?;
/// let title = session.title().await?;
/// println!("Page title: {title}");
/// Box::new(session).close().await?;
/// # Ok(())
/// # }
/// ```
pub struct WebSession {
    driver: WebDriver,
}

impl WebSession {
    /// Creates a new Chrome web session by connecting to the ChromeDriver
    /// server at `driver_url`.
    ///
    /// ChromeDriver must already be running and listening on the supplied URL
    /// before this is called. Use [`crate::driver::start_chromedriver`] to
    /// start a managed ChromeDriver process.
    pub async fn new(driver_url: &str) -> UtoResult<Self> {
        Self::new_with_args(driver_url, &[]).await
    }

    /// Like [`WebSession::new`] but with additional Chrome command-line arguments.
    ///
    /// Useful for running Chrome in headless or sandboxless environments:
    ///
    /// ```rust,no_run
    /// # use uto_core::session::web::WebSession;
    /// # #[tokio::main]
    /// # async fn main() -> uto_core::error::UtoResult<()> {
    /// let session = WebSession::new_with_args(
    ///     "http://localhost:9515",
    ///     &["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"],
    /// )
    /// .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new_with_args(driver_url: &str, extra_args: &[&str]) -> UtoResult<Self> {
        let mut caps = ChromeCapabilities::new();
        for arg in extra_args {
            caps.add_arg(arg).map_err(|e| {
                UtoError::SessionCreationFailed(format!("invalid chrome arg '{arg}': {e}"))
            })?;
        }
        let driver = WebDriver::new(driver_url, caps).await.map_err(|e| {
            UtoError::SessionCreationFailed(format!("ChromeDriver at {driver_url}: {e}"))
        })?;

        log::info!("Web session created (ChromeDriver at {driver_url})");
        Ok(Self { driver })
    }
}

// ---------------------------------------------------------------------------
// UtoSession implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl UtoSession for WebSession {
    async fn goto(&self, url: &str) -> UtoResult<()> {
        self.driver
            .goto(url)
            .await
            .map_err(|e| UtoError::SessionCommandFailed(format!("goto({url}): {e}")))
    }

    async fn title(&self) -> UtoResult<String> {
        self.driver
            .title()
            .await
            .map_err(|e| UtoError::SessionCommandFailed(format!("title(): {e}")))
    }

    async fn find_element(&self, selector: &str) -> UtoResult<UtoElement> {
        let elem = self.driver.find(By::Css(selector)).await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("find_element({selector}): {e}"))
        })?;

        let label = elem.text().await.unwrap_or_default();

        Ok(UtoElement {
            label,
            selector: selector.to_string(),
            handle: ElementHandle::Web(elem),
        })
    }

    async fn click(&self, element: &UtoElement) -> UtoResult<()> {
        let elem = match &element.handle {
            ElementHandle::Web(e) => e,
            ElementHandle::Mobile(_) => {
                return Err(UtoError::SessionCommandFailed(
                    "click(): mobile element passed to web session".to_string(),
                ))
            }
        };
        elem.click().await.map_err(|e| {
            UtoError::SessionCommandFailed(format!("click({}): {e}", element.selector))
        })
    }

    async fn type_text(&self, element: &UtoElement, text: &str) -> UtoResult<()> {
        let elem = match &element.handle {
            ElementHandle::Web(e) => e,
            ElementHandle::Mobile(_) => {
                return Err(UtoError::SessionCommandFailed(
                    "type_text(): mobile element passed to web session".to_string(),
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
            ElementHandle::Web(e) => e,
            ElementHandle::Mobile(_) => {
                return Err(UtoError::SessionCommandFailed(
                    "get_text(): mobile element passed to web session".to_string(),
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
        log::info!("Web session closed");
        Ok(())
    }
}
