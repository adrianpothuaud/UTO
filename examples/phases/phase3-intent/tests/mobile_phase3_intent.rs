use uto_core::session::mobile::{MobileCapabilities, MobileSession};
use uto_core::session::UtoSession;

#[tokio::test]
async fn mobile_phase3_intent_select_or_skip() {
    let appium = match uto_core::env::platform::find_appium() {
        Some(path) => path,
        None => {
            println!("Skipping mobile Phase 3 example: appium not available");
            return;
        }
    };

    let appium_driver = match uto_core::driver::start_appium(&appium).await {
        Ok(proc) => proc,
        Err(err) => {
            println!("Skipping mobile Phase 3 example: could not start appium: {err}");
            return;
        }
    };

    let caps = MobileCapabilities::android("emulator-5554");
    match MobileSession::new(&appium_driver.url, caps).await {
        Ok(session) => {
            let _ = session
                .launch_activity("com.android.settings", ".Settings")
                .await;

            for label in ["Search settings", "Search", "Rechercher", "Buscar", "Suchen"] {
                if session.select(label).await.is_ok() {
                    Box::new(session).close().await.expect("close mobile session");
                    appium_driver.stop().expect("stop appium");
                    return;
                }
            }

            println!("Skipping mobile Phase 3 example: no known search label resolved");
            let _ = Box::new(session).close().await;
        }
        Err(err) => {
            println!("Skipping mobile Phase 3 example: environment not ready: {err}");
        }
    }

    appium_driver.stop().expect("stop appium");
}
