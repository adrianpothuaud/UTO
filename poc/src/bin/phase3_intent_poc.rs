//! # Phase 3 POC — Intent API on Web and Mobile
//!
//! This binary demonstrates the first intent-style APIs added in Phase 3:
//! `select(label)`, `click_intent(label)`, and `fill_intent(label, value)`.
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
//! # JSON report to stdout
//! UTO_REPORT_FORMAT=json cargo run -p uto-poc --bin phase3_intent_poc -- --json
//!
//! # JSON report to file
//! UTO_REPORT_FORMAT=json UTO_REPORT_FILE=./phase3-report.json cargo run -p uto-poc --bin phase3_intent_poc
//!
//! # JSON + HTML report artifacts
//! UTO_REPORT_FORMAT=json UTO_REPORT_FILE=./phase3-report.json UTO_REPORT_HTML=1 cargo run -p uto-poc --bin phase3_intent_poc -- --html
//! ```

use uto_core::{
    driver,
    env::{
        mobile_setup::{prepare_mobile_environment, MobileSetupOptions},
        platform::find_chrome_version,
        provisioning::find_or_provision_chromedriver,
    },
    error::UtoError,
    session::{
        mobile::{MobileCapabilities, MobileSession},
        web::WebSession,
        UtoSession,
    },
};

use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

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

struct ReportCollector {
    format: ReportFormat,
    report_file: Option<String>,
    payload: Value,
    start_ms: u128,
}

impl ReportCollector {
    fn new(options: &CliOptions) -> Self {
        let start_ms = now_unix_ms();
        let run_id = format!("run-{}-{}", start_ms, std::process::id());

        Self {
            format: options.report_format,
            report_file: options.report_file.clone(),
            payload: json!({
                "schema_version": "uto-report/v1",
                "framework": "uto",
                "phase": 3,
                "run_id": run_id,
                "mode": options.demo_mode.as_str(),
                "demo": options.demo_mode.as_str(),
                "status": "running",
                "timeline": {
                    "started_at_unix_ms": start_ms,
                    "finished_at_unix_ms": null,
                    "duration_ms": null
                },
                "events": []
            }),
            start_ms,
        }
    }

    fn enabled(&self) -> bool {
        matches!(self.format, ReportFormat::Json)
    }

    fn event(&mut self, stage: &str, status: &str, detail: Value) {
        if !self.enabled() {
            return;
        }

        if let Some(events) = self.payload.get_mut("events").and_then(Value::as_array_mut) {
            events.push(json!({
                "stage": stage,
                "status": status,
                "detail": detail
            }));
        }
    }

    fn finish(&mut self, status: &str, error: Option<String>) {
        if !self.enabled() {
            return;
        }

        let finish_ms = now_unix_ms();
        self.payload["status"] = json!(status);
        self.payload["timeline"]["finished_at_unix_ms"] = json!(finish_ms);
        self.payload["timeline"]["duration_ms"] = json!((finish_ms - self.start_ms) as u64);
        if let Some(message) = error {
            self.payload["error"] = json!(message);
        }
    }

    fn add_artifact(&mut self, name: &str, path: &str, kind: &str) {
        if !self.enabled() {
            return;
        }

        if self.payload.get("artifacts").is_none() {
            self.payload["artifacts"] = json!([]);
        }

        if let Some(artifacts) = self
            .payload
            .get_mut("artifacts")
            .and_then(Value::as_array_mut)
        {
            artifacts.push(json!({
                "name": name,
                "path": path,
                "kind": kind
            }));
        }
    }

