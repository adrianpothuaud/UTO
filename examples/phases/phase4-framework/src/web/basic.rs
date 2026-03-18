use uto_core::error::{UtoError, UtoResult};
use uto_test::{ManagedSession, Suite};

use crate::finish_test;

const BASIC_PAGE: &str = concat!(
    "data:text/html,",
    "<html><head><title>UTO Phase 4</title></head><body>",
    "<main>",
    "<h1 id='hero'>Phase 4 Framework</h1>",
    "<p id='summary'>Managed sessions keep setup concise without hiding results.</p>",
    "</main>",
    "</body></html>"
);

pub fn register(suite: Suite) -> Suite {
    suite
        .test("web: page title is non-empty", page_title_is_non_empty)
        .test("web: hero copy is visible", hero_copy_is_visible)
}

pub async fn page_title_is_non_empty(session: ManagedSession) -> UtoResult<()> {
    let result = async {
        session.goto(BASIC_PAGE).await?;
        let title = session.title().await?;
        if title.is_empty() {
            return Err(UtoError::SessionCommandFailed(
                "title assertion failed: expected non-empty title".to_string(),
            ));
        }
        Ok(())
    }
    .await;

    finish_test(session, result).await
}

pub async fn hero_copy_is_visible(session: ManagedSession) -> UtoResult<()> {
    let result = async {
        session.goto(BASIC_PAGE).await?;
        let hero = session.find_element("#hero").await?;
        let text = session.get_text(&hero).await?;
        if text != "Phase 4 Framework" {
            return Err(UtoError::SessionCommandFailed(format!(
                "hero assertion failed: expected 'Phase 4 Framework', got '{text}'"
            )));
        }
        Ok(())
    }
    .await;

    finish_test(session, result).await
}