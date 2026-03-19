//! Saucedemo Login example — real-world UTO test project targeting
//! https://www.saucedemo.com (a public demo e-commerce app used for
//! automation training).
//!
//! Structured as a page-object library so the tests in `tests/` stay focused
//! on intent, assertions, and expected outcomes.

pub mod web;

use uto_core::error::UtoResult;
use uto_test::ManagedSession;

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
