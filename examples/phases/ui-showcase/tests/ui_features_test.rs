//! Schema validation and integration test for the UI showcase project.
//!
//! Validates that:
//! - The project's uto.json is properly formed
//! - Report schemas are compatible with the UI server
//! - Tests compile and run without panics

use uto_reporter::schema::{UtoSuiteReportV1, UTO_SUITE_SCHEMA_V1};
use std::path::PathBuf;

/// Verify that uto.json is valid and matches expected schema.
#[test]
fn uto_json_schema_validation() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let config_path = PathBuf::from(manifest_dir).join("uto.json");

    let content = std::fs::read_to_string(&config_path)
        .expect("uto.json should exist");

    let value: serde_json::Value = serde_json::from_str(&content)
        .expect("uto.json must be valid JSON");

    // Validate schema_version
    assert_eq!(
        value["schema_version"].as_str().unwrap(),
        "1",
        "schema_version must be '1'"
    );

    // Validate project_name
    assert_eq!(
        value["project_name"].as_str().unwrap(),
        "ui-showcase",
        "project_name must be 'ui-showcase'"
    );

    // Validate target
    assert_eq!(
        value["default_target"].as_str().unwrap(),
        "web",
        "default_target must be 'web' (mobile requires Appium)"
    );

    // Validate report schema
    assert_eq!(
        value["report_schema"].as_str().unwrap(),
        UTO_SUITE_SCHEMA_V1,
        "report_schema must match uto-suite/v1"
    );
}

/// Verify that the uto-reporter schema used by this project
/// is compatible with the UI server expectations.
#[test]
fn suite_report_schema_compatible_with_ui_server() {
    // Create a synthetic report as this project would emit
    let report = UtoSuiteReportV1::new(
        "ui-showcase-synthetic".to_string(),
        "web".to_string(),
        1_000_000,
    );

    // Serialize and deserialize to verify schema roundtrip
    let json_str = serde_json::to_string(&report)
        .expect("Report should serialize to JSON");

    let deserialized: serde_json::Value = serde_json::from_str(&json_str)
        .expect("JSON should deserialize");

    // Verify top-level fields expected by UI server
    assert_eq!(
        deserialized["schema_version"].as_str().unwrap(),
        UTO_SUITE_SCHEMA_V1,
        "UI server expects schema_version to match uto-suite/v1"
    );

    assert_eq!(
        deserialized["mode"].as_str().unwrap(),
        "web",
        "UI server expects 'mode' field with web/mobile value"
    );

    assert!(
        deserialized["status"].as_str().is_some(),
        "UI server expects 'status' field (running/finished/passed/failed)"
    );

    // Verify that the report can be serialized multiple times
    // (important for replay scenarios)
    let json2 = serde_json::to_string(&report).expect("Re-serialize");
    let value2: serde_json::Value = serde_json::from_str(&json2).expect("Re-deserialize");

    assert_eq!(
        deserialized["schema_version"],
        value2["schema_version"],
        "Schema version should be consistent across serialization"
    );
}

/// Verify Cargo.toml has correct dependencies for UI mode integration.
#[test]
fn cargo_toml_has_ui_compatible_dependencies() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let cargo_path = PathBuf::from(manifest_dir).join("Cargo.toml");

    let content = std::fs::read_to_string(&cargo_path)
        .expect("Cargo.toml should exist");

    // Verify key dependencies are present
    assert!(
        content.contains("uto-core"),
        "Cargo.toml must depend on uto-core"
    );

    assert!(
        content.contains("uto-test"),
        "Cargo.toml must depend on uto-test for test authoring"
    );

    assert!(
        !content.contains("uto-runner"),
        "Cargo.toml should not depend on uto-runner in runnerless mode"
    );

    assert!(
        content.contains("uto-reporter"),
        "Cargo.toml must depend on uto-reporter for schema compatibility"
    );
}

/// Verify test directory structure.
#[test]
fn test_directory_structure_is_valid() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let tests_dir = PathBuf::from(manifest_dir).join("tests");

    assert!(
        tests_dir.exists(),
        "tests/ directory must exist"
    );

    let test_files = std::fs::read_dir(&tests_dir)
        .expect("tests/ must be readable")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path()
                .extension()
                .map(|ext| ext == "rs")
                .unwrap_or(false)
        })
        .count();

    assert!(
        test_files > 0,
        "tests/ directory must contain at least one .rs test file"
    );
}
