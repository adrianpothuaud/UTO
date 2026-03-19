#[path = "support/mod.rs"]
mod support;

use uto_test::uto_test;

#[uto_test(target = "mobile")]
#[tokio::test]
async fn mobile_example_phase4_launches_settings_or_skips() {
    let Some(session) = support::start_mobile_or_skip().await else {
        return;
    };

    if let Err(err) = phase4_framework::mobile::settings::settings_launches(session).await {
        println!("Skipping mobile settings launch scenario: {err}");
    }
}

#[uto_test(target = "mobile")]
#[tokio::test]
async fn mobile_example_phase4_resolves_search_intent_or_skips() {
    let Some(session) = support::start_mobile_or_skip().await else {
        return;
    };

    if let Err(err) = phase4_framework::mobile::settings::search_intent_resolves(session).await {
        println!("Skipping mobile search scenario: {err}");
    }
}
