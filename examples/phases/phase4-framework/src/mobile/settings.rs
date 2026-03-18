use uto_core::error::{UtoError, UtoResult};
use uto_test::{ManagedSession, Suite};

use crate::finish_test;

pub fn register(suite: Suite) -> Suite {
    suite
        .test("mobile: settings launches", settings_launches)
        .test("mobile: search intent resolves", search_intent_resolves)
}

pub async fn settings_launches(session: ManagedSession) -> UtoResult<()> {
    let result = session
        .launch_android_activity("com.android.settings", ".Settings")
        .await;

    finish_test(session, result).await
}

pub async fn search_intent_resolves(session: ManagedSession) -> UtoResult<()> {
    let result = async {
        session
            .launch_android_activity("com.android.settings", ".Settings")
            .await?;

        if session.wait_for_intent("Search", 1500).await.is_ok() {
            return Ok(());
        }

        for label in ["Search settings", "Search", "Rechercher", "Buscar", "Suchen"] {
            if session.select(label).await.is_ok() {
                return Ok(());
            }
        }

        Err(UtoError::SessionCommandFailed(
            "mobile assertion failed: no search intent resolved".to_string(),
        ))
    }
    .await;

    finish_test(session, result).await
}