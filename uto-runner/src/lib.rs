//! Structured test project runner infrastructure for UTO.
//!
//! This crate provides reusable CLI option parsing for generated/reference
//! runner binaries and re-exports reporting surfaces from `uto-reporter`
//! for backward compatibility.

pub mod options;

pub use options::{CliOptions, RunMode};
pub use uto_reporter::{
    render_report_html, write_report_html, Report, ReportEvent, ReportTimeline, UtoReportV1,
    UTO_REPORT_SCHEMA_V1,
};
