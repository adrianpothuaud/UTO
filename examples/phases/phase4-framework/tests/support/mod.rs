use uto_test::ManagedSession;

pub async fn start_web_or_skip() -> Option<ManagedSession> {
    match uto_test::startNewSession("chrome").await {
        Ok(session) => Some(session),
        Err(err) => {
            println!("Skipping web example: web session could not be created: {err}");
            None
        }
    }
}

pub async fn start_mobile_or_skip() -> Option<ManagedSession> {
    match uto_test::startNewSessionWithArg("android", 16).await {
        Ok(session) => Some(session),
        Err(err) => {
            println!("Skipping mobile example: mobile session could not be created: {err}");
            None
        }
    }
}