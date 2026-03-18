//! Structured test project runner infrastructure for UTO.
//!
//! This crate provides reusable components for running UTO tests with
//! structured JSON reporting. Generated and reference test projects
//! can use these components to keep their runner code focused on
//! test scenarios rather than boilerplate.

pub mod html;
pub mod options;
pub mod report;
pub mod schema;

pub use html::{render_report_html, write_report_html};
pub use options::{CliOptions, RunMode};
pub use report::Report;
pub use schema::{ReportEvent, ReportTimeline, UtoReportV1, UTO_REPORT_SCHEMA_V1};
