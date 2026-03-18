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

use crate::managed_session::ManagedSession;

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
    match normalize_target(target)? {
        NormalizedTarget::Chrome => start_web_session().await,
        NormalizedTarget::Android => start_android_session(android_api_level_hint).await,
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

async fn start_web_session() -> UtoResult<ManagedSession> {
    let chrome_version = find_chrome_version()?;
    log::info!("uto-test: discovered chrome version {}", chrome_version);

    let chromedriver = find_or_provision_chromedriver(&chrome_version).await?;
    log::info!("uto-test: using chromedriver at {}", chromedriver.display());

    let driver = driver::start_chromedriver(&chromedriver).await?;
    log::info!("uto-test: started chromedriver at {}", driver.url);

    let session = WebSession::new_with_args(
        &driver.url,
        &["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"],
    )
    .await?;

    Ok(ManagedSession::from_web(session, driver))
}

async fn start_android_session(android_api_level_hint: u16) -> UtoResult<ManagedSession> {
    if android_api_level_hint > 0 {
        log::info!(
            "uto-test: android API hint {} requested (informational)",
            android_api_level_hint
        );
    }

    let setup = prepare_mobile_environment(&MobileSetupOptions {
        require_online_device: true,
        ..MobileSetupOptions::default()
    })?;

    let appium = driver::start_appium(&setup.appium_path).await?;
    log::info!("uto-test: started appium at {}", appium.url);

    let device_serial = setup
        .device_serial
        .unwrap_or_else(|| "emulator-5554".to_string());
    let caps = MobileCapabilities::android(device_serial);

    let session = MobileSession::new(&appium.url, caps).await?;

    Ok(ManagedSession::from_mobile(session, appium))
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
