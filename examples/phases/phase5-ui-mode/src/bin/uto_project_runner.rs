//! Phase 5 UI-mode reference project runner.
//!
//! Demonstrates how a UTO project emits a `uto-suite/v1` report that the
//! `uto ui` server can load and replay interactively.
//!
//! Run directly:
//!   cargo run --bin uto_project_runner -- --target web --json \
//!     --report-file .uto/reports/last-run.json
//!
//! Or via the CLI (and then launch the UI against the saved report):
//!   uto run --project . --target web
//!   uto ui --project . --report .uto/reports/last-run.json --open

use uto_runner::{CliOptions, RunMode};
use uto_test::Suite;

#[tokio::main]
async fn main() {
    let _ = uto_logger::init("phase5-ui-mode");
    let options = CliOptions::from_env();
    let mode = options.mode;

    let suite = Suite::new(options);
    let exit_code = match mode {
        RunMode::Web => build_web_suite(suite).run().await,
        RunMode::Mobile => build_mobile_suite(suite).run().await,
    };

    std::process::exit(exit_code);
}

fn build_web_suite(suite: Suite) -> Suite {
    suite
        .test("web: page load", |_opts| async {
            // This test demonstrates graceful skip when Chrome is unavailable.
            // In a real environment with Chrome installed this would open a browser session.
            log::info!("Checking for Chrome availability…");
            if which::which("chromedriver").is_err() {
                log::info!("chromedriver not found — skipping web test (graceful skip)");
                return Ok(());
            }
            Ok(())
        })
        .test("web: navigation smoke", |_opts| async {
            log::info!("Navigation smoke test placeholder");
            Ok(())
        })
}

fn build_mobile_suite(suite: Suite) -> Suite {
    suite.test("mobile: device check", |_opts| async {
        log::info!("Mobile device check placeholder (graceful skip when Appium unavailable)");
        Ok(())
    })
}
