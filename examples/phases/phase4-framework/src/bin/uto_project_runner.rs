//! Phase 4 framework reference project runner.
//!
//! Demonstrates the Suite API: multiple named test cases execute in sequence,
//! each with an isolated session. Results aggregate into a `uto-suite/v1` JSON
//! and HTML report.
//!
//! Run via the UTO CLI:
//!   uto run --project . --target web
//!
//! Or directly:
//!   cargo run --bin uto_project_runner -- --target web --json \
//!     --report-file .uto/reports/last-run.json

use phase4_framework::{build_mobile_suite, build_web_suite};
use uto_runner::{CliOptions, RunMode};
use uto_test::Suite;

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let _ = uto_logger::init("phase4-framework");
    let options = CliOptions::from_env();
    let mode = options.mode;

    let suite = Suite::new(options);
    let exit_code = match mode {
        RunMode::Web => build_web_suite(suite).run().await,
        RunMode::Mobile => build_mobile_suite(suite).run().await,
    };

    std::process::exit(exit_code);
}
