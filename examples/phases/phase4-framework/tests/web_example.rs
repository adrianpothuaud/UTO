#[path = "support/mod.rs"]
mod support;

#[tokio::test]
async fn web_example_phase4_title_is_non_empty_or_skips() {
    let Some(session) = support::start_web_or_skip().await else {
        return;
    };

    phase4_framework::web::basic::page_title_is_non_empty(session)
        .await
        .expect("page title scenario");
}

#[tokio::test]
async fn web_example_phase4_hero_copy_is_visible_or_skips() {
    let Some(session) = support::start_web_or_skip().await else {
        return;
    };

    phase4_framework::web::basic::hero_copy_is_visible(session)
        .await
        .expect("hero copy scenario");
}
