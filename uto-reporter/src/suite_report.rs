//! Suite-level report accumulator for multi-test runs.

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::schema::{ReportEvent, ReportTimeline, TestCaseResult, UtoSuiteReportV1};

// ---------------------------------------------------------------------------
// TestRunHandle
// ---------------------------------------------------------------------------

/// Accumulates events for a single named test case while it is executing.
///
/// Obtain one via [`SuiteReport::begin_test`] and return it to the suite via
/// [`SuiteReport::record_test`] once the test case completes.
pub struct TestRunHandle {
    pub(crate) name: String,
    pub(crate) events: Vec<ReportEvent>,
    pub(crate) start_ms: u64,
}

impl TestRunHandle {
    /// Records one execution event for this test case.
    pub fn event(&mut self, stage: &str, status: &str, detail: Value) {
        self.events.push(ReportEvent {
            stage: stage.to_string(),
            status: status.to_string(),
            detail,
        });
    }
}

// ---------------------------------------------------------------------------
// SuiteReport
// ---------------------------------------------------------------------------

/// Accumulates test-case outcomes into a suite-level [`UtoSuiteReportV1`] payload.
#[derive(Debug)]
pub struct SuiteReport {
    enabled: bool,
    report_file: Option<String>,
    payload: UtoSuiteReportV1,
    start_ms: u64,
}

impl SuiteReport {
    /// Creates a new suite report. Pass `enabled = false` to suppress all output.
    pub fn new(enabled: bool, report_file: Option<String>, mode: &str) -> Self {
        let start_ms = now_unix_ms();
        let suite_id = format!("suite-{}-{}", start_ms, std::process::id());
        Self {
            enabled,
            report_file,
            payload: UtoSuiteReportV1::new(suite_id, mode.to_string(), start_ms),
            start_ms,
        }
    }

    /// Opens a new [`TestRunHandle`] for a named test case.
    pub fn begin_test(&self, name: &str) -> TestRunHandle {
        TestRunHandle {
            name: name.to_string(),
            events: Vec::new(),
            start_ms: now_unix_ms(),
        }
    }

    /// Records a completed test case into the suite.
    ///
    /// `status` should be one of `"passed"`, `"failed"`, or `"skipped"`.
    pub fn record_test(&mut self, handle: TestRunHandle, status: &str, error: Option<String>) {
        if !self.enabled {
            return;
        }
        let end_ms = now_unix_ms();
        match status {
            "passed" => self.payload.summary.passed += 1,
            "failed" => self.payload.summary.failed += 1,
            "skipped" => self.payload.summary.skipped += 1,
            _ => {}
        }
        self.payload.summary.total += 1;
        self.payload.tests.push(TestCaseResult {
            name: handle.name,
            status: status.to_string(),
            timeline: ReportTimeline {
                started_at_unix_ms: handle.start_ms,
                finished_at_unix_ms: Some(end_ms),
                duration_ms: Some(end_ms.saturating_sub(handle.start_ms)),
            },
            events: handle.events,
            error,
        });
    }

    /// Finalises the suite timeline and derives the overall pass/fail status.
    pub fn finish(&mut self) {
        if !self.enabled {
            return;
        }
        let end_ms = now_unix_ms();
        self.payload.timeline.finished_at_unix_ms = Some(end_ms);
        self.payload.timeline.duration_ms = Some(end_ms.saturating_sub(self.start_ms));

        let s = &self.payload.summary;
        self.payload.status = if s.total == 0 || s.failed == 0 {
            "passed".to_string()
        } else if s.passed > 0 {
            "partial".to_string()
        } else {
            "failed".to_string()
        };
    }

    /// Returns a reference to the suite report payload.
    pub fn payload(&self) -> &UtoSuiteReportV1 {
        &self.payload
    }

    /// Serialises the suite report to stdout and writes JSON to the configured file.
    pub fn emit(&self) {
        if !self.enabled {
            return;
        }
        let serialized = match serde_json::to_string_pretty(&self.payload) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to serialize suite report: {e}");
                return;
            }
        };
        println!("{serialized}");
        if let Some(path) = &self.report_file {
            if let Err(e) = std::fs::write(path, &serialized) {
                eprintln!("Failed to write suite report to {path}: {e}");
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suite_single_passing_test_has_passed_status() {
        let mut suite = SuiteReport::new(true, None, "web");
        let h = suite.begin_test("example");
        suite.record_test(h, "passed", None);
        suite.finish();
        let p = suite.payload();
        assert_eq!(p.summary.total, 1);
        assert_eq!(p.summary.passed, 1);
        assert_eq!(p.status, "passed");
    }

    #[test]
    fn suite_all_fail_has_failed_status() {
        let mut suite = SuiteReport::new(true, None, "web");
        let h = suite.begin_test("bad");
        suite.record_test(h, "failed", Some("boom".to_string()));
        suite.finish();
        assert_eq!(suite.payload().status, "failed");
    }

    #[test]
    fn suite_mixed_results_has_partial_status() {
        let mut suite = SuiteReport::new(true, None, "web");
        let h1 = suite.begin_test("ok");
        suite.record_test(h1, "passed", None);
        let h2 = suite.begin_test("bad");
        suite.record_test(h2, "failed", Some("err".to_string()));
        suite.finish();
        assert_eq!(suite.payload().status, "partial");
        assert_eq!(suite.payload().summary.total, 2);
    }

    #[test]
    fn suite_empty_run_is_passed() {
        let mut suite = SuiteReport::new(true, None, "web");
        suite.finish();
        assert_eq!(suite.payload().status, "passed");
    }

    #[test]
    fn test_run_handle_records_events() {
        let suite = SuiteReport::new(true, None, "web");
        let mut h = suite.begin_test("t1");
        h.event("session.start", "ok", serde_json::json!({}));
        h.event("assert.title", "ok", serde_json::json!({"title": "Example"}));
        assert_eq!(h.events.len(), 2);
    }
}
