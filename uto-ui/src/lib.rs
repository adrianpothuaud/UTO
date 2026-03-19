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
//! - [`runner`] — Subprocess bridge: spawns `uto run --project <dir>`, relays
//!   stdout/stderr as live log events, forwards structured per-test execution
//!   events over WebSocket, and broadcasts `run_finished` on completion.
//! - [`watcher`] — Filesystem watcher: watches the project `tests/` directory for changes and
//!   auto-triggers a re-run (debounced at 300 ms) when `--watch` is enabled.
//!
//! ## Quick start
//!
//! ```no_run
//! use uto_ui::{UiOptions, start_server_sync};
//!
//! let opts = UiOptions { port: 4000, ..Default::default() };
//! start_server_sync(opts).expect("server error");
//! ```

pub(crate) mod runner;
pub mod server;
pub(crate) mod watcher;

pub use server::{start_server, start_server_sync, UiOptions};
