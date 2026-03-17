use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct UtoProjectConfig {
    schema_version: String,
    project_name: String,
    tests_dir: String,
    default_target: String,
    report_dir: String,
    uto_root: String,
    report_schema: String,
}

fn main() {
    if let Err(e) = run_cli() {
        eprintln!("uto CLI error: {e}");
        std::process::exit(1);
    }
}

fn run_cli() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        return Ok(());
    }

    match args[1].as_str() {
        "init" => cmd_init(&args[2..]),
        "run" => cmd_run(&args[2..]),
        "report" => cmd_report(&args[2..]),
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        other => Err(format!(
            "Unknown command '{other}'. Supported commands: init, run, report"
        )),
    }
}

fn print_help() {
    println!(
        "UTO CLI\n\nCommands:\n  uto init <project-dir> [--template web|mobile] [--uto-root <path>]\n  uto run --project <project-dir> [--target web|mobile] [--report-json <path>] [--driver-trace]\n  uto report --project <project-dir> [--input <report-path>]"
    );
}

fn cmd_init(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err("init requires <project-dir>".to_string());
    }

    let project_dir = PathBuf::from(&args[0]);
    let mut template = "web".to_string();
    let mut uto_root = std::env::current_dir().map_err(|e| e.to_string())?;

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--template" => {
                if let Some(next) = args.get(i + 1) {
                    template = next.clone();
                    i += 1;
                }
            }
            "--uto-root" => {
                if let Some(next) = args.get(i + 1) {
                    uto_root = PathBuf::from(next);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    fs::create_dir_all(project_dir.join("src/bin")).map_err(|e| e.to_string())?;
    fs::create_dir_all(project_dir.join("tests")).map_err(|e| e.to_string())?;
    fs::create_dir_all(project_dir.join(".uto/reports")).map_err(|e| e.to_string())?;

    let project_name = project_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("uto-project")
        .to_string();

    let config = UtoProjectConfig {
        schema_version: "1".to_string(),
        project_name: project_name.clone(),
        tests_dir: "tests".to_string(),
        default_target: template.clone(),
        report_dir: ".uto/reports".to_string(),
        uto_root: uto_root.display().to_string(),
        report_schema: "uto-report/v1".to_string(),
    };

    write_json(project_dir.join("uto.json"), &config)?;
    fs::write(
        project_dir.join("Cargo.toml"),
        generated_cargo_toml(&project_name, &uto_root),
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        project_dir.join("src/bin/uto_project_runner.rs"),
        generated_project_runner_rs(),
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        project_dir.join("tests/web_example.rs"),
        generated_web_test_example(),
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        project_dir.join("tests/mobile_example.rs"),
        generated_mobile_test_example(),
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        project_dir.join("README.md"),
        generated_project_readme(&config),
    )
    .map_err(|e| e.to_string())?;

    println!(
        "Created UTO template project at {} (template: {})",
        project_dir.display(),
        template
    );
    println!("Next: uto run --project {}", project_dir.display());
    Ok(())
}

fn cmd_run(args: &[String]) -> Result<(), String> {
    let mut project: Option<PathBuf> = None;
    let mut target: Option<String> = None;
    let mut report_json: Option<PathBuf> = None;
    let mut driver_trace = false;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--project" => {
                if let Some(next) = args.get(i + 1) {
                    project = Some(PathBuf::from(next));
                    i += 1;
                }
            }
            "--target" => {
                if let Some(next) = args.get(i + 1) {
                    target = Some(next.clone());
                    i += 1;
                }
            }
            "--report-json" => {
                if let Some(next) = args.get(i + 1) {
                    report_json = Some(PathBuf::from(next));
                    i += 1;
                }
            }
            "--driver-trace" => {
                driver_trace = true;
            }
            _ => {}
        }
        i += 1;
    }

    let project = project.ok_or_else(|| "run requires --project <project-dir>".to_string())?;
    let config: UtoProjectConfig = read_json(project.join("uto.json"))?;

    let effective_target = target.unwrap_or(config.default_target.clone());
    let report_path =
        report_json.unwrap_or_else(|| project.join(config.report_dir).join("last-run.json"));

    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut cmd = Command::new("cargo");
    cmd.current_dir(&project)
        .arg("run")
        .arg("--bin")
        .arg("uto_project_runner")
        .arg("--")
        .arg("--json")
        .arg("--target")
        .arg(&effective_target)
        .arg("--report-file")
        .arg(report_path.display().to_string())
        .env("RUST_LOG", "info");

    if driver_trace {
        // Reserved for upcoming driver-level payload tracing in runners.
        cmd.env("UTO_DRIVER_TRACE", "1");
    }

    let status = cmd.status().map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!(
            "Test run failed for target '{}' (status: {})",
            effective_target, status
        ));
    }

    println!("Run succeeded. Report: {}", report_path.display());
    Ok(())
}

