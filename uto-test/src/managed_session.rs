use uto_core::{
    driver::DriverProcess,
    error::{UtoError, UtoResult},
    session::{mobile::MobileSession, web::WebSession, UtoElement, UtoSession},
};
use uto_reporter::ReportEvent;

use std::sync::{Arc, Mutex};

pub(crate) type SharedEvents = Arc<Mutex<Vec<ReportEvent>>>;

pub(crate) enum SessionInner {
    Web(WebSession),
    Mobile(MobileSession),
}

/// A managed test session with a consistent API across web and mobile.
///
/// `ManagedSession` owns both the WebDriver session and underlying driver
/// process so a single `close()` call performs full cleanup.
pub struct ManagedSession {
    inner: Option<SessionInner>,
    driver: Option<DriverProcess>,
    target: &'static str,
    report_events: Option<SharedEvents>,
}

impl ManagedSession {
    pub(crate) fn from_web(
        session: WebSession,
        driver: DriverProcess,
        report_events: Option<SharedEvents>,
    ) -> Self {
        Self {
            inner: Some(SessionInner::Web(session)),
            driver: Some(driver),
            target: "chrome",
            report_events,
        }
    }

    pub(crate) fn from_mobile(
        session: MobileSession,
        driver: DriverProcess,
        report_events: Option<SharedEvents>,
    ) -> Self {
        Self {
            inner: Some(SessionInner::Mobile(session)),
            driver: Some(driver),
            target: "android",
            report_events,
        }
    }

    fn record_event(&self, stage: &str, status: &str, detail: serde_json::Value) {
        if let Some(events) = &self.report_events {
            if let Ok(mut guard) = events.lock() {
                guard.push(ReportEvent {
                    stage: stage.to_string(),
                    status: status.to_string(),
                    detail,
                });
            }
        }
    }

