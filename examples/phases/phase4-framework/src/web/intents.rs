use uto_core::error::{UtoError, UtoResult};
use uto_test::{ManagedSession, Suite};

use crate::finish_test;

const FORM_PAGE: &str = concat!(
    "data:text/html,",
    "<html><body>",
    "<input id='e' aria-label='Email' type='text'/>",
    "<button id='s' aria-label='Submit' ",
    "onclick=\"document.getElementById('o').textContent=document.getElementById('e').value\">",
    "Submit</button>",
    "<p id='o'>-</p>",
    "</body></html>"
);

const NAV_PAGE: &str = concat!(
    "data:text/html,",
    "<html><body>",
    "<button id='lnk' aria-label='Go to next page' onclick=\"document.title='Next'\">",
    "Go to next page</button>",
    "</body></html>"
);

pub fn register(suite: Suite) -> Suite {
    suite
        .test("web: inline form fill and assert", inline_form_fill_and_assert)
        .test("web: multi-step navigation", multi_step_navigation)
}

pub async fn inline_form_fill_and_assert(session: ManagedSession) -> UtoResult<()> {
    let result = async {
        session.goto(FORM_PAGE).await?;
        session.fill_intent("Email", "suite@uto.dev").await?;
        session.click_intent("Submit").await?;

        let output = session.find_element("#o").await?;
        let text = session.get_text(&output).await?;
        if text != "suite@uto.dev" {
            return Err(UtoError::SessionCommandFailed(format!(
                "form assertion failed: expected 'suite@uto.dev', got '{text}'"
            )));
        }
        Ok(())
    }
    .await;

    finish_test(session, result).await
}

pub async fn multi_step_navigation(session: ManagedSession) -> UtoResult<()> {
    let result = async {
        session.goto(NAV_PAGE).await?;
        session.click_intent("Go to next page").await?;

        let title = session.title().await?;
        if title != "Next" {
            return Err(UtoError::SessionCommandFailed(format!(
                "navigation assertion failed: expected title 'Next', got '{title}'"
            )));
        }
        Ok(())
    }
    .await;

    finish_test(session, result).await
}