//! Integration tests for the UTO session layer.
//!
//! ## Structure
//!
//! | Test group | What is exercised |
//! |---|---|
//! | `mobile_capabilities_*` | `MobileCapabilities` / `MobilePlatform` serialisation — **pure unit tests**, no I/O |
//! | `web_session_*` | `WebSession` against a real ChromeDriver + headless Chrome |
//!
//! The `web_session_*` tests require Chrome and ChromeDriver to be installed.
//! They skip gracefully when ChromeDriver is not found on the host.
//!
//! Run the external-network test (requires internet) with:
//! ```sh
//! cargo test -- --ignored
//! ```

use std::path::PathBuf;

use uto_core::session::mobile::{MobileCapabilities, MobilePlatform};
use uto_core::session::web::WebSession;
use uto_core::session::UtoSession;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Chrome arguments for headless / sandboxless environments (e.g. CI, Docker).
const HEADLESS_ARGS: &[&str] =
    &["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"];

/// Returns the path to a `chromedriver` binary, or `None` if not found.
fn find_chromedriver() -> Option<PathBuf> {
    for p in &["/usr/bin/chromedriver", "/usr/local/bin/chromedriver"] {
        let path = PathBuf::from(p);
        if path.exists() {
            return Some(path);
        }
    }
    which::which("chromedriver").ok()
}

/// Starts the system ChromeDriver or returns `None` so the test can skip.
async fn start_system_chromedriver() -> Option<uto_core::driver::DriverProcess> {
    let path = find_chromedriver()?;
    uto_core::driver::start_chromedriver(&path).await.ok()
}

// ---------------------------------------------------------------------------
// MobileCapabilities — pure unit tests (no I/O, no network)
// ---------------------------------------------------------------------------

#[test]
fn mobile_capabilities_android_minimal() {
    let caps = MobileCapabilities::android("emulator-5554");
    let json = caps.to_json_pub();

    assert_eq!(json["platformName"], "Android");
    assert_eq!(json["appium:deviceName"], "emulator-5554");
    assert_eq!(json["appium:automationName"], "UiAutomator2");
    assert!(json.get("appium:platformVersion").is_none());
    assert!(json.get("appium:app").is_none());
}

#[test]
fn mobile_capabilities_ios_minimal() {
    let caps = MobileCapabilities::ios("iPhone 14");
    let json = caps.to_json_pub();

    assert_eq!(json["platformName"], "iOS");
    assert_eq!(json["appium:deviceName"], "iPhone 14");
    assert_eq!(json["appium:automationName"], "XCUITest");
}

#[test]
fn mobile_capabilities_with_platform_version() {
    let caps = MobileCapabilities::android("emulator-5554").with_platform_version("13.0");
    let json = caps.to_json_pub();

    assert_eq!(json["appium:platformVersion"], "13.0");
}

#[test]
fn mobile_capabilities_with_app() {
    let caps = MobileCapabilities::android("emulator-5554").with_app("/data/app.apk");
    let json = caps.to_json_pub();

    assert_eq!(json["appium:app"], "/data/app.apk");
}

#[test]
fn mobile_capabilities_with_extra() {
    let caps = MobileCapabilities::android("emulator-5554")
        .with_extra("appium:noReset", serde_json::json!(true));
    let json = caps.to_json_pub();

    assert_eq!(json["appium:noReset"], true);
}

#[test]
fn mobile_platform_names_and_automation() {
    assert_eq!(MobilePlatform::Android.platform_name(), "Android");
    assert_eq!(MobilePlatform::Android.automation_name(), "UiAutomator2");
    assert_eq!(MobilePlatform::Ios.platform_name(), "iOS");
    assert_eq!(MobilePlatform::Ios.automation_name(), "XCUITest");
}

// ---------------------------------------------------------------------------
// WebSession — error path (no driver present)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn web_session_new_fails_when_no_driver_running() {
    // Port 1 is reserved; nothing should be listening.
    let result = WebSession::new("http://127.0.0.1:1").await;
    assert!(
        result.is_err(),
        "WebSession::new must fail when no driver is present"
    );
}

// ---------------------------------------------------------------------------
// WebSession — real ChromeDriver integration tests
//
// Each test skips gracefully when ChromeDriver is not installed on the host.
// HTML is loaded via inline `data:` URIs so the tests have no network dependency.
// ---------------------------------------------------------------------------

/// Verify that a session can be created and closed.
#[tokio::test]
async fn web_session_creates_and_closes_with_headless_chrome() {
    let Some(driver_proc) = start_system_chromedriver().await else {
        println!("Skipping: chromedriver not available");
        return;
    };

    let session = WebSession::new_with_args(&driver_proc.url, HEADLESS_ARGS)
        .await
        .expect("session creation should succeed");

    Box::new(session).close().await.expect("close should succeed");
    driver_proc.stop().unwrap();
}

