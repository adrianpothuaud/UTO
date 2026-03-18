//! UTO UI mode — embedded HTTP + WebSocket server for interactive test visualization.
//!
//! # Overview
//!
//! This crate provides the `uto ui` CLI command backend: a lightweight local HTTP + WebSocket
//! server that serves an embedded single-page application (SPA) for running, watching, and
//! debugging UTO test suites in the browser.
//!
//! ## Architecture
//!
//! - [`server`] — Axum-based HTTP + WebSocket server with embedded SPA and REST API.
//!
//! ## Quick start
//!
//! ```no_run
//! use uto_ui::{UiOptions, start_server_sync};
//!
//! fn main() {
//!     let opts = UiOptions { port: 4000, ..Default::default() };
//!     start_server_sync(opts).expect("server error");
//! }
//! ```

pub mod server;

pub use server::{start_server, start_server_sync, UiOptions};
