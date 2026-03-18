//! Structured JSON report generation for test runs.

use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::options::RunMode;
use crate::schema::{ReportEvent, UtoReportV1};

/// Accumulates test execution events and generates structured reports.
#[derive(Debug)]
pub struct Report {
    enabled: bool,
    report_file: Option<String>,
    payload: UtoReportV1,
    start_ms: u64,
}

pub type RunResult = Result<(), String>;

impl Report {
    /// Creates a new report with the given options.
    pub fn new(enabled: bool, report_file: Option<String>, mode: RunMode) -> Self {
        let start_ms = now_unix_ms();
        let run_id = format!("run-{}-{}", start_ms, std::process::id());
        let mode = match mode {
            RunMode::Web => "web".to_string(),
            RunMode::Mobile => "mobile".to_string(),
        };

        Self {
            enabled,
            report_file,
            payload: UtoReportV1::new(run_id, mode, start_ms),
            start_ms,
        }
    }

    /// Records an event in the report timeline.
    pub fn event(&mut self, stage: &str, status: &str, detail: Value) {
        if !self.enabled {
            return;
        }

        self.payload.events.push(ReportEvent {
            stage: stage.to_string(),
            status: status.to_string(),
            detail,
        });
    }

    /// Marks the report as finished with overall status and optional error.
    pub fn finish(&mut self, status: &str, error: Option<String>) {
        if !self.enabled {
            return;
        }

        let end_ms = now_unix_ms();
        self.payload.status = status.to_string();
        self.payload.timeline.finished_at_unix_ms = Some(end_ms);
        self.payload.timeline.duration_ms = Some(end_ms - self.start_ms);

        if let Some(err) = error {
            self.payload.error = Some(err);
        }
    }

    /// Prints and writes the report to stdout and optional file.
    pub fn emit(&self) {
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

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_new_initializes_structure() {
        let report = Report::new(true, None, RunMode::Web);
        assert_eq!(report.payload.schema_version, "uto-report/v1");
        assert_eq!(report.payload.mode, "web");
        assert_eq!(report.payload.status, "running");
    }

    #[test]
    fn report_event_records_when_enabled() {
        let mut report = Report::new(true, None, RunMode::Web);
        report.event("test.stage", "ok", serde_json::json!({"key": "value"}));
        assert_eq!(report.payload.events.len(), 1);
        assert_eq!(report.payload.events[0].stage, "test.stage");
    }

    #[test]
    fn report_finish_updates_status_and_timeline() {
        let mut report = Report::new(true, None, RunMode::Mobile);
        report.finish("passed", None);

        assert_eq!(report.payload.status, "passed");
        assert!(report.payload.timeline.finished_at_unix_ms.is_some());
        assert!(report.payload.timeline.duration_ms.is_some());
    }

    #[test]
    fn report_finish_records_error() {
        let mut report = Report::new(true, None, RunMode::Web);
        report.finish("failed", Some("Test failed".to_string()));

        assert_eq!(report.payload.status, "failed");
        assert_eq!(report.payload.error.as_deref(), Some("Test failed"));
    }
}
