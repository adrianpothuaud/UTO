//! Phase 6 Studio — schema compatibility and project config tests.

use uto_reporter::schema::{UtoSuiteReportV1, UTO_SUITE_SCHEMA_V1};

/// Verify that the `uto-suite/v1` report schema emitted by this project
/// is compatible with the UTO UI server (Phase 5+).
#[test]
fn suite_report_schema_compatible_with_ui_server() {
    let report = UtoSuiteReportV1::new(
        "phase6-studio-suite".to_string(),
        "web".to_string(),
        1_000_000,
    );

    let json = serde_json::to_string(&report).expect("serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("parse");

    assert_eq!(
        value["schema_version"].as_str().unwrap(),
        UTO_SUITE_SCHEMA_V1,
        "schema_version must match uto-suite/v1 for the UI server to render it"
    );
    assert_eq!(value["mode"].as_str().unwrap(), "web");
    assert_eq!(value["status"].as_str().unwrap(), "running");
}

/// Verify that `uto.json` in this project is valid.
#[test]
fn uto_json_is_valid() {
    let config_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("uto.json");
    let content = std::fs::read_to_string(&config_path)
        .unwrap_or_else(|_| panic!("uto.json not found at {}", config_path.display()));
    let value: serde_json::Value =
        serde_json::from_str(&content).expect("uto.json must be valid JSON");

    assert_eq!(value["schema_version"].as_str().unwrap(), "1");
    assert_eq!(value["project_name"].as_str().unwrap(), "phase6-studio");
    assert_eq!(value["default_target"].as_str().unwrap(), "web");
    assert_eq!(value["report_schema"].as_str().unwrap(), UTO_SUITE_SCHEMA_V1);
    assert_eq!(value["framework_version"].as_str().unwrap(), "6.0");
}
