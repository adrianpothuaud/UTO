use uto_core::{
    driver::DriverProcess,
    error::{UtoError, UtoResult},
    session::{mobile::MobileSession, web::WebSession, UtoElement, UtoSession},
};

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
}

impl ManagedSession {
    pub(crate) fn from_web(session: WebSession, driver: DriverProcess) -> Self {
        Self {
            inner: Some(SessionInner::Web(session)),
            driver: Some(driver),
            target: "chrome",
        }
    }

    pub(crate) fn from_mobile(session: MobileSession, driver: DriverProcess) -> Self {
        Self {
            inner: Some(SessionInner::Mobile(session)),
            driver: Some(driver),
            target: "android",
        }
    }

    /// Returns the normalized target kind.
    pub fn target(&self) -> &'static str {
        self.target
    }

    /// Navigates the active session to a URL/deep-link.
    pub async fn goto(&self, url: &str) -> UtoResult<()> {
        match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.goto(url).await,
            Some(SessionInner::Mobile(session)) => session.goto(url).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        }
    }

    /// Returns page title/app activity title.
    pub async fn title(&self) -> UtoResult<String> {
        match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.title().await,
            Some(SessionInner::Mobile(session)) => session.title().await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        }
    }

    /// Finds an element by selector.
    pub async fn find_element(&self, selector: &str) -> UtoResult<UtoElement> {
        match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.find_element(selector).await,
            Some(SessionInner::Mobile(session)) => session.find_element(selector).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        }
    }

    /// Selects an element by human-readable label.
    pub async fn select(&self, label: &str) -> UtoResult<UtoElement> {
        match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.select(label).await,
            Some(SessionInner::Mobile(session)) => session.select(label).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        }
    }

    /// Clicks by intent label.
    pub async fn click_intent(&self, label: &str) -> UtoResult<()> {
        match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.click_intent(label).await,
            Some(SessionInner::Mobile(session)) => session.click_intent(label).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        }
    }

    /// Fills an input by intent label.
    pub async fn fill_intent(&self, label: &str, value: &str) -> UtoResult<()> {
        match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.fill_intent(label, value).await,
            Some(SessionInner::Mobile(session)) => session.fill_intent(label, value).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        }
    }

    /// Reads text from the provided element.
    pub async fn get_text(&self, element: &UtoElement) -> UtoResult<String> {
        match self.inner.as_ref() {
            Some(SessionInner::Web(session)) => session.get_text(element).await,
            Some(SessionInner::Mobile(session)) => session.get_text(element).await,
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        }
    }

    /// Launches an Android activity for mobile sessions.
    pub async fn launch_android_activity(
        &self,
        app_package: &str,
        app_activity: &str,
    ) -> UtoResult<()> {
        match self.inner.as_ref() {
            Some(SessionInner::Mobile(session)) => {
                session.launch_activity(app_package, app_activity).await
            }
            Some(SessionInner::Web(_)) => Err(UtoError::SessionCommandFailed(
                "launch_android_activity is only available for android sessions".to_string(),
            )),
            None => Err(UtoError::SessionCommandFailed(
                "session already closed".to_string(),
            )),
        }
    }

    /// Closes the WebDriver session and stops the managed driver process.
    pub async fn close(mut self) -> UtoResult<()> {
        if let Some(inner) = self.inner.take() {
            match inner {
                SessionInner::Web(session) => Box::new(session).close().await?,
                SessionInner::Mobile(session) => Box::new(session).close().await?,
            }
        }

        if let Some(driver) = self.driver.take() {
            driver.stop()?;
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
