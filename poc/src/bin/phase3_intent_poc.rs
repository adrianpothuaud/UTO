//! # Phase 3 POC — Intent API on Web and Mobile
//!
//! This binary demonstrates the Phase 3 intent APIs as a compact multi-scenario
//! suite. Each selected mode runs several representative scenarios and can emit
//! both `uto-suite/v1` JSON and native HTML.
//!
//! ## Usage
//!
//! ```sh
//! # Web intent demo (default)
//! cargo run -p uto-poc --bin phase3_intent_poc
//!
//! # Mobile intent demo
//! UTO_DEMO=mobile cargo run -p uto-poc --bin phase3_intent_poc
//!
//! # JSON suite report to stdout
//! UTO_REPORT_FORMAT=json cargo run -p uto-poc --bin phase3_intent_poc -- --json
//!
//! # JSON suite report to file
//! UTO_REPORT_FORMAT=json UTO_REPORT_FILE=./phase3-report.json cargo run -p uto-poc --bin phase3_intent_poc
//!
//! # JSON + HTML suite report artifacts
//! UTO_REPORT_FORMAT=json UTO_REPORT_FILE=./phase3-report.json UTO_REPORT_HTML=1 cargo run -p uto-poc --bin phase3_intent_poc -- --html
//! ```

#[path = "../phase3_intent/mod.rs"]
mod phase3_intent;

#[derive(Clone, Copy)]
enum DemoMode {
    Web,
    Mobile,
}

impl DemoMode {
    fn as_str(self) -> &'static str {
        match self {
            DemoMode::Web => "web",
            DemoMode::Mobile => "mobile",
        }
    }
}

#[derive(Clone, Copy)]
enum ReportFormat {
    Text,
    Json,
}

struct CliOptions {
    demo_mode: DemoMode,
    report_format: ReportFormat,
    report_file: Option<String>,
    report_html: bool,
    report_html_file: Option<String>,
}

fn parse_cli_options() -> CliOptions {
    let mut demo_mode = match std::env::var("UTO_DEMO") {
        Ok(v) if v.eq_ignore_ascii_case("mobile") => DemoMode::Mobile,
        _ => DemoMode::Web,
    };

    let mut report_format = match std::env::var("UTO_REPORT_FORMAT") {
        Ok(v) if v.eq_ignore_ascii_case("json") => ReportFormat::Json,
        _ => ReportFormat::Text,
    };

    let mut report_file = std::env::var("UTO_REPORT_FILE").ok();
    let mut report_html = match std::env::var("UTO_REPORT_HTML") {
        Ok(v) => v == "1" || v.eq_ignore_ascii_case("true"),
        Err(_) => false,
    };
    let mut report_html_file = std::env::var("UTO_REPORT_HTML_FILE").ok();

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--mobile" => demo_mode = DemoMode::Mobile,
            "--web" => demo_mode = DemoMode::Web,
            "--json" => report_format = ReportFormat::Json,
            "--html" => report_html = true,
            "--report-file" => {
                if let Some(next) = args.get(i + 1) {
                    report_file = Some(next.clone());
                    i += 1;
                }
            }
            "--html-file" => {
                if let Some(next) = args.get(i + 1) {
                    report_html_file = Some(next.clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    CliOptions {
        demo_mode,
        report_format,
        report_file,
        report_html,
        report_html_file,
    }
}

#[tokio::main]
async fn main() {
    let _ = uto_logger::init("phase3-poc");

    let options = parse_cli_options();
    let report_json = matches!(options.report_format, ReportFormat::Json);
    let mut suite = uto_reporter::SuiteReport::new(
        report_json,
        options.report_file.clone(),
        options.demo_mode.as_str(),
    );

    let result = match options.demo_mode {
        DemoMode::Mobile => phase3_intent::mobile::run_suite(&mut suite).await,
        DemoMode::Web => phase3_intent::web::run_suite(&mut suite).await,
    };

    suite.finish();
    suite.emit();

    if options.report_html {
        let html_path = if let Some(path) = options.report_html_file.as_deref() {
            std::path::PathBuf::from(path)
        } else if let Some(path) = options.report_file.as_deref() {
            std::path::Path::new(path).with_extension("html")
        } else {
            std::path::PathBuf::from("./phase3-report.html")
        };

        if let Err(err) = uto_reporter::write_suite_html(suite.payload(), &html_path) {
            log::error!(
                "Failed to write HTML report {}: {}",
                html_path.display(),
                err
            );
        } else {
            log::info!("HTML report written to {}", html_path.display());
        }
    }

    if let Err(err) = result {
        log::error!("Phase 3 demo failed: {err}");
        std::process::exit(1);
    }
}
