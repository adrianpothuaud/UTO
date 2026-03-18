//! Multi-test suite runner for UTO test projects.
//!
//! [`Suite`] provides a WDIO-style declarative API for running multiple named
//! test cases in sequence. Each test case receives a fresh [`ManagedSession`]
//! so failures are fully isolated. Pass/fail outcomes accumulate into a
//! `uto-suite/v1` JSON and HTML report.
//!
//! # Example
//!
//! ```rust,no_run
//! use uto_test::{Suite, ManagedSession};
//! use uto_core::error::UtoResult;
//! use uto_runner::CliOptions;
//!
//! #[tokio::main]
//! async fn main() {
//!     let code = Suite::new(CliOptions::from_env())
//!         .test("home page loads", home_page_test)
//!         .test("form submission", form_test)
//!         .run()
//!         .await;
//!     std::process::exit(code);
//! }
//!
//! async fn home_page_test(session: ManagedSession) -> UtoResult<()> {
//!     session.goto("https://example.com").await?;
//!     let title = session.title().await?;
//!     assert!(!title.is_empty());
//!     session.close().await
//! }
//!
//! async fn form_test(session: ManagedSession) -> UtoResult<()> {
//!     session.goto("https://example.com/form").await?;
//!     session.fill_intent("Email", "test@uto.dev").await?;
//!     session.click_intent("Submit").await?;
//!     session.close().await
//! }
//! ```

use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use uto_core::error::UtoResult;
use uto_reporter::SuiteReport;
use uto_runner::CliOptions;

use crate::managed_session::{ManagedSession, SharedEvents};
use crate::start::start_new_session_with_hint_and_events;

// ---------------------------------------------------------------------------
// Type alias for boxed async test functions
// ---------------------------------------------------------------------------

type PinFut = Pin<Box<dyn Future<Output = UtoResult<()>> + Send + 'static>>;
type BoxedTestFn = Box<dyn FnOnce(ManagedSession) -> PinFut + Send + 'static>;

// ---------------------------------------------------------------------------
// Suite builder
// ---------------------------------------------------------------------------

/// A declarative multi-test suite runner.
///
/// Build a suite with [`Suite::new`], register test cases with [`Suite::test`],
/// then execute with [`Suite::run`]. Each test case gets its own [`ManagedSession`]
/// so failures are fully isolated. The suite emits a `uto-suite/v1` JSON artifact
/// and a matching HTML report at the end of the run.
pub struct Suite {
    options: CliOptions,
    tests: Vec<(String, BoxedTestFn)>,
}

impl Suite {
    /// Creates a new suite backed by the given CLI options.
    ///
    /// Pass `CliOptions::from_env()` for automatic argument parsing, which reads
    /// `--target`, `--json`, and `--report-file` from command-line and env vars.
    pub fn new(options: CliOptions) -> Self {
        Self {
            options,
            tests: Vec::new(),
        }
    }

    /// Registers a named async test function.
    ///
    /// `f` receives ownership of a freshly-started [`ManagedSession`]. Call
    /// [`ManagedSession::close`] at the end of the test for a clean shutdown,
    /// or let the session drop for best-effort cleanup via [`Drop`].
    pub fn test<F, Fut>(mut self, name: &str, f: F) -> Self
    where
        F: FnOnce(ManagedSession) -> Fut + Send + 'static,
        Fut: Future<Output = UtoResult<()>> + Send + 'static,
    {
        self.tests.push((
            name.to_string(),
            Box::new(move |s| Box::pin(f(s)) as PinFut),
        ));
        self
    }

    /// Executes all registered test cases sequentially and returns an exit code.
    ///
    /// Returns `0` if all tests passed, `1` if any test failed.
    pub async fn run(self) -> i32 {
        let mode = self.options.mode.as_str();
        let mut suite = SuiteReport::new(
            self.options.report_json,
            self.options.report_file.clone(),
            mode,
        );

        for (name, test_fn) in self.tests {
            run_one_test(&mut suite, &name, mode, test_fn).await;
        }

        suite.finish();
        suite.emit();

        // Emit HTML report alongside the JSON file
        if let Some(json_path) = self.options.report_file.as_deref() {
            let html_path = Path::new(json_path).with_extension("html");
            if let Err(e) = uto_reporter::write_suite_html(suite.payload(), &html_path) {
                log::warn!("Failed to write HTML suite report: {e}");
            } else {
                log::info!("HTML suite report written to {}", html_path.display());
            }
        }

        if suite.payload().summary.failed > 0 {
            1
        } else {
            0
        }
    }
}

// ---------------------------------------------------------------------------
// Per-test execution
// ---------------------------------------------------------------------------

async fn run_one_test(suite: &mut SuiteReport, name: &str, mode: &str, test_fn: BoxedTestFn) {
    let mut handle = suite.begin_test(name);
    let report_events: SharedEvents = Arc::new(Mutex::new(Vec::new()));
    log::info!("Suite: starting test '{name}'");

    match start_new_session_with_hint_and_events(mode, 0, Some(Arc::clone(&report_events))).await {
        Ok(session) => {
            handle.event("session.start", "ok", serde_json::json!({ "target": mode }));

            match test_fn(session).await {
                Ok(_) => {
                    if let Ok(events) = report_events.lock() {
                        for event in events.iter().cloned() {
                            handle.event(&event.stage, &event.status, event.detail.clone());
                        }
                    }
                    handle.event(
                        "test.result",
                        "ok",
                        serde_json::json!({ "outcome": "passed" }),
                    );
                    log::info!("Suite: '{}' passed", name);
                    suite.record_test(handle, "passed", None);
                }
                Err(err) => {
                    let msg = err.to_string();
                    if let Ok(events) = report_events.lock() {
                        for event in events.iter().cloned() {
                            handle.event(&event.stage, &event.status, event.detail.clone());
                        }
                    }
                    handle.event(
                        "test.result",
                        "failed",
                        serde_json::json!({ "error": &msg }),
                    );
                    log::error!("Suite: '{}' failed — {msg}", name);
                    suite.record_test(handle, "failed", Some(msg));
                }
            }
        }
        Err(err) => {
            let msg = err.to_string();
            if let Ok(events) = report_events.lock() {
                for event in events.iter().cloned() {
                    handle.event(&event.stage, &event.status, event.detail.clone());
                }
            }
            handle.event(
                "session.start",
                "failed",
                serde_json::json!({ "error": &msg }),
            );
            log::error!("Suite: '{}' could not start session — {msg}", name);
            suite.record_test(handle, "failed", Some(msg));
        }
    }
}