    fn emit(&self) {
        if !self.enabled() {
            return;
        }

        match serde_json::to_string_pretty(&self.payload) {
            Ok(json_output) => {
                println!("{json_output}");
                if let Some(path) = &self.report_file {
                    if let Err(e) = std::fs::write(path, &json_output) {
                        log::error!("Failed to write JSON report to {}: {}", path, e);
                    } else {
                        log::info!("JSON report written to {}", path);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to serialize JSON report: {e}");
            }
        }
    }
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

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[tokio::main]
async fn main() {
    let _ = uto_logger::init("phase3-poc");

    let options = parse_cli_options();
    let mut report = ReportCollector::new(&options);

    report.event(
        "run.start",
        "ok",
        json!({
            "demo": options.demo_mode.as_str(),
            "report_format": match options.report_format {
                ReportFormat::Text => "text",
                ReportFormat::Json => "json",
            }
        }),
    );

    let result = match options.demo_mode {
        DemoMode::Mobile => run_mobile_intent_demo(&mut report).await,
        DemoMode::Web => run_web_intent_demo(&mut report).await,
    };

    match &result {
        Ok(_) => report.finish("passed", None),
        Err(e) => report.finish("failed", Some(e.to_string())),
    }

    if let Some(path) = &options.report_file {
        report.add_artifact("run_report", path, "json_report");
    }

    report.emit();

    if options.report_html {
        match serde_json::from_value::<uto_reporter::UtoReportV1>(report.payload.clone()) {
            Ok(parsed) => {
                let html_path = if let Some(path) = options.report_html_file.as_deref() {
                    std::path::PathBuf::from(path)
                } else if let Some(path) = options.report_file.as_deref() {
                    std::path::Path::new(path).with_extension("html")
                } else {
                    std::path::PathBuf::from("./phase3-report.html")
                };

                if let Err(err) = uto_reporter::write_report_html(&parsed, &html_path) {
                    log::error!("Failed to write HTML report {}: {}", html_path.display(), err);
                } else {
                    log::info!("HTML report written to {}", html_path.display());
                }
            }
            Err(err) => {
                log::error!("Failed to parse report payload for HTML export: {err}");
            }
        }
    }

    if let Err(e) = result {
        log::error!("Phase 3 demo failed: {e}");
        std::process::exit(1);
    }
}

async fn run_web_intent_demo(report: &mut ReportCollector) -> uto_core::error::UtoResult<()> {
    log::info!("=== UTO Phase 3 — Web Intent Demo ===");

    let chrome_version = match find_chrome_version() {
        Ok(v) => v,
        Err(e) => {
            log::error!("Chrome discovery failed: {e}");
            report.event(
                "env.chrome_discovery",
                "failed",
                json!({ "error": e.to_string() }),
            );
            return Err(e);
        }
    };
    report.event(
        "env.chrome_discovery",
        "ok",
        json!({ "chrome_version": chrome_version }),
    );

    let chromedriver_path = match find_or_provision_chromedriver(&chrome_version).await {
        Ok(p) => p,
        Err(e) => {
            log::error!("ChromeDriver provisioning failed: {e}");
            report.event(
                "env.chromedriver_provision",
                "failed",
                json!({ "error": e.to_string() }),
            );
            return Err(e);
        }
    };
    report.event(
        "env.chromedriver_provision",
        "ok",
        json!({ "path": chromedriver_path.display().to_string() }),
    );

    let driver_proc = match driver::start_chromedriver(&chromedriver_path).await {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to start ChromeDriver: {e}");
            report.event(
                "driver.chromedriver_start",
                "failed",
                json!({ "error": e.to_string() }),
            );
            return Err(e);
        }
    };
    report.event(
        "driver.chromedriver_start",
        "ok",
        json!({ "url": driver_proc.url, "port": driver_proc.port }),
    );

    let mut run_result = Ok(());
    match WebSession::new(&driver_proc.url).await {
        Ok(session) => {
            report.event(
                "session.web_create",
                "ok",
                json!({ "driver_url": driver_proc.url }),
            );

            let result = web_intent_interaction(&session, report).await;
            if let Err(e) = result {
                log::error!("Web intent interaction error: {e}");
                run_result = Err(e);
            }
            if let Err(e) = Box::new(session).close().await {
                log::warn!("Session close error: {e}");
                report.event(
                    "session.web_close",
                    "failed",
                    json!({ "error": e.to_string() }),
                );
            } else {
                report.event("session.web_close", "ok", json!({}));
            }
        }
        Err(e) => {
            log::error!("Session creation failed: {e}");
            report.event(
                "session.web_create",
                "failed",
                json!({ "error": e.to_string() }),
            );
            run_result = Err(e);
        }
    }

    if let Err(e) = driver_proc.stop() {
        log::warn!("ChromeDriver stop error: {e}");
        report.event(
            "driver.chromedriver_stop",
            "failed",
            json!({ "error": e.to_string() }),
        );
    } else {
        report.event("driver.chromedriver_stop", "ok", json!({}));
    }

    run_result
}

async fn web_intent_interaction(
    session: &WebSession,
    report: &mut ReportCollector,
) -> uto_core::error::UtoResult<()> {
    session
        .goto(concat!(
            "data:text/html,",
            "<html><body>",
            "<h1>Phase 3 Intent Demo</h1>",
            "<input id='email' aria-label='Email Address' type='text'/>",
            "<button id='cancel' aria-label='Cancel Action' onclick=\"document.getElementById('out').textContent='cancel'\">Cancel</button>",
            "<button id='submit' aria-label='Submit Order' onclick=\"document.getElementById('out').textContent=document.getElementById('email').value\">Submit</button>",
            "<p id='out'>pending</p>",
            "</body></html>"
        ))
        .await?;
    report.event(
        "session.goto",
        "ok",
        json!({ "target": "inline_intent_fixture" }),
    );

    let fill_ranking = session.debug_select_ranking("Email Address", 5).await?;
    log::info!("Intent ranking for 'Email Address': {fill_ranking}");
    report.event(
        "intent.rank",
        "ok",
        json!({ "intent": "Email Address", "ranking": fill_ranking }),
    );

    session
        .fill_intent("Email Address", "phase3@uto.dev")
        .await?;
    report.event(
        "intent.fill",
        "ok",
        json!({ "intent": "Email Address", "value": "phase3@uto.dev" }),
    );

    let click_ranking = session.debug_select_ranking("Submit Order", 5).await?;
    log::info!("Intent ranking for 'Submit Order': {click_ranking}");
    report.event(
        "intent.rank",
        "ok",
        json!({ "intent": "Submit Order", "ranking": click_ranking }),
    );

    session.click_intent("Submit Order").await?;
    report.event("intent.click", "ok", json!({ "intent": "Submit Order" }));

    let out = session.find_element("#out").await?;
    let out_text = session.get_text(&out).await?;
    log::info!("Intent output text: '{out_text}'");
    report.event(
        "assert.output",
        if out_text == "phase3@uto.dev" {
            "ok"
        } else {
            "failed"
        },
        json!({ "expected": "phase3@uto.dev", "actual": out_text }),
    );

    if out_text == "phase3@uto.dev" {
        log::info!("Web intent POC succeeded");
    } else {
        log::warn!("Web intent POC produced unexpected output: '{out_text}'");
        return Err(UtoError::SessionCommandFailed(format!(
            "Web Phase 3 intent objective failed: expected phase3@uto.dev, got {out_text}"
        )));
    }

    Ok(())
}

async fn run_mobile_intent_demo(report: &mut ReportCollector) -> uto_core::error::UtoResult<()> {
    log::info!("=== UTO Phase 3 — Mobile Intent Demo ===");

    let setup_options = MobileSetupOptions {
        require_online_device: true,
        ..MobileSetupOptions::default()
    };

    let mobile_setup = match prepare_mobile_environment(&setup_options) {
        Ok(result) => result,
        Err(e) => {
            log::error!("Mobile setup failed: {e}");
            report.event(
                "env.mobile_setup",
                "failed",
                json!({ "error": e.to_string() }),
            );
            return Err(e);
        }
    };
    report.event(
        "env.mobile_setup",
        "ok",
        json!({
            "android_sdk_root": mobile_setup.android_sdk.root.display().to_string(),
            "appium_path": mobile_setup.appium_path.display().to_string(),
            "device_serial": mobile_setup.device_serial
        }),
    );

    let driver_proc = match driver::start_appium(&mobile_setup.appium_path).await {
        Ok(p) => p,
        Err(e) => {
            log::error!("Failed to start Appium: {e}");
            report.event(
                "driver.appium_start",
                "failed",
                json!({ "error": e.to_string() }),
            );
            return Err(e);
        }
    };
    report.event(
        "driver.appium_start",
        "ok",
        json!({ "url": driver_proc.url, "port": driver_proc.port }),
    );

    let device_name = mobile_setup
        .device_serial
        .unwrap_or_else(|| "emulator-5554".to_string());

    let caps = MobileCapabilities::android(device_name);

    let mut run_result = Ok(());
    match MobileSession::new(&driver_proc.url, caps).await {
        Ok(session) => {
            report.event(
                "session.mobile_create",
                "ok",
                json!({ "driver_url": driver_proc.url }),
            );

            let result = mobile_intent_interaction(&session, report).await;
            if let Err(e) = result {
                log::error!("Mobile intent interaction error: {e}");
                run_result = Err(e);
            }
            if let Err(e) = Box::new(session).close().await {
                log::warn!("Session close error: {e}");
                report.event(
                    "session.mobile_close",
                    "failed",
                    json!({ "error": e.to_string() }),
                );
            } else {
                report.event("session.mobile_close", "ok", json!({}));
            }
        }
        Err(e) => {
            log::error!("Mobile session creation failed: {e}");
            report.event(
                "session.mobile_create",
                "failed",
                json!({ "error": e.to_string() }),
            );
            run_result = Err(e);
        }
    }

    if let Err(e) = driver_proc.stop() {
        log::warn!("Appium stop error: {e}");
        report.event(
            "driver.appium_stop",
            "failed",
            json!({ "error": e.to_string() }),
        );
    } else {
        report.event("driver.appium_stop", "ok", json!({}));
    }

    run_result
}

async fn mobile_intent_interaction(
    session: &MobileSession,
    report: &mut ReportCollector,
) -> uto_core::error::UtoResult<()> {
    session
        .launch_activity("com.android.settings", ".Settings")
        .await?;
    report.event(
        "session.launch_activity",
        "ok",
        json!({ "package": "com.android.settings", "activity": ".Settings" }),
    );

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Objective: mobile demo must resolve at least one search target.
    // Labels vary by Android version and locale, so we try several intents.
    let labels = [
        "Search settings",
        "Search",
        "Rechercher",
        "Buscar",
        "Suchen",
    ];

    for label in labels {
        match session.select(label).await {
            Ok(_) => {
                log::info!("Intent select('{label}') resolved on mobile");
                report.event(
                    "intent.select",
                    "ok",
                    json!({ "intent": label, "strategy": "label" }),
                );
                return Ok(());
            }
            Err(e) => {
                log::warn!("Intent select('{label}') not resolved on this device/locale: {e}");
                report.event(
                    "intent.select",
                    "failed",
                    json!({ "intent": label, "strategy": "label", "error": e.to_string() }),
                );
            }
        }
    }

    // Fallback: common Android Settings search resource IDs across versions.
    let fallback_selectors = [
        "//*[@resource-id='com.android.settings:id/search_action_bar']",
        "//*[@resource-id='com.android.settings:id/search']",
        "//*[@resource-id='com.android.settings.intelligence:id/open_search_view_edit_text']",
    ];

    for selector in fallback_selectors {
        match session.find_element(selector).await {
            Ok(_) => {
                log::info!("Mobile search target resolved via fallback selector: {selector}");
                report.event(
                    "intent.select",
                    "ok",
                    json!({ "strategy": "fallback_selector", "selector": selector }),
                );
                return Ok(());
            }
            Err(e) => {
                log::warn!("Fallback selector not resolved ({selector}): {e}");
                report.event(
                    "intent.select",
                    "failed",
                    json!({
                        "strategy": "fallback_selector",
                        "selector": selector,
                        "error": e.to_string()
                    }),
                );
            }
        }
    }

    Err(UtoError::SessionCommandFailed(
        "Mobile Phase 3 intent demo objective not met: no search target was resolved via intent labels or fallback selectors".to_string(),
    ))
}
