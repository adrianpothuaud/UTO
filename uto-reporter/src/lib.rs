//! Structured reporting surfaces for UTO.
//!
//! This crate owns the machine-readable `uto-report/v1` schema and
//! deterministic JSON/HTML emission utilities used by CLI, POCs, and examples.

pub mod html;
pub mod report;
pub mod schema;

pub use html::{render_report_html, write_report_html};
pub use report::Report;
pub use schema::{ReportEvent, ReportTimeline, UtoReportV1, UTO_REPORT_SCHEMA_V1};
