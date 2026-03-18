//! Typed uto-report/v1 schema surfaces.

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const UTO_REPORT_SCHEMA_V1: &str = "uto-report/v1";

/// Top-level structured report artifact for a single run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtoReportV1 {
    pub schema_version: String,
    pub framework: String,
    pub run_id: String,
    pub mode: String,
    pub status: String,
    pub timeline: ReportTimeline,
    pub events: Vec<ReportEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl UtoReportV1 {
    /// Creates a new running report payload.
    pub fn new(run_id: String, mode: String, start_ms: u64) -> Self {
        Self {
            schema_version: UTO_REPORT_SCHEMA_V1.to_string(),
            framework: "uto".to_string(),
            run_id,
            mode,
            status: "running".to_string(),
            timeline: ReportTimeline {
                started_at_unix_ms: start_ms,
                finished_at_unix_ms: None,
                duration_ms: None,
            },
            events: Vec::new(),
            error: None,
        }
    }
}

/// Timing metadata for a run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportTimeline {
    pub started_at_unix_ms: u64,
    pub finished_at_unix_ms: Option<u64>,
    pub duration_ms: Option<u64>,
}

/// A report event emitted during execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportEvent {
    pub stage: String,
    pub status: String,
    pub detail: Value,
}

// ---------------------------------------------------------------------------
// Suite schema (uto-suite/v1)
// ---------------------------------------------------------------------------

/// Schema version constant for a multi-test suite run.
pub const UTO_SUITE_SCHEMA_V1: &str = "uto-suite/v1";

/// Pass/fail/skip summary counts for a suite run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuiteSummary {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
}

/// Result of a single named test case within a suite run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub name: String,
    /// `passed` | `failed` | `skipped`
    pub status: String,
    pub timeline: ReportTimeline,
    pub events: Vec<ReportEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Top-level structured report artifact for a multi-test suite run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtoSuiteReportV1 {
    pub schema_version: String,
    pub framework: String,
    pub suite_id: String,
    pub mode: String,
    /// `passed` | `partial` | `failed`
    pub status: String,
    pub timeline: ReportTimeline,
    pub summary: SuiteSummary,
    pub tests: Vec<TestCaseResult>,
}

impl UtoSuiteReportV1 {
    /// Creates a new in-progress suite report payload.
    pub fn new(suite_id: String, mode: String, start_ms: u64) -> Self {
        Self {
            schema_version: UTO_SUITE_SCHEMA_V1.to_string(),
            framework: "uto".to_string(),
            suite_id,
            mode,
            status: "running".to_string(),
            timeline: ReportTimeline {
                started_at_unix_ms: start_ms,
                finished_at_unix_ms: None,
                duration_ms: None,
            },
            summary: SuiteSummary {
                total: 0,
                passed: 0,
                failed: 0,
                skipped: 0,
            },
            tests: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_v1_round_trip_serialization() {
        let mut report = UtoReportV1::new("run-1".to_string(), "web".to_string(), 1000);
        report.events.push(ReportEvent {
            stage: "session.goto".to_string(),
            status: "ok".to_string(),
            detail: serde_json::json!({ "target": "https://example.com" }),
        });
        report.status = "passed".to_string();
        report.timeline.finished_at_unix_ms = Some(1100);
        report.timeline.duration_ms = Some(100);

        let json = serde_json::to_string(&report).expect("serialize");
        let parsed: UtoReportV1 = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.schema_version, UTO_REPORT_SCHEMA_V1);
        assert_eq!(parsed.mode, "web");
        assert_eq!(parsed.status, "passed");
        assert_eq!(parsed.events.len(), 1);
    }
}
