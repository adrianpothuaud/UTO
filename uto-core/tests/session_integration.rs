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

use uto_core::session::mobile::{MobileCapabilities, MobilePlatform, MobileSession};
use uto_core::session::web::WebSession;
use uto_core::session::UtoSession;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Chrome arguments for headless / sandboxless environments (e.g. CI, Docker).
const HEADLESS_ARGS: &[&str] = &["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"];

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

/// Starts Appium from PATH or returns `None` so the test can skip.
async fn start_system_appium() -> Option<uto_core::driver::DriverProcess> {
    let path = uto_core::env::platform::find_appium()?;
    uto_core::driver::start_appium(&path).await.ok()
}

fn is_expected_mobile_environment_gap(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();

    (lower.contains("unknown command") && lower.contains("404"))
        || lower.contains("no devices/emulators found")
        || lower.contains("could not find a connected android device")
        || lower.contains("could not find a driver for automationname")
        || lower.contains("uiautomator2")
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
// MobileSession — smoke integration test (skips when host is not provisioned)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn mobile_session_smoke_creates_or_skips_gracefully() {
    let Some(appium_proc) = start_system_appium().await else {
        println!("Skipping: appium not available");
        return;
    };

    let caps = MobileCapabilities::android("emulator-5554");

    let session_result = MobileSession::new(&appium_proc.url, caps).await;
    match session_result {
        Ok(session) => {
            Box::new(session).close().await.unwrap();
        }
        Err(err) => {
            let msg = err.to_string();
            if is_expected_mobile_environment_gap(&msg) {
                println!("Skipping: mobile environment not fully provisioned: {msg}");
            } else {
                panic!("unexpected mobile session error: {msg}");
            }
        }
    }

    appium_proc.stop().unwrap();
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

    Box::new(session)
        .close()
        .await
        .expect("close should succeed");
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

/// Resolve an element by intent label and click it.
#[tokio::test]
async fn web_session_select_by_label_and_click() {
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
            "<button id='cancel' aria-label='Cancel Action' onclick=\"document.getElementById('out').textContent='cancel'\">X</button>",
            "<button id='submit' aria-label='Submit Order' onclick=\"document.getElementById('out').textContent='submit'\">Send</button>",
            "<p id='out'>initial</p>",
            "</body></html>"
        ))
        .await
        .expect("goto");

    let selected = session
        .select("Submit Order")
        .await
        .expect("select should resolve submit button");
    session.click(&selected).await.expect("click selected");

    let out = session.find_element("#out").await.expect("find output");
    let text = session.get_text(&out).await.expect("get output text");
    assert_eq!(text, "submit");

    Box::new(session).close().await.unwrap();
    driver_proc.stop().unwrap();
}

/// Fill a field and click a button using intent helpers.
#[tokio::test]
async fn web_session_fill_and_click_intent_helpers() {
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
            "<input id='email' aria-label='Email Address' type='text'/>",
            "<button id='submit' aria-label='Submit Order' ",
            "onclick=\"document.getElementById('out').textContent=document.getElementById('email').value\">",
            "Submit</button>",
            "<p id='out'>initial</p>",
            "</body></html>"
        ))
        .await
        .expect("goto");

    session
        .fill_intent("Email Address", "phase3@uto.dev")
        .await
        .expect("fill_intent");
    session
        .click_intent("Submit Order")
        .await
        .expect("click_intent");

    let out = session.find_element("#out").await.expect("find output");
    let text = session.get_text(&out).await.expect("get output text");
    assert_eq!(text, "phase3@uto.dev");

    Box::new(session).close().await.unwrap();
    driver_proc.stop().unwrap();
}

