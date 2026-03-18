#[tokio::test]
async fn mobile_example_phase4_uses_mobile_helpers_or_skips() {
    let session = match uto_test::startNewSessionWithArg("android", 16).await {
        Ok(session) => session,
        Err(err) => {
            println!("Skipping mobile example: mobile session could not be created: {err}");
            return;
        }
    };

    if let Err(err) = session
        .launch_android_activity("com.android.settings", ".Settings")
        .await
    {
        println!("Skipping mobile launch activity: {err}");
        let _ = session.close().await;
        return;
    }

    let _ = session.wait_for_intent("Search", 1500).await;

    session.close().await.expect("close mobile session");
}
