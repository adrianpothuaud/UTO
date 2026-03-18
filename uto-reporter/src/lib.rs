//! Structured reporting surfaces for UTO.
//!
//! This crate owns the machine-readable `uto-report/v1` and `uto-suite/v1` schemas
//! plus deterministic JSON/HTML emission utilities used by the CLI, POCs, and examples.

pub mod html;
pub mod report;
pub mod schema;
pub mod suite_report;

pub use html::{render_report_html, render_suite_html, write_report_html, write_suite_html};
pub use report::Report;
pub use schema::{
    ReportEvent, ReportTimeline, SuiteSummary, TestCaseResult, UtoReportV1, UtoSuiteReportV1,
    UTO_REPORT_SCHEMA_V1, UTO_SUITE_SCHEMA_V1,
};
pub use suite_report::{SuiteReport, TestRunHandle};
