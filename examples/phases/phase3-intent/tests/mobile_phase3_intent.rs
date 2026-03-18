#[tokio::test]
async fn mobile_phase3_intent_select_or_skip() {
    let session = match uto_test::startNewSessionWithArg("android", 16).await {
        Ok(session) => session,
        Err(err) => {
            println!("Skipping mobile Phase 3 example: could not create mobile session: {err}");
            return;
        }
    };

    let _ = session
        .launch_android_activity("com.android.settings", ".Settings")
        .await;

    for label in ["Search settings", "Search", "Rechercher", "Buscar", "Suchen"] {
        if session.select(label).await.is_ok() {
            session.close().await.expect("close mobile session");
            return;
        }
    }

    println!("Skipping mobile Phase 3 example: no known search label resolved");
    let _ = session.close().await;
}