/// Navigate to an inline page and verify the title is read correctly.
#[tokio::test]
async fn web_session_navigate_and_read_title() {
    let Some(driver_proc) = start_system_chromedriver().await else {
        println!("Skipping: chromedriver not available");
        return;
    };

    let session = WebSession::new_with_args(&driver_proc.url, HEADLESS_ARGS)
        .await
        .expect("session creation");

    session
        .goto("data:text/html,<html><head><title>UTO Test</title></head><body></body></html>")
        .await
        .expect("goto");

    let title = session.title().await.expect("title");
    assert_eq!(title, "UTO Test");

    Box::new(session).close().await.unwrap();
    driver_proc.stop().unwrap();
}

/// Find an element and read its text content.
#[tokio::test]
async fn web_session_find_element_and_get_text() {
    let Some(driver_proc) = start_system_chromedriver().await else {
        println!("Skipping: chromedriver not available");
        return;
    };

    let session = WebSession::new_with_args(&driver_proc.url, HEADLESS_ARGS)
        .await
        .expect("session creation");

    session
        .goto("data:text/html,<html><body><h1>Hello UTO</h1></body></html>")
        .await
        .expect("goto");

    let heading = session.find_element("h1").await.expect("find h1");
    let text = session.get_text(&heading).await.expect("get_text");
    assert_eq!(text, "Hello UTO");

    Box::new(session).close().await.unwrap();
    driver_proc.stop().unwrap();
}

/// Click a button and verify the DOM change it triggers.
#[tokio::test]
async fn web_session_click_element() {
    let Some(driver_proc) = start_system_chromedriver().await else {
        println!("Skipping: chromedriver not available");
        return;
    };

    let session = WebSession::new_with_args(&driver_proc.url, HEADLESS_ARGS)
        .await
        .expect("session creation");

    session
        .goto(concat!(
            "data:text/html,",
            "<html><body>",
            "<button id='btn' onclick=\"document.getElementById('out').textContent='clicked'\">",
            "Click</button>",
            "<p id='out'>initial</p>",
            "</body></html>"
        ))
        .await
        .expect("goto");

    let btn = session.find_element("#btn").await.expect("find button");
    session.click(&btn).await.expect("click");

    let out = session.find_element("#out").await.expect("find output");
    let text = session.get_text(&out).await.expect("get_text after click");
    assert_eq!(text, "clicked");

    Box::new(session).close().await.unwrap();
    driver_proc.stop().unwrap();
}

/// Type text into an input and verify no error is returned.
#[tokio::test]
async fn web_session_type_text_into_input() {
    let Some(driver_proc) = start_system_chromedriver().await else {
        println!("Skipping: chromedriver not available");
        return;
    };

    let session = WebSession::new_with_args(&driver_proc.url, HEADLESS_ARGS)
        .await
        .expect("session creation");

    session
        .goto("data:text/html,<html><body><input id='inp' type='text'/></body></html>")
        .await
        .expect("goto");

    let input = session.find_element("#inp").await.expect("find input");
    session
        .type_text(&input, "hello UTO")
        .await
        .expect("type_text should not error");

    Box::new(session).close().await.unwrap();
    driver_proc.stop().unwrap();
}

/// Capture a screenshot and verify the PNG signature.
#[tokio::test]
async fn web_session_screenshot_returns_png() {
    let Some(driver_proc) = start_system_chromedriver().await else {
        println!("Skipping: chromedriver not available");
        return;
    };

    let session = WebSession::new_with_args(&driver_proc.url, HEADLESS_ARGS)
        .await
        .expect("session creation");

    session
        .goto("data:text/html,<html><body><p>screenshot test</p></body></html>")
        .await
        .expect("goto");

    let png = session.screenshot().await.expect("screenshot");
    assert!(!png.is_empty(), "screenshot must not be empty");
    // All PNG files start with the 8-byte PNG magic number.
    assert_eq!(&png[0..4], b"\x89PNG", "screenshot must be a valid PNG");

    Box::new(session).close().await.unwrap();
    driver_proc.stop().unwrap();
}

// ---------------------------------------------------------------------------
// External-network navigation — off by default, requires internet access
// ---------------------------------------------------------------------------

/// Full navigation test against a live external website.
///
/// Run with: `cargo test -- --ignored web_session_real_external_navigation`
#[tokio::test]
#[ignore]
async fn web_session_real_external_navigation() {
    use uto_core::driver;
    use uto_core::env::{
        platform::find_chrome_version, provisioning::find_or_provision_chromedriver,
    };

    let chrome_version = find_chrome_version().expect("Chrome must be installed");
    let driver_path = find_or_provision_chromedriver(&chrome_version)
        .await
        .expect("ChromeDriver must be provisionable");

    let driver_proc = driver::start_chromedriver(&driver_path)
        .await
        .expect("ChromeDriver must start");

    let session = WebSession::new_with_args(&driver_proc.url, HEADLESS_ARGS)
        .await
        .expect("session must be created");

    session.goto("https://example.com").await.unwrap();
    let title = session.title().await.unwrap();
    assert!(title.contains("Example"), "unexpected title: {title}");

    let h1 = session.find_element("h1").await.unwrap();
    let text = session.get_text(&h1).await.unwrap();
    assert!(!text.is_empty());

    let png = session.screenshot().await.unwrap();
    assert!(!png.is_empty());

    Box::new(session).close().await.unwrap();
    driver_proc.stop().unwrap();
}
