use std::sync::OnceLock;

use uto_reporter::ReportEvent;
use uto_runner::{append_live_event, LiveEventConfig, LiveEventEnvelope};

static LIVE_EVENT_CONFIG: OnceLock<Option<LiveEventConfig>> = OnceLock::new();

fn live_event_config() -> Option<&'static LiveEventConfig> {
    LIVE_EVENT_CONFIG
        .get_or_init(LiveEventConfig::from_env)
        .as_ref()
}

pub(crate) fn emit_report_event(stage: &str, status: &str, detail: serde_json::Value) {
    let Some(config) = live_event_config() else {
        return;
    };

    let event = LiveEventEnvelope::report_event(
        config,
        ReportEvent {
            stage: stage.to_string(),
            status: status.to_string(),
            detail,
        },
    );

    if let Err(err) = append_live_event(&config.file_path, &event) {
        log::warn!("uto-test: failed to append live event stream: {err}");
    }
}
