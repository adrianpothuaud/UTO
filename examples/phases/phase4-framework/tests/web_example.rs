#[tokio::test]
async fn web_example_phase4_uses_managed_session_or_skips() {
    let session = match uto_test::startNewSession("chrome").await {
        Ok(session) => session,
        Err(err) => {
            println!("Skipping web example: web session could not be created: {err}");
            return;
        }
    };

    session.goto("https://example.com").await.expect("goto");
    let title = session.title().await.expect("title");
    assert!(!title.is_empty(), "title should be non-empty");

    session.close().await.expect("close");
}
