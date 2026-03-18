#[path = "support/mod.rs"]
mod support;

#[tokio::test]
async fn web_example_phase4_form_flow_or_skips() {
    let Some(session) = support::start_web_or_skip().await else {
        return;
    };

    phase4_framework::web::intents::inline_form_fill_and_assert(session)
        .await
        .expect("form intent scenario");
}

#[tokio::test]
async fn web_example_phase4_navigation_flow_or_skips() {
    let Some(session) = support::start_web_or_skip().await else {
        return;
    };

    phase4_framework::web::intents::multi_step_navigation(session)
        .await
        .expect("navigation intent scenario");
}