use uto_core::error::UtoResult;
use uto_test::{ManagedSession, Suite};

pub mod mobile;
pub mod web;

/// Registers the representative web capability suite for the Phase 4 example.
pub fn build_web_suite(suite: Suite) -> Suite {
    let suite = web::basic::register(suite);
    web::intents::register(suite)
}

/// Registers the representative mobile capability suite for the Phase 4 example.
pub fn build_mobile_suite(suite: Suite) -> Suite {
    mobile::settings::register(suite)
}

/// Closes a managed session while preserving the original test result.
pub async fn finish_test(session: ManagedSession, result: UtoResult<()>) -> UtoResult<()> {
    match result {
        Ok(()) => session.close().await,
        Err(err) => {
            let _ = session.close().await;
            Err(err)
        }
    }
}