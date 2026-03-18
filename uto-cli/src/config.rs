//! Project configuration types and validation.

use serde::{Deserialize, Serialize};
use std::path::Path;

pub const PROJECT_SCHEMA_VERSION: &str = "1";
pub const DEFAULT_REPORT_SCHEMA_VERSION: &str = uto_reporter::UTO_SUITE_SCHEMA_V1;

fn is_supported_report_schema(schema: &str) -> bool {
    matches!(
        schema,
        uto_reporter::UTO_REPORT_SCHEMA_V1 | uto_reporter::UTO_SUITE_SCHEMA_V1
    )
}

/// UTO project configuration from uto.json.
#[derive(Debug, Serialize, Deserialize)]
pub struct UtoProjectConfig {
    pub schema_version: String,
    pub project_name: String,
    pub tests_dir: String,
    pub default_target: String,
    pub report_dir: String,
    pub uto_root: String,
    pub report_schema: String,
}

impl UtoProjectConfig {
    /// Validates that the config has a recognized schema and consistent fields.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != PROJECT_SCHEMA_VERSION {
            return Err(format!(
                "Unsupported uto.json schema_version '{}'. Expected {}",
                self.schema_version, PROJECT_SCHEMA_VERSION
            ));
        }

        if !is_supported_report_schema(&self.report_schema) {
            return Err(format!(
                "Unsupported report_schema '{}'. Expected {} or {}",
                self.report_schema,
                uto_reporter::UTO_REPORT_SCHEMA_V1,
                uto_reporter::UTO_SUITE_SCHEMA_V1,
            ));
        }

        if self.project_name.trim().is_empty() {
            return Err("Invalid uto.json: project_name must not be empty".to_string());
        }
        if self.tests_dir.trim().is_empty() {
            return Err("Invalid uto.json: tests_dir must not be empty".to_string());
        }
        if self.report_dir.trim().is_empty() {
            return Err("Invalid uto.json: report_dir must not be empty".to_string());
        }

        let _ = super::parsing::normalize_target(&self.default_target)?;
        Ok(())
    }
}

/// Loads and validates project config from uto.json.
pub fn load_project_config(project: &Path) -> Result<UtoProjectConfig, String> {
    if !project.exists() {
        return Err(format!(
            "Project directory does not exist: {}",
            project.display()
        ));
    }

    let config_path = project.join("uto.json");
    if !config_path.exists() {
        return Err(format!(
            "Missing project config: {}. Run `uto init <project-dir>` first.",
            config_path.display()
        ));
    }

    let config: UtoProjectConfig = super::io::read_json(config_path)?;
    config.validate()?;
    Ok(config)
}

/// Validates that a project runner exists.
pub fn validate_project_runner(project: &Path) -> Result<(), String> {
    let runner = project.join("src/bin/uto_project_runner.rs");
    if !runner.exists() {
        return Err(format!(
            "Missing project runner: {}. Re-run `uto init` or restore the generated runner.",
            runner.display()
        ));
    }
    Ok(())
}

/// Parsed report — either a single-run or a multi-test suite.
pub enum ParsedReport {
    Single(uto_reporter::UtoReportV1),
    Suite(uto_reporter::UtoSuiteReportV1),
}

/// Validates report JSON and returns the appropriate typed report.
///
/// Supports both `uto-report/v1` (single run) and `uto-suite/v1` (suite run).
pub fn parse_report_json(report_value: &serde_json::Value) -> Result<ParsedReport, String> {
    let schema_version = report_value
        .get("schema_version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Invalid report: missing schema_version".to_string())?;

    match schema_version {
        uto_reporter::UTO_REPORT_SCHEMA_V1 => {
            let parsed: uto_reporter::UtoReportV1 = serde_json::from_value(report_value.clone())
                .map_err(|e| format!("Invalid uto-report/v1 shape: {e}"))?;
            if parsed.status.trim().is_empty() {
                return Err("Invalid report: missing status".to_string());
            }
            Ok(ParsedReport::Single(parsed))
        }
        uto_reporter::UTO_SUITE_SCHEMA_V1 => {
            let parsed: uto_reporter::UtoSuiteReportV1 =
                serde_json::from_value(report_value.clone())
                    .map_err(|e| format!("Invalid uto-suite/v1 shape: {e}"))?;
            if parsed.status.trim().is_empty() {
                return Err("Invalid suite report: missing status".to_string());
            }
            Ok(ParsedReport::Suite(parsed))
        }
        other => Err(format!(
            "Unsupported report schema '{other}'. Expected {} or {}",
            uto_reporter::UTO_REPORT_SCHEMA_V1,
            uto_reporter::UTO_SUITE_SCHEMA_V1,
        )),
    }
}