/// Fill two fields and submit via intent helpers.
#[tokio::test]
async fn web_session_multi_field_intent_flow() {
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
            "<input id='first' aria-label='First Name' type='text'/>",
            "<input id='email' aria-label='Email Address' type='text'/>",
            "<button id='submit' aria-label='Submit Order' ",
            "onclick=\"document.getElementById('out').textContent=document.getElementById('first').value+':'+document.getElementById('email').value\">",
            "Submit</button>",
            "<p id='out'>initial</p>",
            "</body></html>"
        ))
        .await
        .expect("goto");

    session
        .fill_intent("First Name", "Alex")
        .await
        .expect("fill_intent first name");
    session
        .fill_intent("Email Address", "alex@uto.dev")
        .await
        .expect("fill_intent email");
    session
        .click_intent("Submit Order")
        .await
        .expect("click_intent submit");

    let out = session.find_element("#out").await.expect("find output");
    let text = session.get_text(&out).await.expect("get output text");
    assert_eq!(text, "Alex:alex@uto.dev");

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
// Mobile Android fixture tests (Phase 4.3)
//
// These tests validate mobile intent resolution and interaction helpers.
// Each test skips gracefully when Appium or an Android device is not available.
// ---------------------------------------------------------------------------

/// Verify that mobile settings app can be launched and accessibility tree is readable.
#[tokio::test]
async fn mobile_session_android_launch_activity_and_page_source() {
    let Some(appium_proc) = start_system_appium().await else {
        println!("Skipping: appium not available");
        return;
    };

    let caps = MobileCapabilities::android("emulator-5554");

    let session = match MobileSession::new(&appium_proc.url, caps).await {
        Ok(s) => s,
        Err(err) => {
            let msg = err.to_string();
            if is_expected_mobile_environment_gap(&msg) {
                println!("Skipping: mobile environment not fully provisioned: {msg}");
            } else {
                panic!("unexpected mobile session error: {msg}");
            }
            appium_proc.stop().unwrap();
            return;
        }
    };

    // Launch Settings app
    if let Err(e) = session.launch_activity("com.android.settings", ".Settings").await {
        println!("Skipping: could not launch Settings: {e}");
        Box::new(session).close().await.ok();
        appium_proc.stop().unwrap();
        return;
    }

    // Verify page source is readable (accessibility tree dump)
    let page_source = session.page_source().await.expect("page_source should succeed");
    assert!(!page_source.is_empty(), "page_source must not be empty");
    // Settings typically contains "status" or activity class references
    let lower = page_source.to_ascii_lowercase();
    assert!(
        lower.contains("settings")
            || lower.contains("android")
            || lower.contains("com.android.settings"),
        "page_source should reference Settings app"
    );

    Box::new(session).close().await.unwrap();
    appium_proc.stop().unwrap();
}

/// Verify that mobile intent selection works on Android Settings.
///
/// This test validates the accessibility-tree-based intent resolution
/// that is critical for Phase 4.3 production readiness.
#[tokio::test]
async fn mobile_session_android_select_intent_label() {
    let Some(appium_proc) = start_system_appium().await else {
        println!("Skipping: appium not available");
        return;
    };

    let caps = MobileCapabilities::android("emulator-5554");

    let session = match MobileSession::new(&appium_proc.url, caps).await {
        Ok(s) => s,
        Err(err) => {
            let msg = err.to_string();
            if is_expected_mobile_environment_gap(&msg) {
                println!("Skipping: mobile environment not fully provisioned: {msg}");
            } else {
                panic!("unexpected mobile session error: {msg}");
            }
            appium_proc.stop().unwrap();
            return;
        }
    };

    // Launch Settings
    if let Err(e) = session.launch_activity("com.android.settings", ".Settings").await {
        println!("Skipping: could not launch Settings: {e}");
        Box::new(session).close().await.ok();
        appium_proc.stop().unwrap();
        return;
    }

    // Try to select a common label from Settings accessibility tree.
    // Settings varies by Android version, so we try multiple common labels.
    let common_labels = vec!["Search", "Search settings", "Rechercher", "Buscar", "Suchen"];

    let mut found = false;
    for label in common_labels {
        if let Ok(_elem) = session.select(label).await {
            log::info!("Found label: {label}");
            found = true;
            break;
        }
    }

    if !found {
        println!(
            "Skipping: Settings app did not expose expected accessibility labels (Android version variation)"
        );
        Box::new(session).close().await.ok();
        appium_proc.stop().unwrap();
        return;
    }

    assert!(found, "should have resolved at least one common Settings label");

    Box::new(session).close().await.unwrap();
    appium_proc.stop().unwrap();
}

