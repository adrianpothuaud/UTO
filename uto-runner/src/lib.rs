//! Structured test project runner infrastructure for UTO.
//!
//! This crate provides reusable CLI option parsing for generated/reference
//! runner binaries, a shared live-event schema for streaming execution detail,
//! and re-exports reporting surfaces from `uto-reporter` for backward
//! compatibility.

pub mod live;
pub mod options;

pub use live::{
    append_live_event, LiveEventConfig, LiveEventEnvelope, LiveEventPayload,
    UTO_LIVE_EVENTS_FILE_ENV, UTO_LIVE_EVENTS_TARGET_ENV, UTO_LIVE_EVENTS_TEST_BIN_ENV,
    UTO_LIVE_EVENTS_TEST_NAME_ENV, UTO_LIVE_EVENT_SCHEMA_V1,
};
pub use options::{CliOptions, RunMode};
pub use uto_reporter::{
    render_report_html, write_report_html, Report, ReportEvent, ReportTimeline, UtoReportV1,
    UTO_REPORT_SCHEMA_V1,
};
