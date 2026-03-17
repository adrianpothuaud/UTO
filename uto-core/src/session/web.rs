use async_trait::async_trait;
use thirtyfour::common::capabilities::chromium::ChromiumLikeCapabilities;
use thirtyfour::{By, ChromeCapabilities, WebDriver};

use crate::error::{UtoError, UtoResult};
use crate::vision::{
    resolve_candidates, select_by_label, summarize_ranked_candidates, AccessibilityNode,
    ConsensusConfig, DetectedElement,
};

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

    async fn collect_select_candidates(
        &self,
    ) -> UtoResult<
        Vec<(
            DetectedElement,
            AccessibilityNode,
            thirtyfour::WebElement,
            String,
        )>,
    > {
        // Focus on commonly interactive elements first.
        const CANDIDATE_SELECTOR: &str =
            "button,a,input,textarea,select,[role='button'],[role='link'],[role='textbox']";

        let nodes = self
            .driver
            .find_all(By::Css(CANDIDATE_SELECTOR))
            .await
            .map_err(|e| {
                UtoError::SessionCommandFailed(format!(
                    "collect_select_candidates({CANDIDATE_SELECTOR}): {e}"
                ))
            })?;

        let mut out = Vec::with_capacity(nodes.len());
        for (idx, elem) in nodes.into_iter().enumerate() {
            let tag = elem
                .tag_name()
                .await
                .unwrap_or_else(|_| "unknown".to_string());
            let role = elem.attr("role").await.unwrap_or(None);
            let text = elem.text().await.unwrap_or_default();
            let aria_label = elem.attr("aria-label").await.unwrap_or(None);
            let placeholder = elem.attr("placeholder").await.unwrap_or(None);
            let value = elem.attr("value").await.unwrap_or(None);
            let id_attr = elem.attr("id").await.unwrap_or(None);

            let label = first_non_empty(&[
                aria_label.clone(),
                non_empty(text),
                placeholder.clone(),
                value.clone(),
            ]);
            let element_type = role.clone().unwrap_or_else(|| tag.clone());

            let rect = elem.rect().await.map_err(|e| {
                UtoError::SessionCommandFailed(format!("select() rect lookup failed: {e}"))
            })?;
            let bbox = (
                rect.x.round() as i32,
                rect.y.round() as i32,
                rect.width.max(1.0).round() as i32,
                rect.height.max(1.0).round() as i32,
            );

            let detected = DetectedElement {
                bbox,
                confidence: 0.75,
                element_type: element_type.clone(),
                label: label.clone(),
            };

            let accessibility = AccessibilityNode {
                label,
                role: Some(element_type),
                bbox: Some(bbox),
            };

            let selector = if let Some(id) = id_attr {
                format!("#{id}")
            } else {
                format!("{tag}:nth-match({idx})")
            };

            out.push((detected, accessibility, elem, selector));
        }

        Ok(out)
    }

    /// Returns a human-readable ranking summary for an intent label.
    pub async fn debug_select_ranking(&self, label: &str, max_items: usize) -> UtoResult<String> {
        let candidates = self.collect_select_candidates().await?;
        if candidates.is_empty() {
            return Ok("<no-candidates>".to_string());
        }

        let detected: Vec<DetectedElement> = candidates.iter().map(|c| c.0.clone()).collect();
        let accessibility: Vec<AccessibilityNode> =
            candidates.iter().map(|c| c.1.clone()).collect();
        let ranked = resolve_candidates(
            &detected,
            &accessibility,
            label,
            &ConsensusConfig::default(),
        );

        Ok(summarize_ranked_candidates(&ranked, max_items))
    }
}

fn non_empty(value: String) -> Option<String> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn first_non_empty(candidates: &[Option<String>]) -> Option<String> {
    candidates
        .iter()
        .filter_map(|v| v.as_ref())
        .map(|s| s.trim())
        .find(|s| !s.is_empty())
        .map(ToString::to_string)
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

    async fn select(&self, label: &str) -> UtoResult<UtoElement> {
        let candidates = self.collect_select_candidates().await?;
        if candidates.is_empty() {
            return Err(UtoError::VisionResolutionFailed(
                "select(): no interactive web candidates found".to_string(),
            ));
        }

        let detected: Vec<DetectedElement> = candidates.iter().map(|c| c.0.clone()).collect();
        let accessibility: Vec<AccessibilityNode> =
            candidates.iter().map(|c| c.1.clone()).collect();

        if log::log_enabled!(log::Level::Debug) {
            let ranked = resolve_candidates(
                &detected,
                &accessibility,
                label,
                &ConsensusConfig::default(),
            );
            log::debug!(
                "select('{}') candidate ranking: {}",
                label,
                summarize_ranked_candidates(&ranked, 3)
            );
        }

        let resolved = select_by_label(&detected, &accessibility, label)?;

        let selected = candidates
            .into_iter()
            .find(|(d, _, _, _)| {
                d.bbox == resolved.element.bbox
                    && d.element_type == resolved.element.element_type
                    && d.label == resolved.element.label
            })
            .ok_or_else(|| {
                UtoError::VisionResolutionFailed(
                    "select(): resolver produced a candidate that could not be mapped to a DOM node"
                        .to_string(),
                )
            })?;

        let (detected, _ax, elem, selector) = selected;
        Ok(UtoElement {
            label: detected.label.unwrap_or_else(|| label.to_string()),
            selector,
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