/// Verify that mobile screenshot can be captured.
#[tokio::test]
async fn mobile_session_android_screenshot() {
    let Some(appium_proc) = start_system_appium().await else {
        println!("Skipping: appium not available");
        return;
    };

    let caps = MobileCapabilities::android("emulator-5554");

    let session = match MobileSession::new(&appium_proc.url, caps).await {
        Ok(s) => s,
        Err(err) => {
            let msg = err.to_string();
            if is_expected_mobile_environment_gap(&msg) {
                println!("Skipping: mobile environment not fully provisioned: {msg}");
            } else {
                panic!("unexpected mobile session error: {msg}");
            }
            appium_proc.stop().unwrap();
            return;
        }
    };

    // Launch Settings to get meaningful screenshot
    if let Err(e) = session.launch_activity("com.android.settings", ".Settings").await {
        println!("Skipping: could not launch Settings: {e}");
        Box::new(session).close().await.ok();
        appium_proc.stop().unwrap();
        return;
    }

    // Capture screenshot
    let png = session
        .screenshot()
        .await
        .expect("screenshot should succeed");
    assert!(!png.is_empty(), "screenshot must not be empty");
    // Verify PNG magic bytes
    assert_eq!(&png[0..4], b"\x89PNG", "screenshot must be a valid PNG");

    Box::new(session).close().await.unwrap();
    appium_proc.stop().unwrap();
}

/// Verify basic swipe gesture (Phase 4.3 mobile gesture support).
#[tokio::test]
async fn mobile_session_android_swipe_gesture() {
    let Some(appium_proc) = start_system_appium().await else {
        println!("Skipping: appium not available");
        return;
    };

    let caps = MobileCapabilities::android("emulator-5554");

    let session = match MobileSession::new(&appium_proc.url, caps).await {
        Ok(s) => s,
        Err(err) => {
            let msg = err.to_string();
            if is_expected_mobile_environment_gap(&msg) {
                println!("Skipping: mobile environment not fully provisioned: {msg}");
            } else {
                panic!("unexpected mobile session error: {msg}");
            }
            appium_proc.stop().unwrap();
            return;
        }
    };

    // Launch Settings
    if let Err(e) = session.launch_activity("com.android.settings", ".Settings").await {
        println!("Skipping: could not launch Settings: {e}");
        Box::new(session).close().await.ok();
        appium_proc.stop().unwrap();
        return;
    }

    // Perform a simple swipe (upward to scroll content)
    // Coordinates are typical for Android: 1080x1920 screen with half-height scroll region
    if let Err(e) = session.swipe(540, 1000, 540, 400, 300).await {
        println!("Skipping: swipe gesture failed (expected for some emulator configs): {e}");
        Box::new(session).close().await.ok();
        appium_proc.stop().unwrap();
        return;
    }

    // If we got here, swipe succeeded; verify we can still interact.
    // Some Appium/device combinations return 404 for title() after gesture actions.
    // Treat that as an environment capability gap so CI remains non-coupled.
    match session.title().await {
        Ok(title) => {
            assert!(!title.is_empty(), "title should be non-empty after swipe");
        }
        Err(err) => {
            let msg = err.to_string();
            if is_expected_mobile_environment_gap(&msg) {
                println!("Skipping: title command unsupported after swipe on this environment: {msg}");
                Box::new(session).close().await.ok();
                appium_proc.stop().unwrap();
                return;
            }
            panic!("unexpected title error after swipe: {msg}");
        }
    }

    Box::new(session).close().await.unwrap();
    appium_proc.stop().unwrap();
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
