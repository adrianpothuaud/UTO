/// Web session: communicates with ChromeDriver via the W3C WebDriver protocol.
pub mod web;

/// Mobile session: communicates with Appium via the W3C WebDriver protocol.
pub mod mobile;

use async_trait::async_trait;

pub use mobile::{MobileCapabilities, MobilePlatform, MobileSession};
pub use web::WebSession;

use crate::error::UtoResult;

// ---------------------------------------------------------------------------
// Unified element representation
// ---------------------------------------------------------------------------

/// A UI element discovered in the current session.
///
/// Returned by [`UtoSession::find_element`] and accepted by interaction
/// methods such as [`UtoSession::click`] and [`UtoSession::type_text`].
#[derive(Debug, Clone)]
pub struct UtoElement {
    /// The visible text or accessible label of the element.
    pub label: String,
    /// The CSS selector or XPath expression that was used to locate the element.
    pub selector: String,
    /// Internal platform handle (opaque to callers).
    pub(crate) handle: ElementHandle,
}

/// Internal representation of the platform-specific element handle.
#[derive(Debug, Clone)]
pub(crate) enum ElementHandle {
    /// A `thirtyfour` web element handle.
    Web(thirtyfour::WebElement),
    /// A `thirtyfour` mobile element handle (Appium speaks the same protocol).
    Mobile(thirtyfour::WebElement),
}

// ---------------------------------------------------------------------------
// Session configuration
// ---------------------------------------------------------------------------

/// Configuration used to create a new UTO session.
pub enum SessionConfig {
    /// Start a Chrome browser session via ChromeDriver.
    Web {
        /// WebDriver server URL (e.g. `"http://localhost:9515"`).
        driver_url: String,
    },
    /// Start a mobile app/browser session via Appium.
    Mobile {
        /// Appium server URL (e.g. `"http://localhost:4723/wd/hub"`).
        driver_url: String,
        /// Appium capabilities describing the target device and app.
        capabilities: MobileCapabilities,
    },
}

// ---------------------------------------------------------------------------
// UtoSession trait — the unified communication interface
// ---------------------------------------------------------------------------

/// The unified, platform-agnostic interaction API.
///
/// Both [`WebSession`] and [`MobileSession`] implement this trait, which means
/// test logic can be written against `UtoSession` without caring whether it is
/// running against a browser or a mobile device.
#[async_trait]
pub trait UtoSession: Send + Sync {
    /// Navigates to the given URL (web) or deep-link URI (mobile).
    async fn goto(&self, url: &str) -> UtoResult<()>;

    /// Returns the current page title (web) or app activity name (mobile).
    async fn title(&self) -> UtoResult<String>;

    /// Finds the first element matching `selector` (CSS for web, XPath for
    /// mobile).
    async fn find_element(&self, selector: &str) -> UtoResult<UtoElement>;

    /// Clicks / taps the given element.
    async fn click(&self, element: &UtoElement) -> UtoResult<()>;

    /// Types `text` into the given element (clears existing content first).
    async fn type_text(&self, element: &UtoElement, text: &str) -> UtoResult<()>;

    /// Returns the visible text of the given element.
    async fn get_text(&self, element: &UtoElement) -> UtoResult<String>;

    /// Captures a PNG screenshot of the current view.
    ///
    /// Returns the raw PNG bytes.
    async fn screenshot(&self) -> UtoResult<Vec<u8>>;

    /// Closes the session and releases all associated resources.
    async fn close(self: Box<Self>) -> UtoResult<()>;
}
