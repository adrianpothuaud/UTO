//! End-user helpers for concise UTO test authoring.
//!
//! This crate provides a simple session bootstrap API so test files can stay
//! focused on intent/actions/assertions while setup and lifecycle remain
//! observable through logs.

mod managed_session;
mod start;
mod suite;

pub use managed_session::ManagedSession;
pub use start::{start_new_session, start_new_session_with_hint};
pub use suite::Suite;
pub use uto_test_macros::uto_test;

pub use start::{startNewSession, startNewSessionWithArg};
