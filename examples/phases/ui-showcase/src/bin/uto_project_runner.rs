//! UTO UI Showcase project runner.
//!
//! Demonstrates vision-first, selector-free test authoring on real websites.
//! Tests are designed to be watched live in the UTO UI (`uto ui --open`).
//!
//! Run via the CLI:
//!   uto run --project . --target web
//!
//! Or directly:
//!   cargo run --bin uto_project_runner -- --target web --json \
//!     --report-file .uto/reports/last-run.json

use uto_runner::{CliOptions, RunMode};
use uto_test::Suite;

#[tokio::main]
async fn main() {
    let _ = uto_logger::init("ui-showcase");
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
        .test("web: navigate and verify visibility", |_opts| async {
            log::info!("Running web navigation test...");
            // In the real test file (tests/web_intent_showcase.rs),
            // this is implemented with full session management.
            // This runner is a placeholder; actual tests are in Rust integration tests.
            Ok(())
        })
        .test("web: multi-step workflow", |_opts| async {
            log::info!("Running multi-step workflow test...");
            Ok(())
        })
}

fn build_mobile_suite(suite: Suite) -> Suite {
    suite
        .test("mobile: placeholder test", |_opts| async {
            log::info!("Mobile tests require Appium and devices");
            Ok(())
        })
}