fn cmd_report(args: &[String]) -> Result<(), String> {
    let mut project: Option<PathBuf> = None;
    let mut input: Option<PathBuf> = None;

    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--project" => {
                if let Some(next) = args.get(i + 1) {
                    project = Some(PathBuf::from(next));
                    i += 1;
                }
            }
            "--input" => {
                if let Some(next) = args.get(i + 1) {
                    input = Some(PathBuf::from(next));
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    let project = project.ok_or_else(|| "report requires --project <project-dir>".to_string())?;
    let config: UtoProjectConfig = read_json(project.join("uto.json"))?;

    let report_path =
        input.unwrap_or_else(|| project.join(config.report_dir).join("last-run.json"));
    let report_value: serde_json::Value = read_json(report_path.clone())?;

    let status = report_value
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let schema_version = report_value
        .get("schema_version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let run_id = report_value
        .get("run_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let demo = report_value
        .get("demo")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let duration_ms = report_value
        .get("timeline")
        .and_then(|t| t.get("duration_ms"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let events_count = report_value
        .get("events")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);

    println!("UTO Report Summary");
    println!("- Project: {}", config.project_name);
    println!("- Schema: {schema_version}");
    println!("- Run ID: {run_id}");
    println!("- Demo: {demo}");
    println!("- Status: {status}");
    println!("- Duration (ms): {duration_ms}");
    println!("- Events: {events_count}");
    println!("- File: {}", report_path.display());

    Ok(())
}

fn write_json<T: Serialize>(path: PathBuf, value: &T) -> Result<(), String> {
    let content = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, content).map_err(|e| e.to_string())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: impl AsRef<Path>) -> Result<T, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

fn generated_project_readme(config: &UtoProjectConfig) -> String {
    format!(
        "# {}\n\nGenerated by `uto init`.\n\nThis is a standalone UTO test project template with example tests and a local runner.\n\n## Quick Start\n\n```sh\n# Run default target\nuto run --project .\n\n# Run explicit target\nuto run --project . --target {} --report-json ./.uto/reports/last-run.json\n\n# Summarize report\nuto report --project .\n\n# Run template integration tests directly\ncargo test\n```\n\n## Files\n\n- `uto.json`: project config\n- `src/bin/uto_project_runner.rs`: local project runner used by `uto run`\n- `tests/web_example.rs`: web example test using uto-core\n- `tests/mobile_example.rs`: mobile example test using uto-core\n- `.uto/reports/`: structured run reports\n",
        config.project_name, config.default_target
    )
}

fn generated_cargo_toml(project_name: &str, uto_root: &Path) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n\n[dependencies]\nuto-core = {{ path = \"{}\" }}\ntokio = {{ version = \"1\", features = [\"full\"] }}\nlog = \"0.4\"\nenv_logger = \"0.11\"\nserde_json = \"1\"\nwhich = \"7\"\n\n",
        project_name,
        uto_root.join("uto-core").display()
    )
}

fn generated_project_runner_rs() -> String {
    r##"use serde_json::{json, Value};
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
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to serialize report: {e}");
                return;
            }
        };

        println!("{serialized}");
        if let Some(path) = &self.report_file {
            if let Err(e) = std::fs::write(path, serialized) {
                eprintln!("Failed to write report file {}: {}", path, e);
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
                    } else {
                        demo_mode = DemoMode::Web;
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
        Err(e) => report.finish("failed", Some(e.to_string())),
    }

    report.emit();

    if let Err(e) = result {
        eprintln!("Runner failed: {e}");
        std::process::exit(1);
    }
}

async fn run_web(report: &mut Report) -> uto_core::error::UtoResult<()> {
    let chrome_version = find_chrome_version()?;
    report.event("env.chrome_discovery", "ok", json!({ "chrome_version": chrome_version }));

    let chromedriver = find_or_provision_chromedriver(&chrome_version).await?;
    report.event("env.chromedriver_provision", "ok", json!({ "path": chromedriver.display().to_string() }));

    let driver = driver::start_chromedriver(&chromedriver).await?;
    report.event("driver.chromedriver_start", "ok", json!({ "url": driver.url, "port": driver.port }));

    let mut run_result = Ok(());
    match WebSession::new(&driver.url).await {
        Ok(session) => {
            report.event("session.web_create", "ok", json!({ "driver_url": driver.url }));
            let action = web_actions(&session, report).await;
            if let Err(e) = action {
                run_result = Err(e);
            }
            let _ = Box::new(session).close().await;
        }
        Err(e) => {
            report.event("session.web_create", "failed", json!({ "error": e.to_string() }));
            run_result = Err(e);
        }
    }

    let _ = driver.stop();
    report.event("driver.chromedriver_stop", "ok", json!({}));
    run_result
}

async fn web_actions(session: &WebSession, report: &mut Report) -> uto_core::error::UtoResult<()> {
    session.goto("https://example.com").await?;
    report.event("session.goto", "ok", json!({ "target": "https://example.com" }));

    let title = session.title().await?;
    report.event("assert.title", "ok", json!({ "title": title }));

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
    report.event("driver.appium_start", "ok", json!({ "url": appium.url, "port": appium.port }));

    let caps = MobileCapabilities::android(
        setup
            .device_serial
            .unwrap_or_else(|| "emulator-5554".to_string()),
    );

    let mut run_result = Ok(());
    match MobileSession::new(&appium.url, caps).await {
        Ok(session) => {
            report.event("session.mobile_create", "ok", json!({ "driver_url": appium.url }));
            let action = mobile_actions(&session, report).await;
            if let Err(e) = action {
                run_result = Err(e);
            }
            let _ = Box::new(session).close().await;
        }
        Err(e) => {
            report.event(
                "session.mobile_create",
                "failed",
                json!({ "error": e.to_string() }),
            );
            run_result = Err(e);
        }
    }

    let _ = appium.stop();
    report.event("driver.appium_stop", "ok", json!({}));
    run_result
}

async fn mobile_actions(
    session: &MobileSession,
    report: &mut Report,
) -> uto_core::error::UtoResult<()> {
    session
        .launch_activity("com.android.settings", ".Settings")
        .await?;
    report.event("session.launch_activity", "ok", json!({ "activity": "Settings" }));

    for label in ["Search settings", "Search", "Rechercher", "Buscar", "Suchen"] {
        match session.select(label).await {
            Ok(_) => {
                report.event(
                    "intent.select",
                    "ok",
                    json!({ "intent": label, "strategy": "label" }),
                );
                return Ok(());
            }
            Err(e) => {
                report.event(
                    "intent.select",
                    "failed",
                    json!({ "intent": label, "error": e.to_string() }),
                );
            }
        }
    }

    Err(UtoError::SessionCommandFailed(
        "Mobile template objective failed: no search label resolved".to_string(),
    ))
}
"##
    .to_string()
}

fn generated_web_test_example() -> String {
    r##"use uto_core::session::web::WebSession;
use uto_core::session::UtoSession;

#[tokio::test]
async fn web_example_navigates_example_dot_com_or_skips() {
    let chromedriver = match which::which("chromedriver") {
        Ok(path) => path,
        Err(_) => {
            println!("Skipping web example: chromedriver not available");
            return;
        }
    };

    let driver = match uto_core::driver::start_chromedriver(&chromedriver).await {
        Ok(p) => p,
        Err(err) => {
            println!("Skipping web example: could not start chromedriver: {err}");
            return;
        }
    };

    let session = WebSession::new(&driver.url).await.expect("web session");
    session.goto("https://example.com").await.expect("goto");
    let title = session.title().await.expect("title");
    assert!(!title.is_empty());

    Box::new(session).close().await.expect("close");
    driver.stop().expect("stop driver");
}
"##
    .to_string()
}

fn generated_mobile_test_example() -> String {
    r##"use uto_core::session::mobile::{MobileCapabilities, MobileSession};
use uto_core::session::UtoSession;

#[tokio::test]
async fn mobile_example_creates_session_or_skips() {
    let appium = match uto_core::env::platform::find_appium() {
        Some(path) => path,
        None => {
            println!("Skipping mobile example: appium not available");
            return;
        }
    };

    let appium_driver = match uto_core::driver::start_appium(&appium).await {
        Ok(p) => p,
        Err(err) => {
            println!("Skipping mobile example: could not start appium: {err}");
            return;
        }
    };

    let caps = MobileCapabilities::android("emulator-5554");
    match MobileSession::new(&appium_driver.url, caps).await {
        Ok(session) => {
            Box::new(session).close().await.expect("close mobile session");
        }
        Err(err) => {
            println!("Skipping mobile example: environment not ready: {err}");
        }
    }

    appium_driver.stop().expect("stop appium");
}
"##
    .to_string()
}
