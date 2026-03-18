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

fn init_project(temp: &TempDir, name: &str, template: &str) -> PathBuf {
    let project = temp.path().join(name);
    let root = workspace_root();
    let output = run_uto(&[
        "init",
        project.to_str().expect("project path utf-8"),
        "--template",
        template,
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

fn cargo_check_tests(project: &Path) -> std::process::Output {
    Command::new("cargo")
        .arg("check")
        .arg("--tests")
        .current_dir(project)
        .output()
        .expect("failed to run cargo check --tests")
}

#[test]
fn generated_web_project_compiles_with_uto_test_and_uto_runner() {
    let temp = TempDir::new().expect("temp dir");
    let project = init_project(&temp, "compat-web", "web");

    let cargo_toml =
        fs::read_to_string(project.join("Cargo.toml")).expect("read generated Cargo.toml");
    assert!(cargo_toml.contains("uto-test"));
    assert!(cargo_toml.contains("uto-runner"));
    assert!(cargo_toml.contains("uto-reporter"));
    assert!(cargo_toml.contains("uto-logger"));

    let check = cargo_check_tests(&project);
    assert!(
        check.status.success(),
        "cargo check --tests failed: {}",
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn generated_mobile_project_compiles_with_uto_test_and_uto_runner() {
    let temp = TempDir::new().expect("temp dir");
    let project = init_project(&temp, "compat-mobile", "mobile");

    let runner =
        fs::read_to_string(project.join("src/bin/uto_project_runner.rs")).expect("read runner");
    assert!(runner.contains("use uto_test::{ManagedSession, Suite};"));
    assert!(runner.contains("use uto_runner::{CliOptions, RunMode};"));

    let check = cargo_check_tests(&project);
    assert!(
        check.status.success(),
        "cargo check --tests failed: {}",
        String::from_utf8_lossy(&check.stderr)
    );
}
