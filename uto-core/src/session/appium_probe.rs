use std::time::Duration;

use crate::error::{UtoError, UtoResult};

/// Diagnostics from an Appium preflight probe.
#[derive(Debug, Clone)]
pub struct AppiumProbe {
    /// Whether the `/status` endpoint is reachable.
    pub status_endpoint_ok: bool,
    /// Whether the `/session` endpoint appears to accept POST requests.
    pub session_endpoint_ok: bool,
    /// Detected Appium version (if available).
    pub appium_version: Option<String>,
    /// List of available Appium drivers detected.
    pub available_drivers: Vec<String>,
}

/// Probes an Appium server to detect route compatibility and configuration.
///
/// Returns diagnostic information that can help troubleshoot session creation failures.
pub async fn probe_appium(appium_url: &str) -> UtoResult<AppiumProbe> {
    let client = reqwest::Client::new();
    let status_url = format!("{}/status", appium_url.trim_end_matches('/'));

    // Probe 1: /status endpoint
    let mut appium_version = None;
    let mut available_drivers = vec![];

    let status_result =
        tokio::time::timeout(Duration::from_secs(5), client.get(&status_url).send()).await;

    let status_endpoint_ok = match status_result {
        Ok(Ok(resp)) if resp.status().is_success() => {
            if let Ok(body) = resp.json::<serde_json::Value>().await {
                appium_version = body
                    .get("value")
                    .and_then(|v| v.get("build"))
                    .and_then(|v| v.get("version"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                // Extract available drivers from the response.
                if let Some(value) = body.get("value") {
                    if let Some(drivers_obj) = value.get("drivers") {
                        if let Ok(drivers_map) = serde_json::from_value::<
                            std::collections::HashMap<String, serde_json::Value>,
                        >(drivers_obj.clone())
                        {
                            available_drivers = drivers_map.keys().cloned().collect();
                        }
                    }
                }

                log::debug!(
                    "Appium status: version={:?}, drivers={:?}",
                    appium_version,
                    available_drivers
                );
            }
            true
        }
        _ => {
            return Err(UtoError::EnvironmentSetupFailed(format!(
                "Appium status endpoint not reachable at {status_url}. \
                 Verify Appium is running and the URL is correct."
            )));
        }
    };

    // Probe 2: /session endpoint (attempt GET to check if route exists)
    let session_url = format!("{}/session", appium_url.trim_end_matches('/'));
    let session_endpoint_ok = client
        .get(&session_url)
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map(|r| !r.status().is_client_error())
        .unwrap_or(false);

    Ok(AppiumProbe {
        status_endpoint_ok,
        session_endpoint_ok,
        appium_version,
        available_drivers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_probe_structure() {
        let probe = AppiumProbe {
            status_endpoint_ok: true,
            session_endpoint_ok: true,
            appium_version: Some("2.0.0".to_string()),
            available_drivers: vec!["uiautomator2".to_string()],
        };

        assert!(probe.status_endpoint_ok);
        assert_eq!(probe.appium_version.as_deref(), Some("2.0.0"));
    }
}
