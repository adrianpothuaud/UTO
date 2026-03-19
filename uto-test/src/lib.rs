//! End-user helpers for concise UTO test authoring.
//!
//! This crate provides a simple session bootstrap API so test files can stay
//! focused on intent/actions/assertions while setup and lifecycle remain
//! observable through logs.
//!
//! # Library Usage (without CLI scaffolding)
//!
//! UTO can be used as a standalone library in any Rust project:
//!
//! ```rust,no_run
//! use uto_test::{uto_test, startNewSession};
//!
//! #[uto_test(target = "web")]
//! #[tokio::test]
//! async fn my_test() {
//!     let session = startNewSession("chrome").await.unwrap();
//!     session.goto("https://example.com").await.unwrap();
//!     session.close().await.unwrap();
//! }
//! ```
//!
//! No `uto.json` or CLI scaffolding required.

mod live_stream;
mod managed_session;
mod start;
mod suite;

pub use managed_session::ManagedSession;
pub use start::{start_new_session, start_new_session_with_hint};
pub use suite::Suite;
pub use uto_test_macros::uto_test;

pub use start::{startNewSession, startNewSessionWithArg};

// Re-export core session types for library users
pub use uto_core::session::{MobileSession, UtoSession, WebSession};
