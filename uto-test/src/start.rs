use uto_core::{
    driver,
    env::{
        mobile_setup::{prepare_mobile_environment, MobileSetupOptions},
        platform::find_chrome_version,
        provisioning::find_or_provision_chromedriver,
    },
    error::{UtoError, UtoResult},
    session::{mobile::MobileCapabilities, mobile::MobileSession, web::WebSession},
};
use uto_reporter::ReportEvent;

use crate::managed_session::ManagedSession;
use crate::managed_session::SharedEvents;

fn report_event(
    report_events: &Option<SharedEvents>,
    stage: &str,
    status: &str,
    detail: serde_json::Value,
) {
    if let Some(shared) = report_events {
        if let Ok(mut events) = shared.lock() {
            events.push(ReportEvent {
                stage: stage.to_string(),
                status: status.to_string(),
                detail: detail.clone(),
            });
        }
    }

    crate::live_stream::emit_report_event(stage, status, detail);
}

/// Starts a new managed session with a simple target identifier.
pub async fn start_new_session(target: &str) -> UtoResult<ManagedSession> {
    start_new_session_with_hint(target, 0).await
}

/// Starts a new managed session with an optional platform hint.
///
/// For Android, `android_api_level_hint` is currently informational and
/// logged, while environment readiness still follows discover-or-deploy.
pub async fn start_new_session_with_hint(
    target: &str,
    android_api_level_hint: u16,
) -> UtoResult<ManagedSession> {
    start_new_session_with_hint_and_events(target, android_api_level_hint, None).await
}

pub(crate) async fn start_new_session_with_hint_and_events(
    target: &str,
    android_api_level_hint: u16,
    report_events: Option<SharedEvents>,
) -> UtoResult<ManagedSession> {
    match normalize_target(target)? {
        NormalizedTarget::Chrome => start_web_session(report_events).await,
        NormalizedTarget::Android => {
            start_android_session(android_api_level_hint, report_events).await
        }
    }
}

#[allow(non_snake_case)]
/// JavaScript-style alias for end-user familiarity.
pub async fn startNewSession(target: &str) -> UtoResult<ManagedSession> {
    start_new_session(target).await
}

#[allow(non_snake_case)]
/// JavaScript-style alias with optional hint argument.
pub async fn startNewSessionWithArg(target: &str, arg: u16) -> UtoResult<ManagedSession> {
    start_new_session_with_hint(target, arg).await
}

#[derive(Debug)]
enum NormalizedTarget {
    Chrome,
    Android,
}

fn normalize_target(target: &str) -> UtoResult<NormalizedTarget> {
    match target.to_ascii_lowercase().as_str() {
        "chrome" | "web" => Ok(NormalizedTarget::Chrome),
        "android" | "mobile" => Ok(NormalizedTarget::Android),
        other => Err(UtoError::SessionCommandFailed(format!(
            "Unsupported session target '{other}'. Use chrome or android"
        ))),
    }
}

async fn start_web_session(report_events: Option<SharedEvents>) -> UtoResult<ManagedSession> {
    let chrome_version = match find_chrome_version() {
        Ok(version) => {
            report_event(
                &report_events,
                "env.chrome_discovery",
                "ok",
                serde_json::json!({ "chrome_version": version }),
            );
            version
        }
        Err(err) => {
            report_event(
                &report_events,
                "env.chrome_discovery",
                "failed",
                serde_json::json!({ "error": err.to_string() }),
            );
            return Err(err);
        }
    };
    log::info!("uto-test: discovered chrome version {}", chrome_version);

    let chromedriver = match find_or_provision_chromedriver(&chrome_version).await {
        Ok(path) => {
            report_event(
                &report_events,
                "env.chromedriver_provision",
                "ok",
                serde_json::json!({ "path": path.display().to_string() }),
            );
            path
        }
        Err(err) => {
            report_event(
                &report_events,
                "env.chromedriver_provision",
                "failed",
                serde_json::json!({ "error": err.to_string() }),
            );
            return Err(err);
        }
    };
    log::info!("uto-test: using chromedriver at {}", chromedriver.display());

    let driver = match driver::start_chromedriver(&chromedriver).await {
        Ok(driver) => {
            report_event(
                &report_events,
                "driver.chromedriver_start",
                "ok",
                serde_json::json!({ "url": driver.url, "port": driver.port }),
            );
            driver
        }
        Err(err) => {
            report_event(
                &report_events,
                "driver.chromedriver_start",
                "failed",
                serde_json::json!({ "error": err.to_string() }),
            );
            return Err(err);
        }
    };
    log::info!("uto-test: started chromedriver at {}", driver.url);

    let session = match WebSession::new_with_args(
        &driver.url,
        &["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"],
    )
    .await
    {
        Ok(session) => {
            report_event(
                &report_events,
                "session.web_create",
                "ok",
                serde_json::json!({ "driver_url": driver.url }),
            );
            session
        }
        Err(err) => {
            report_event(
                &report_events,
                "session.web_create",
                "failed",
                serde_json::json!({ "driver_url": driver.url, "error": err.to_string() }),
            );
            let _ = driver.stop();
            return Err(err);
        }
    };

    Ok(ManagedSession::from_web(session, driver, report_events))
}