    /// Returns the normalized target kind.
    pub fn target(&self) -> &'static str {
        self.target
    }

    /// Navigates the active session to a URL/deep-link.
    pub async fn goto(&self, url: &str) -> UtoResult<()> {
        let result = match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.goto(url).await,
            Some(SessionInner::Mobile(session)) => session.goto(url).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        };

        match &result {
            Ok(()) => self.record_event(
                "session.goto",
                "ok",
                serde_json::json!({ "target": self.target(), "url": url }),
            ),
            Err(err) => self.record_event(
                "session.goto",
                "failed",
                serde_json::json!({ "target": self.target(), "url": url, "error": err.to_string() }),
            ),
        }

        result
    }

    /// Returns page title/app activity title.
    pub async fn title(&self) -> UtoResult<String> {
        let result = match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.title().await,
            Some(SessionInner::Mobile(session)) => session.title().await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        };

        match &result {
            Ok(title) => self.record_event(
                "session.title",
                "ok",
                serde_json::json!({ "target": self.target(), "title": title }),
            ),
            Err(err) => self.record_event(
                "session.title",
                "failed",
                serde_json::json!({ "target": self.target(), "error": err.to_string() }),
            ),
        }

        result
    }

    /// Finds an element by selector.
    pub async fn find_element(&self, selector: &str) -> UtoResult<UtoElement> {
        let result = match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.find_element(selector).await,
            Some(SessionInner::Mobile(session)) => session.find_element(selector).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        };

        match &result {
            Ok(element) => self.record_event(
                "session.find_element",
                "ok",
                serde_json::json!({
                    "target": self.target(),
                    "selector": selector,
                    "resolved_selector": element.selector,
                    "label": element.label,
                }),
            ),
            Err(err) => self.record_event(
                "session.find_element",
                "failed",
                serde_json::json!({ "target": self.target(), "selector": selector, "error": err.to_string() }),
            ),
        }

        result
    }

    /// Selects an element by human-readable label.
    pub async fn select(&self, label: &str) -> UtoResult<UtoElement> {
        let result = match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.select(label).await,
            Some(SessionInner::Mobile(session)) => session.select(label).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        };

        match &result {
            Ok(element) => self.record_event(
                "intent.select",
                "ok",
                serde_json::json!({
                    "target": self.target(),
                    "label": label,
                    "resolved_selector": element.selector,
                }),
            ),
            Err(err) => self.record_event(
                "intent.select",
                "failed",
                serde_json::json!({ "target": self.target(), "label": label, "error": err.to_string() }),
            ),
        }

        result
    }

    /// Clicks by intent label.
    pub async fn click_intent(&self, label: &str) -> UtoResult<()> {
        let result = match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.click_intent(label).await,
            Some(SessionInner::Mobile(session)) => session.click_intent(label).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        };

        match &result {
            Ok(()) => self.record_event(
                "intent.click",
                "ok",
                serde_json::json!({ "target": self.target(), "label": label }),
            ),
            Err(err) => self.record_event(
                "intent.click",
                "failed",
                serde_json::json!({ "target": self.target(), "label": label, "error": err.to_string() }),
            ),
        }

        result
    }

    /// Fills an input by intent label.
    pub async fn fill_intent(&self, label: &str, value: &str) -> UtoResult<()> {
        let result = match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.fill_intent(label, value).await,
            Some(SessionInner::Mobile(session)) => session.fill_intent(label, value).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        };

        match &result {
            Ok(()) => self.record_event(
                "intent.fill",
                "ok",
                serde_json::json!({ "target": self.target(), "label": label, "value": value }),
            ),
            Err(err) => self.record_event(
                "intent.fill",
                "failed",
                serde_json::json!({ "target": self.target(), "label": label, "error": err.to_string() }),
            ),
        }

        result
    }

    /// Reads text from the provided element.
    pub async fn get_text(&self, element: &UtoElement) -> UtoResult<String> {
        let result = match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.get_text(element).await,
            Some(SessionInner::Mobile(session)) => session.get_text(element).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        };

        match &result {
            Ok(text) => self.record_event(
                "assert.get_text",
                "ok",
                serde_json::json!({ "target": self.target(), "selector": element.selector, "text": text }),
            ),
            Err(err) => self.record_event(
                "assert.get_text",
                "failed",
                serde_json::json!({ "target": self.target(), "selector": element.selector, "error": err.to_string() }),
            ),
        }

        result
    }

    /// Launches an Android activity for mobile sessions.
    pub async fn launch_android_activity(
        &self,
        app_package: &str,
        app_activity: &str,
    ) -> UtoResult<()> {
        let result = match self.inner.as_ref() {
            Some(SessionInner::Mobile(session)) => {
                session.launch_activity(app_package, app_activity).await
            }
            Some(SessionInner::Web(_)) => Err(UtoError::SessionCommandFailed(
                "launch_android_activity is only available for android sessions".to_string(),
            )),
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        };

        match &result {
            Ok(()) => self.record_event(
                "session.launch_activity",
                "ok",
                serde_json::json!({
                    "target": self.target(),
                    "app_package": app_package,
                    "app_activity": app_activity,
                }),
            ),
            Err(err) => self.record_event(
                "session.launch_activity",
                "failed",
                serde_json::json!({
                    "target": self.target(),
                    "app_package": app_package,
                    "app_activity": app_activity,
                    "error": err.to_string(),
                }),
            ),
        }

        result
    }

    // -----------------------------------------------------------------------
    // Mobile-specific intent helpers (Phase 4.3)
    // -----------------------------------------------------------------------

    /// Waits for an element to be present and returns it. Mobile-only helper.
    ///
    /// Polls the page source / accessibility tree every 50ms until the element
    /// selector matches or timeout is exceeded.
    pub async fn wait_for_element(&self, selector: &str, timeout_ms: u64) -> UtoResult<UtoElement> {
        let result = match self.inner.as_ref() {
            Some(SessionInner::Mobile(session)) => {
                session.wait_for_element(selector, timeout_ms).await
            }
            Some(SessionInner::Web(_)) => Err(UtoError::SessionCommandFailed(
                "wait_for_element on mobile is recommended; web sessions should use native polling"
                    .to_string(),
            )),
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        };

        match &result {
            Ok(element) => self.record_event(
                "mobile.wait_for_element",
                "ok",
                serde_json::json!({
                    "target": self.target(),
                    "selector": selector,
                    "timeout_ms": timeout_ms,
                    "resolved_selector": element.selector,
                }),
            ),
            Err(err) => self.record_event(
                "mobile.wait_for_element",
                "failed",
                serde_json::json!({
                    "target": self.target(),
                    "selector": selector,
                    "timeout_ms": timeout_ms,
                    "error": err.to_string(),
                }),
            ),
        }

        result
    }

    /// Scrolls through the page to find an element by intent label, then clicks it.
    ///
    /// Use this helper for mobile lists/scrollable content where the target
    /// element is not initially visible on screen.
    ///
    /// Default: up to 10 scroll attempts before giving up.
    pub async fn scroll_intent(&self, label: &str) -> UtoResult<()> {
        self.scroll_intent_with_max(label, 10).await
    }

    /// Scrolls with custom maximum scroll attempts.
    pub async fn scroll_intent_with_max(&self, label: &str, max_scrolls: usize) -> UtoResult<()> {
        let result = match self.inner.as_ref() {
            Some(SessionInner::Mobile(session)) => {
                session.scroll_intent(label, max_scrolls).await
            }
            Some(SessionInner::Web(_)) => Err(UtoError::SessionCommandFailed(
                "scroll_intent is mobile-only; use web native scrolling patterns"
                    .to_string(),
            )),
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        };

        match &result {
            Ok(()) => self.record_event(
                "mobile.scroll_intent",
                "ok",
                serde_json::json!({ "target": self.target(), "label": label, "max_scrolls": max_scrolls }),
            ),
            Err(err) => self.record_event(
                "mobile.scroll_intent",
                "failed",
                serde_json::json!({
                    "target": self.target(),
                    "label": label,
                    "max_scrolls": max_scrolls,
                    "error": err.to_string(),
                }),
            ),
        }

        result
    }

    /// Waits for an element to be resolvable by intent label, then clicks it.
    ///
    /// Use this helper when the element is likely in-view but the accessibility
    /// tree is loading asynchronously. Polls every 200ms until found or timeout.
    pub async fn wait_for_intent(&self, label: &str, timeout_ms: u64) -> UtoResult<()> {
        let result = match self.inner.as_ref() {
            Some(SessionInner::Mobile(session)) => {
                session.wait_for_intent(label, timeout_ms).await
            }
            Some(SessionInner::Web(_)) => Err(UtoError::SessionCommandFailed(
                "wait_for_intent is mobile-only".to_string(),
            )),
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        };

        match &result {
            Ok(()) => self.record_event(
                "mobile.wait_for_intent",
                "ok",
                serde_json::json!({ "target": self.target(), "label": label, "timeout_ms": timeout_ms }),
            ),
            Err(err) => self.record_event(
                "mobile.wait_for_intent",
                "failed",
                serde_json::json!({
                    "target": self.target(),
                    "label": label,
                    "timeout_ms": timeout_ms,
                    "error": err.to_string(),
                }),
            ),
        }

        result
    }

    /// Closes the WebDriver session and stops the managed driver process.
    pub async fn close(mut self) -> UtoResult<()> {
        let target = self.target;
        if let Some(inner) = self.inner.take() {
            let result = match inner {
                SessionInner::Web(session) => Box::new(session).close().await?,
                SessionInner::Mobile(session) => Box::new(session).close().await?,
            };
            self.record_event(
                "session.close",
                "ok",
                serde_json::json!({ "target": target }),
            );
            result
        }

        if let Some(driver) = self.driver.take() {
            match driver.stop() {
                Ok(()) => self.record_event(
                    "driver.stop",
                    "ok",
                    serde_json::json!({ "target": target }),
                ),
                Err(err) => {
                    self.record_event(
                        "driver.stop",
                        "failed",
                        serde_json::json!({ "target": target, "error": err.to_string() }),
                    );
                    return Err(err);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uto_core::error::UtoError;

    fn closed_session(target: &'static str) -> ManagedSession {
        ManagedSession {
            inner: None,
            driver: None,
            target,
            report_events: None,
        }
    }

    #[test]
    fn target_accessor_returns_constructor_target() {
        assert_eq!(closed_session("chrome").target(), "chrome");
        assert_eq!(closed_session("android").target(), "android");
    }

    fn assert_closed_error(err: UtoError) {
        let message = err.to_string();
        assert!(message.contains("session already closed"), "{message}");
    }

    #[tokio::test]
    async fn goto_returns_closed_error_when_session_is_closed() {
        let err = closed_session("chrome").goto("https://example.com").await;
        assert_closed_error(err.expect_err("expected closed-session error"));
    }

    #[tokio::test]
    async fn title_returns_closed_error_when_session_is_closed() {
        let err = closed_session("chrome").title().await;
        assert_closed_error(err.expect_err("expected closed-session error"));
    }

    #[tokio::test]
    async fn find_element_returns_closed_error_when_session_is_closed() {
        let err = closed_session("chrome").find_element("#missing").await;
        assert_closed_error(err.expect_err("expected closed-session error"));
    }

    #[tokio::test]
    async fn select_returns_closed_error_when_session_is_closed() {
        let err = closed_session("chrome").select("Submit").await;
        assert_closed_error(err.expect_err("expected closed-session error"));
    }

    #[tokio::test]
    async fn click_intent_returns_closed_error_when_session_is_closed() {
        let err = closed_session("chrome").click_intent("Submit").await;
        assert_closed_error(err.expect_err("expected closed-session error"));
    }

    #[tokio::test]
    async fn fill_intent_returns_closed_error_when_session_is_closed() {
        let err = closed_session("chrome")
            .fill_intent("Email", "phase4@uto.dev")
            .await;
        assert_closed_error(err.expect_err("expected closed-session error"));
    }

    #[tokio::test]
    async fn launch_android_activity_returns_closed_error_when_session_is_closed() {
        let err = closed_session("android")
            .launch_android_activity("com.android.settings", ".Settings")
            .await;
        assert_closed_error(err.expect_err("expected closed-session error"));
    }
}
