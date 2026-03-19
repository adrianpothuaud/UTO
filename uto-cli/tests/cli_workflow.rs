use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("uto-cli should be in workspace root/uto-cli")
        .to_path_buf()
}

fn uto_bin() -> &'static str {
    env!("CARGO_BIN_EXE_uto")
}

fn run_uto(args: &[&str]) -> std::process::Output {
    Command::new(uto_bin())
        .args(args)
        .output()
        .expect("failed to run uto binary")
}

fn create_project(temp: &TempDir, name: &str) -> PathBuf {
    let project = temp.path().join(name);
    let root = workspace_root();
    let output = run_uto(&[
        "init",
        project.to_str().expect("project path utf-8"),
        "--template",
        "web",
        "--uto-root",
        root.to_str().expect("root path utf-8"),
    ]);
    assert!(
        output.status.success(),
        "init failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    project
}

fn set_framework_version(project: &Path, version: &str) {
    let config_path = project.join("uto.json");
    let config_raw = fs::read_to_string(&config_path).expect("read uto.json");
    let mut config: serde_json::Value =
        serde_json::from_str(&config_raw).expect("parse uto.json as JSON");
    config["framework_version"] = serde_json::Value::String(version.to_string());
    fs::write(
        &config_path,
        serde_json::to_string_pretty(&config).expect("serialize uto.json"),
    )
    .expect("write uto.json");
}

fn add_legacy_runner(project: &Path) {
    let bin_dir = project.join("src/bin");
    fs::create_dir_all(&bin_dir).expect("create src/bin");
    fs::write(
        bin_dir.join("uto_project_runner.rs"),
        r#"fn main() {
    let report_path = std::env::var("UTO_REPORT_JSON").expect("UTO_REPORT_JSON");

    let report = serde_json::json!({
        "schema_version": "uto-report/v1",
        "framework": "uto",
        "run_id": "legacy-run",
        "mode": "web",
        "status": "passed",
        "timeline": {
            "started_at_unix_ms": 1,
            "finished_at_unix_ms": 2,
            "duration_ms": 1
        },
        "events": []
    });

    std::fs::write(report_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
}
"#,
    )
    .expect("write legacy runner");
}

#[test]
fn init_scaffolds_expected_project_files() {
    let temp = TempDir::new().expect("temp dir");
    let project = create_project(&temp, "init-smoke");

    assert!(project.join("uto.json").exists());
    assert!(project.join("Cargo.toml").exists());
    assert!(project.join("README.md").exists());
    assert!(project.join("src/lib.rs").exists());
    assert!(!project.join("src/bin/uto_project_runner.rs").exists());
    assert!(project.join("tests/web_example.rs").exists());
    assert!(project.join("tests/mobile_example.rs").exists());

    let uto_json = fs::read_to_string(project.join("uto.json")).expect("read uto.json");
    assert!(uto_json.contains(r#""report_schema": "uto-suite/v1""#));
    assert!(uto_json.contains(r#""framework_version": "4.5""#));
}

#[test]
fn run_accepts_suite_schema_in_project_config() {
    let temp = TempDir::new().expect("temp dir");
    let project = create_project(&temp, "run-suite-schema");

    let output = run_uto(&[
        "run",
        "--project",
        project.to_str().expect("project path utf-8"),
        "--target",
        "web",
    ]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Unsupported report_schema"),
        "stderr: {stderr}"
    );
}

#[test]
fn run_works_without_project_runner_binary() {
    let temp = TempDir::new().expect("temp dir");
    let project = create_project(&temp, "run-runnerless");

    let output = run_uto(&[
        "run",
        "--project",
        project.to_str().expect("project path utf-8"),
        "--target",
        "web",
    ]);

    assert!(
        output.status.success(),
        "run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(project.join(".uto/reports/last-run.json").exists());
    assert!(project.join(".uto/reports/last-run.html").exists());
}

#[test]
fn legacy_runner_warns_for_framework_4_6() {
    let temp = TempDir::new().expect("temp dir");
    let project = create_project(&temp, "run-legacy-46");

    set_framework_version(&project, "4.6");
    add_legacy_runner(&project);

    let output = run_uto(&[
        "run",
        "--project",
        project.to_str().expect("project path utf-8"),
        "--target",
        "web",
    ]);

    assert!(
        output.status.success(),
        "run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("legacy sunset window"), "stderr: {stderr}");
}

#[test]
fn legacy_runner_rejected_for_framework_4_8() {
    let temp = TempDir::new().expect("temp dir");
    let project = create_project(&temp, "run-legacy-48");

    set_framework_version(&project, "4.8");
    add_legacy_runner(&project);

    let output = run_uto(&[
        "run",
        "--project",
        project.to_str().expect("project path utf-8"),
        "--target",
        "web",
    ]);

    assert!(!output.status.success(), "expected uto run to fail");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Legacy runner mode is disabled for framework_version 4.8"),
        "stderr: {stderr}"
    );
}

#[test]
fn report_rejects_invalid_schema() {
    let temp = TempDir::new().expect("temp dir");
    let project = create_project(&temp, "report-invalid-schema");

    let report_path = project.join(".uto/reports/last-run.json");
    fs::write(
        &report_path,
        r#"{
  "schema_version": "uto-report/v2",
  "status": "passed",
  "timeline": { "duration_ms": 1 },
  "events": []
}"#,
    )
    .expect("write report");

    let output = run_uto(&[
        "report",
        "--project",
        project.to_str().expect("project path utf-8"),
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Unsupported report schema"),
        "stderr: {stderr}"
    );
}

#[test]
fn report_summarizes_valid_report() {
    let temp = TempDir::new().expect("temp dir");
    let project = create_project(&temp, "report-valid");

    let report_path = project.join(".uto/reports/last-run.json");
    fs::write(
        &report_path,
        r#"{
  "schema_version": "uto-report/v1",
    "framework": "uto",
  "run_id": "run-123",
    "mode": "web",
  "status": "passed",
    "timeline": {
        "started_at_unix_ms": 1,
        "finished_at_unix_ms": 43,
        "duration_ms": 42
    },
  "events": []
}"#,
    )
    .expect("write report");

    let output = run_uto(&[
        "report",
        "--project",
        project.to_str().expect("project path utf-8"),
    ]);

    assert!(
        output.status.success(),
        "report failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("UTO Report Summary"), "stdout: {stdout}");
    assert!(stdout.contains("Schema: uto-report/v1"), "stdout: {stdout}");
}

#[test]
fn report_generates_html_when_flag_set() {
    let temp = TempDir::new().expect("temp dir");
    let project = create_project(&temp, "report-html");

    let report_path = project.join(".uto/reports/last-run.json");
    fs::write(
        &report_path,
        r#"{
  "schema_version": "uto-report/v1",
  "framework": "uto",
  "run_id": "run-456",
  "mode": "web",
  "status": "passed",
  "timeline": {
    "started_at_unix_ms": 100,
    "finished_at_unix_ms": 250,
    "duration_ms": 150
  },
  "events": [
    {
      "stage": "session.goto",
      "status": "ok",
      "detail": {"url": "https://example.com"}
    }
  ]
}"#,
    )
    .expect("write report");

    let html_path = project.join(".uto/reports/last-run.html");
    let output = run_uto(&[
        "report",
        "--project",
        project.to_str().expect("project path utf-8"),
        "--html",
    ]);

    assert!(
        output.status.success(),
        "report failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("HTML:"), "stdout: {stdout}");

    assert!(
        html_path.exists(),
        "HTML file not created at {}",
        html_path.display()
    );
    let html_content = fs::read_to_string(&html_path).expect("read HTML");
    assert!(html_content.contains("Execution Report"));
    assert!(html_content.contains("run-456"));
    assert!(html_content.contains("session.goto"));
    assert!(html_content.contains("uto-report/v1"));
}
