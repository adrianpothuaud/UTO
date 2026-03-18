use std::sync::Arc;
use std::time::Duration;

use serde::Deserialize;
use serde_json::json;
use thirtyfour::{session::handle::SessionHandle, Capabilities, SessionId, WebDriver};

use crate::error::{UtoError, UtoResult};

/// Returns an alternate Appium base URL by toggling `/wd/hub` suffix.
pub(crate) fn appium_alternate_base_url(url: &str) -> Option<String> {
    let trimmed = url.trim_end_matches('/');

    if let Some(base) = trimmed.strip_suffix("/wd/hub") {
        Some(base.to_string())
    } else if !trimmed.is_empty() {
        Some(format!("{trimmed}/wd/hub"))
    } else {
        None
    }
}

/// Detects Appium errors that usually indicate an incorrect base path.
pub(crate) fn is_appium_base_path_mismatch_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("unknown command") && lower.contains("404")
}

#[derive(Debug, Deserialize)]
struct AppiumSessionResponse {
    #[serde(default, rename = "sessionId")]
    session_id: String,
    value: AppiumSessionValue,
}

#[derive(Debug, Deserialize)]
struct AppiumSessionValue {
    #[serde(default, rename = "sessionId")]
    session_id: String,
}

/// Creates a WebDriver handle by calling Appium's W3C new-session endpoint.
pub(crate) async fn connect_appium_driver(
    appium_url: &str,
    caps: &Capabilities,
) -> UtoResult<WebDriver> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| {
            UtoError::SessionCreationFailed(format!("Failed to build HTTP client: {e}"))
        })?;

    let session_url = format!("{}/session", appium_url.trim_end_matches('/'));
    let response = client
        .post(&session_url)
        .json(&json!({
            "capabilities": {
                "alwaysMatch": caps,
                "firstMatch": [ {} ]
            }
        }))
        .send()
        .await
        .map_err(|e| {
            UtoError::SessionCreationFailed(format!(
                "Failed to create Appium session at {session_url}: {e}"
            ))
        })?;

    let status = response.status();
    let body = response.text().await.map_err(|e| {
        UtoError::SessionCreationFailed(format!(
            "Failed to read Appium session response from {session_url}: {e}"
        ))
    })?;

    if !status.is_success() {
        return Err(UtoError::SessionCreationFailed(format!(
            "Appium at {appium_url}: {body}"
        )));
    }

    let session_response: AppiumSessionResponse = serde_json::from_str(&body).map_err(|e| {
        UtoError::SessionCreationFailed(format!(
            "Failed to parse Appium new-session response from {session_url}: {e}; body: {body}"
        ))
    })?;

    let session_id = if session_response.session_id.is_empty() {
        session_response.value.session_id
    } else {
        session_response.session_id
    };

    if session_id.is_empty() {
        return Err(UtoError::SessionCreationFailed(format!(
            "Appium at {appium_url} returned a successful new-session response without a session id"
        )));
    }

    let http_client: Arc<dyn thirtyfour::session::http::HttpClient> = Arc::new(client);
    let handle =
        SessionHandle::new(http_client, appium_url, SessionId::from(session_id)).map_err(|e| {
            UtoError::SessionCreationFailed(format!(
                "Failed to attach Appium session to thirtyfour handle: {e}"
            ))
        })?;

    Ok(WebDriver {
        handle: Arc::new(handle),
    })
}

#[cfg(test)]
mod tests {
    use super::{appium_alternate_base_url, is_appium_base_path_mismatch_error};

    #[test]
    fn alternate_base_url_removes_wd_hub_suffix() {
        let alt = appium_alternate_base_url("http://localhost:4723/wd/hub");
        assert_eq!(alt.as_deref(), Some("http://localhost:4723"));
    }

    #[test]
    fn alternate_base_url_adds_wd_hub_suffix() {
        let alt = appium_alternate_base_url("http://localhost:4723");
        assert_eq!(alt.as_deref(), Some("http://localhost:4723/wd/hub"));
    }

    #[test]
    fn base_path_mismatch_detector_matches_unknown_command_404() {
        let msg = "Unknown command:\nStatus: 404";
        assert!(is_appium_base_path_mismatch_error(msg));
    }

    #[test]
    fn base_path_mismatch_detector_ignores_non_404_failures() {
        let msg = "Could not find a connected Android device";
        assert!(!is_appium_base_path_mismatch_error(msg));
    }
}