async fn start_android_session(
    android_api_level_hint: u16,
    report_events: Option<SharedEvents>,
) -> UtoResult<ManagedSession> {
    if android_api_level_hint > 0 {
        log::info!(
            "uto-test: android API hint {} requested (informational)",
            android_api_level_hint
        );
    }

    let setup = match prepare_mobile_environment(&MobileSetupOptions {
        require_online_device: true,
        ..MobileSetupOptions::default()
    }) {
        Ok(setup) => {
            report_event(
                &report_events,
                "env.mobile_setup",
                "ok",
                serde_json::json!({
                    "android_sdk_root": setup.android_sdk.root.display().to_string(),
                    "appium_path": setup.appium_path.display().to_string(),
                    "device_serial": setup.device_serial,
                }),
            );
            setup
        }
        Err(err) => {
            report_event(
                &report_events,
                "env.mobile_setup",
                "failed",
                serde_json::json!({ "error": err.to_string() }),
            );
            return Err(err);
        }
    };

    let appium = match driver::start_appium(&setup.appium_path).await {
        Ok(appium) => {
            report_event(
                &report_events,
                "driver.appium_start",
                "ok",
                serde_json::json!({ "url": appium.url, "port": appium.port }),
            );
            appium
        }
        Err(err) => {
            report_event(
                &report_events,
                "driver.appium_start",
                "failed",
                serde_json::json!({ "error": err.to_string() }),
            );
            return Err(err);
        }
    };
    log::info!("uto-test: started appium at {}", appium.url);

    let device_serial = setup
        .device_serial
        .unwrap_or_else(|| "emulator-5554".to_string());
    let caps = MobileCapabilities::android(device_serial);

    let session = match MobileSession::new(&appium.url, caps).await {
        Ok(session) => {
            report_event(
                &report_events,
                "session.mobile_create",
                "ok",
                serde_json::json!({ "driver_url": appium.url }),
            );
            session
        }
        Err(err) => {
            report_event(
                &report_events,
                "session.mobile_create",
                "failed",
                serde_json::json!({ "driver_url": appium.url, "error": err.to_string() }),
            );
            let _ = appium.stop();
            return Err(err);
        }
    };

    Ok(ManagedSession::from_mobile(session, appium, report_events))
}

#[cfg(test)]
mod tests {
    use super::{normalize_target, NormalizedTarget};

    #[test]
    fn normalize_target_accepts_aliases() {
        assert!(matches!(
            normalize_target("chrome").expect("chrome"),
            NormalizedTarget::Chrome
        ));
        assert!(matches!(
            normalize_target("web").expect("web"),
            NormalizedTarget::Chrome
        ));
        assert!(matches!(
            normalize_target("android").expect("android"),
            NormalizedTarget::Android
        ));
        assert!(matches!(
            normalize_target("mobile").expect("mobile"),
            NormalizedTarget::Android
        ));
    }

    #[test]
    fn normalize_target_rejects_unknown() {
        let err = normalize_target("ios").expect_err("unsupported");
        assert!(err.to_string().contains("Unsupported session target"));
    }
}
