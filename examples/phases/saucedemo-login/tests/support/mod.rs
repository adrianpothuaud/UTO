use uto_test::ManagedSession;

/// Attempts to start a Chrome web session, skipping the test if the driver
/// is not available (e.g. in CI without a browser).
pub async fn start_web_or_skip() -> Option<ManagedSession> {
    match uto_test::startNewSession("chrome").await {
        Ok(session) => Some(session),
        Err(err) => {
            println!("Skipping web test: session could not be created: {err}");
            None
        }
    }
}
