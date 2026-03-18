use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

use uto_core::{
    driver,
    error::UtoError,
    env::{
        mobile_setup::{prepare_mobile_environment, MobileSetupOptions},
        platform::find_chrome_version,
        provisioning::find_or_provision_chromedriver,
    },
    session::{
        mobile::{MobileCapabilities, MobileSession},
        web::WebSession,
        UtoSession,
    },
};

#[derive(Clone, Copy)]
enum DemoMode {
    Web,
    Mobile,
}

struct CliOptions {
    demo_mode: DemoMode,
    report_json: bool,
    report_file: Option<String>,
}

struct Report {
    enabled: bool,
    report_file: Option<String>,
    payload: Value,
    start_ms: u128,
}

impl Report {
    fn new(options: &CliOptions) -> Self {
        let start_ms = now_unix_ms();
        let run_id = format!("run-{}-{}", start_ms, std::process::id());

        Self {
            enabled: options.report_json,
            report_file: options.report_file.clone(),
            payload: json!({
                "schema_version": "uto-report/v1",
                "framework": "uto",
                "phase": 3,
                "run_id": run_id,
                "demo": match options.demo_mode {
                    DemoMode::Web => "web",
                    DemoMode::Mobile => "mobile",
                },
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

    fn event(&mut self, stage: &str, status: &str, detail: Value) {
        if !self.enabled {
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
        if !self.enabled {
            return;
        }

        let end_ms = now_unix_ms();
        self.payload["status"] = json!(status);
        self.payload["timeline"]["finished_at_unix_ms"] = json!(end_ms);
        self.payload["timeline"]["duration_ms"] = json!((end_ms - self.start_ms) as u64);

        if let Some(err) = error {
            self.payload["error"] = json!(err);
        }
    }

    fn emit(&self) {
        if !self.enabled {
            return;
        }

        let serialized = match serde_json::to_string_pretty(&self.payload) {
            Ok(value) => value,
            Err(err) => {
                eprintln!("Failed to serialize report: {err}");
                return;
            }
        };

        println!("{serialized}");

        if let Some(path) = &self.report_file {
            if let Err(err) = std::fs::write(path, serialized) {
                eprintln!("Failed to write report file {}: {}", path, err);
            }
        }
    }
}

fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

fn parse_cli_options() -> CliOptions {
    let mut demo_mode = DemoMode::Web;
    let mut report_json = false;
    let mut report_file = std::env::var("UTO_REPORT_FILE").ok();

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--target" => {
                if let Some(next) = args.get(i + 1) {
                    if next.eq_ignore_ascii_case("mobile") {
                        demo_mode = DemoMode::Mobile;
                    }
                    i += 1;
                }
            }
            "--json" => report_json = true,
            "--report-file" => {
                if let Some(next) = args.get(i + 1) {
                    report_file = Some(next.clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    CliOptions {
        demo_mode,
        report_json,
        report_file,
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let options = parse_cli_options();
    let mut report = Report::new(&options);

    let result = match options.demo_mode {
        DemoMode::Web => run_web(&mut report).await,
        DemoMode::Mobile => run_mobile(&mut report).await,
    };

    match &result {
        Ok(_) => report.finish("passed", None),
        Err(err) => report.finish("failed", Some(err.to_string())),
    }

    report.emit();

    if let Err(err) = result {
        eprintln!("Runner failed: {err}");
        std::process::exit(1);
    }
}

async fn run_web(report: &mut Report) -> uto_core::error::UtoResult<()> {
    let chrome_version = find_chrome_version()?;
    report.event(
        "env.chrome_discovery",
        "ok",
        json!({ "chrome_version": chrome_version }),
    );

    let chromedriver = find_or_provision_chromedriver(&chrome_version).await?;
    report.event(
        "env.chromedriver_provision",
        "ok",
        json!({ "path": chromedriver.display().to_string() }),
    );

    let driver = driver::start_chromedriver(&chromedriver).await?;
    report.event(
        "driver.chromedriver_start",
        "ok",
        json!({ "url": driver.url, "port": driver.port }),
    );

    let mut run_result = Ok(());
    match WebSession::new_with_args(
        &driver.url,
        &["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"],
    )
    .await
    {
        Ok(session) => {
            report.event("session.web_create", "ok", json!({ "driver_url": driver.url }));

            let action_result = web_intent_flow(&session, report).await;
            if let Err(err) = action_result {
                run_result = Err(err);
            }

            let _ = Box::new(session).close().await;
        }
        Err(err) => {
            report.event(
                "session.web_create",
                "failed",
                json!({ "error": err.to_string() }),
            );
            run_result = Err(err);
        }
    }

    let _ = driver.stop();
    report.event("driver.chromedriver_stop", "ok", json!({}));

    run_result
}

async fn web_intent_flow(session: &WebSession, report: &mut Report) -> uto_core::error::UtoResult<()> {
    session
        .goto(concat!(
            "data:text/html,",
            "<html><body>",
            "<input id='email' aria-label='Email Address' type='text'/>",
            "<button id='submit' aria-label='Submit Order' ",
            "onclick=\"document.getElementById('out').textContent=document.getElementById('email').value\">",
            "Submit</button>",
            "<p id='out'>initial</p>",
            "</body></html>"
        ))
        .await?;

    report.event("session.goto", "ok", json!({ "target": "data:text/html" }));

    let ranking = session
        .debug_select_ranking("Submit Order", 3)
        .await
        .unwrap_or_else(|_| "<unavailable>".to_string());
    report.event(
        "intent.ranking",
        "ok",
        json!({ "label": "Submit Order", "summary": ranking }),
    );

    session.fill_intent("Email Address", "phase3@uto.dev").await?;
    report.event(
        "intent.fill",
        "ok",
        json!({ "label": "Email Address", "value": "phase3@uto.dev" }),
    );

    session.click_intent("Submit Order").await?;
    report.event(
        "intent.click",
        "ok",
        json!({ "label": "Submit Order" }),
    );

    let output = session.find_element("#out").await?;
    let text = session.get_text(&output).await?;

    if text != "phase3@uto.dev" {
        return Err(UtoError::SessionCommandFailed(format!(
            "Phase 3 web objective mismatch: expected 'phase3@uto.dev', got '{text}'"
        )));
    }

    report.event("assert.output", "ok", json!({ "text": text }));
    Ok(())
}

async fn run_mobile(report: &mut Report) -> uto_core::error::UtoResult<()> {
    let setup = prepare_mobile_environment(&MobileSetupOptions {
        require_online_device: true,
        ..MobileSetupOptions::default()
    })?;

    report.event(
        "env.mobile_setup",
        "ok",
        json!({
            "android_sdk_root": setup.android_sdk.root.display().to_string(),
            "appium_path": setup.appium_path.display().to_string(),
            "device_serial": setup.device_serial
        }),
    );

    let appium = driver::start_appium(&setup.appium_path).await?;
    report.event(
        "driver.appium_start",
        "ok",
        json!({ "url": appium.url, "port": appium.port }),
    );

    let caps = MobileCapabilities::android(
        setup
            .device_serial
            .unwrap_or_else(|| "emulator-5554".to_string()),
    );

    let mut run_result = Ok(());
    match MobileSession::new(&appium.url, caps).await {
        Ok(session) => {
            report.event(
                "session.mobile_create",
                "ok",
                json!({ "driver_url": appium.url }),
            );

            let action_result = mobile_intent_flow(&session, report).await;
            if let Err(err) = action_result {
                run_result = Err(err);
            }

            let _ = Box::new(session).close().await;
        }
        Err(err) => {
            report.event(
                "session.mobile_create",
                "failed",
                json!({ "error": err.to_string() }),
            );
            run_result = Err(err);
        }
    }

    let _ = appium.stop();
    report.event("driver.appium_stop", "ok", json!({}));

    run_result
}

async fn mobile_intent_flow(
    session: &MobileSession,
    report: &mut Report,
) -> uto_core::error::UtoResult<()> {
    session
        .launch_activity("com.android.settings", ".Settings")
        .await?;
    report.event(
        "session.launch_activity",
        "ok",
        json!({ "activity": "Settings" }),
    );

    for label in ["Search settings", "Search", "Rechercher", "Buscar", "Suchen"] {
        match session.select(label).await {
            Ok(_) => {
                report.event(
                    "intent.select",
                    "ok",
                    json!({ "label": label, "strategy": "mobile-accessibility" }),
                );
                return Ok(());
            }
            Err(err) => {
                report.event(
                    "intent.select",
                    "failed",
                    json!({ "label": label, "error": err.to_string() }),
                );
            }
        }
    }

    Err(UtoError::SessionCommandFailed(
        "Phase 3 mobile objective failed: no search label resolved".to_string(),
    ))
}
