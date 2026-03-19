use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use uto_reporter::ReportEvent;

pub const UTO_LIVE_EVENT_SCHEMA_V1: &str = "uto-live/v1";

pub const UTO_LIVE_EVENTS_FILE_ENV: &str = "UTO_LIVE_EVENTS_FILE";
pub const UTO_LIVE_EVENTS_TEST_BIN_ENV: &str = "UTO_LIVE_EVENTS_TEST_BIN";
pub const UTO_LIVE_EVENTS_TEST_NAME_ENV: &str = "UTO_LIVE_EVENTS_TEST_NAME";
pub const UTO_LIVE_EVENTS_TARGET_ENV: &str = "UTO_LIVE_EVENTS_TARGET";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveEventConfig {
    pub file_path: PathBuf,
    pub test_bin: String,
    pub test_name: String,
    pub target: Option<String>,
}

impl LiveEventConfig {
    pub fn from_env() -> Option<Self> {
        let file_path = std::env::var(UTO_LIVE_EVENTS_FILE_ENV).ok()?;
        let test_bin = std::env::var(UTO_LIVE_EVENTS_TEST_BIN_ENV).ok()?;
        let test_name = std::env::var(UTO_LIVE_EVENTS_TEST_NAME_ENV).ok()?;
        let target = std::env::var(UTO_LIVE_EVENTS_TARGET_ENV)
            .ok()
            .filter(|value| !value.trim().is_empty());

        Some(Self {
            file_path: PathBuf::from(file_path),
            test_bin,
            test_name,
            target,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveEventEnvelope {
    pub schema_version: String,
    pub ts_unix_ms: u64,
    pub test_bin: String,
    pub test_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(flatten)]
    pub payload: LiveEventPayload,
}

impl LiveEventEnvelope {
    pub fn test_started(config: &LiveEventConfig) -> Self {
        Self {
            schema_version: UTO_LIVE_EVENT_SCHEMA_V1.to_string(),
            ts_unix_ms: now_unix_ms(),
            test_bin: config.test_bin.clone(),
            test_name: config.test_name.clone(),
            target: config.target.clone(),
            payload: LiveEventPayload::TestStarted,
        }
    }

    pub fn report_event(config: &LiveEventConfig, event: ReportEvent) -> Self {
        Self {
            schema_version: UTO_LIVE_EVENT_SCHEMA_V1.to_string(),
            ts_unix_ms: now_unix_ms(),
            test_bin: config.test_bin.clone(),
            test_name: config.test_name.clone(),
            target: config.target.clone(),
            payload: LiveEventPayload::ReportEvent { event },
        }
    }

    pub fn test_finished(
        config: &LiveEventConfig,
        status: &str,
        error: Option<String>,
        duration_ms: u64,
    ) -> Self {
        Self {
            schema_version: UTO_LIVE_EVENT_SCHEMA_V1.to_string(),
            ts_unix_ms: now_unix_ms(),
            test_bin: config.test_bin.clone(),
            test_name: config.test_name.clone(),
            target: config.target.clone(),
            payload: LiveEventPayload::TestFinished {
                status: status.to_string(),
                error,
                duration_ms,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LiveEventPayload {
    TestStarted,
    ReportEvent {
        event: ReportEvent,
    },
    TestFinished {
        status: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
        duration_ms: u64,
    },
}

pub fn append_live_event(path: &Path, event: &LiveEventEnvelope) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|err| {
            format!(
                "failed to create live event directory '{}': {err}",
                parent.display()
            )
        })?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| {
            format!(
                "failed to open live event stream '{}': {err}",
                path.display()
            )
        })?;

    let line = serde_json::to_string(event)
        .map_err(|err| format!("failed to serialize live event: {err}"))?;

    file.write_all(line.as_bytes())
        .and_then(|_| file.write_all(b"\n"))
        .map_err(|err| {
            format!(
                "failed to write live event stream '{}': {err}",
                path.display()
            )
        })
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_event_config_reads_environment() {
        unsafe {
            std::env::set_var(UTO_LIVE_EVENTS_FILE_ENV, "/tmp/uto-live.jsonl");
            std::env::set_var(UTO_LIVE_EVENTS_TEST_BIN_ENV, "login_test");
            std::env::set_var(UTO_LIVE_EVENTS_TEST_NAME_ENV, "login_ok");
            std::env::set_var(UTO_LIVE_EVENTS_TARGET_ENV, "web");
        }

        let config = LiveEventConfig::from_env().expect("expected config");
        assert_eq!(config.file_path, PathBuf::from("/tmp/uto-live.jsonl"));
        assert_eq!(config.test_bin, "login_test");
        assert_eq!(config.test_name, "login_ok");
        assert_eq!(config.target.as_deref(), Some("web"));
    }

    #[test]
    fn append_live_event_writes_json_line() {
        let temp = tempfile::NamedTempFile::new().expect("temp file");
        let config = LiveEventConfig {
            file_path: temp.path().to_path_buf(),
            test_bin: "login_test".to_string(),
            test_name: "login_ok".to_string(),
            target: Some("web".to_string()),
        };
        let event = LiveEventEnvelope::report_event(
            &config,
            ReportEvent {
                stage: "session.goto".to_string(),
                status: "ok".to_string(),
                detail: serde_json::json!({ "url": "https://example.com" }),
            },
        );

        append_live_event(temp.path(), &event).expect("write live event");

        let content = std::fs::read_to_string(temp.path()).expect("read stream file");
        let parsed: LiveEventEnvelope =
            serde_json::from_str(content.trim()).expect("parse live event");
        assert_eq!(parsed.test_bin, "login_test");
        match parsed.payload {
            LiveEventPayload::ReportEvent { event } => {
                assert_eq!(event.stage, "session.goto");
            }
            other => panic!("unexpected payload: {other:?}"),
        }
    }
}
