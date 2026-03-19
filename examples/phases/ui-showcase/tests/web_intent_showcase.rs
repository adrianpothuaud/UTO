//! Phase 5 UI showcase — runnerless web tests used by `uto run` and `uto ui`.

use uto_core::error::{UtoError, UtoResult};
use uto_test::{uto_test, ManagedSession};

#[uto_test(target = "web")]
#[tokio::test]
async fn web_navigate_and_verify_visibility() {
    let session = match uto_test::startNewSession("chrome").await {
        Ok(session) => session,
        Err(err) => {
            println!("Skipping web test: could not create web session: {err}");
            return;
        }
    };

    web_title_test(session).await.expect("web title scenario");
}

#[uto_test(target = "web")]
#[tokio::test]
async fn web_multi_step_workflow() {
    let session = match uto_test::startNewSession("chrome").await {
        Ok(session) => session,
        Err(err) => {
            println!("Skipping web workflow test: could not create web session: {err}");
            return;
        }
    };

    web_form_test(session).await.expect("web form scenario");
}

async fn web_title_test(session: ManagedSession) -> UtoResult<()> {
    session.goto("https://example.com").await?;
    let title = session.title().await?;
    if title.is_empty() {
        return Err(UtoError::SessionCommandFailed(
            "title assertion failed: expected non-empty title".to_string(),
        ));
    }
    session.close().await
}

async fn web_form_test(session: ManagedSession) -> UtoResult<()> {
    session
        .goto(concat!(
            "data:text/html,",
            "<html><body>",
            "<input id='e' aria-label='Email' type='text'/>",
            "<button id='s' aria-label='Submit' ",
            "onclick=\"document.getElementById('o').textContent=",
            "document.getElementById('e').value\">Submit</button>",
            "<p id='o'>-</p>",
            "</body></html>"
        ))
        .await?;

    session.fill_intent("Email", "template@uto.dev").await?;
    session.click_intent("Submit").await?;

    let out = session.find_element("#o").await?;
    let text = session.get_text(&out).await?;
    if text != "template@uto.dev" {
        return Err(UtoError::SessionCommandFailed(format!(
            "form assertion failed: expected 'template@uto.dev', got '{text}'"
        )));
    }

    session.close().await
}
