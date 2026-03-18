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

#[test]
fn init_scaffolds_expected_project_files() {
    let temp = TempDir::new().expect("temp dir");
    let project = create_project(&temp, "init-smoke");

    assert!(project.join("uto.json").exists());
    assert!(project.join("Cargo.toml").exists());
    assert!(project.join("README.md").exists());
    assert!(project.join("src/bin/uto_project_runner.rs").exists());
    assert!(project.join("tests/web_example.rs").exists());
    assert!(project.join("tests/mobile_example.rs").exists());
}

#[test]
fn run_fails_fast_when_runner_is_missing() {
    let temp = TempDir::new().expect("temp dir");
    let project = create_project(&temp, "run-missing-runner");

    fs::remove_file(project.join("src/bin/uto_project_runner.rs")).expect("remove runner");

    let output = run_uto(&[
        "run",
        "--project",
        project.to_str().expect("project path utf-8"),
        "--target",
        "web",
    ]);

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Missing project runner"),
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
    assert!(html_content.contains("UTO Execution Report"));
    assert!(html_content.contains("run-456"));
    assert!(html_content.contains("session.goto"));
    assert!(html_content.contains("uto-report/v1"));
}
