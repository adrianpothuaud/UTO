use serde_json::json;
use uto_core::{
    driver,
    env::mobile_setup::{prepare_mobile_environment, MobileSetupOptions},
    error::{UtoError, UtoResult},
    session::{mobile::{MobileCapabilities, MobileSession}, UtoSession},
};
use uto_reporter::SuiteReport;

pub async fn run_suite(suite: &mut SuiteReport) -> UtoResult<()> {
    let mut failed = false;

    if run_launch_case(suite).await.is_err() {
        failed = true;
    }
    if run_search_case(suite).await.is_err() {
        failed = true;
    }

    if failed {
        Err(UtoError::SessionCommandFailed(
            "Phase 3 mobile suite failed".to_string(),
        ))
    } else {
        Ok(())
    }
}

async fn run_launch_case(suite: &mut SuiteReport) -> UtoResult<()> {
    let mut handle = suite.begin_test("mobile: settings activity launches");

    let setup = match prepare_mobile_environment(&MobileSetupOptions {
        require_online_device: true,
        ..MobileSetupOptions::default()
    }) {
        Ok(setup) => {
            handle.event(
                "env.mobile_setup",
                "ok",
                json!({
                    "android_sdk_root": setup.android_sdk.root.display().to_string(),
                    "appium_path": setup.appium_path.display().to_string(),
                    "device_serial": setup.device_serial,
                }),
            );
            setup
        }
        Err(err) => {
            let msg = err.to_string();
            handle.event("env.mobile_setup", "failed", json!({ "error": &msg }));
            suite.record_test(handle, "failed", Some(msg.clone()));
            return Err(err);
        }
    };

    let appium = match driver::start_appium(&setup.appium_path).await {
        Ok(appium) => {
            handle.event(
                "driver.appium_start",
                "ok",
                json!({ "url": appium.url, "port": appium.port }),
            );
            appium
        }
        Err(err) => {
            let msg = err.to_string();
            handle.event("driver.appium_start", "failed", json!({ "error": &msg }));
            suite.record_test(handle, "failed", Some(msg.clone()));
            return Err(err);
        }
    };

    let caps = MobileCapabilities::android(
        setup
            .device_serial
            .unwrap_or_else(|| "emulator-5554".to_string()),
    );

    let run_result: UtoResult<()> = async {
        let session = MobileSession::new(&appium.url, caps).await?;
        handle.event("session.mobile_create", "ok", json!({ "driver_url": appium.url }));

        session.launch_activity("com.android.settings", ".Settings").await?;
        handle.event(
            "session.launch_activity",
            "ok",
            json!({ "activity": "Settings" }),
        );

        Box::new(session).close().await?;
        Ok(())
    }
    .await;

    let stop_result = appium.stop();
    match &stop_result {
        Ok(_) => handle.event("driver.appium_stop", "ok", json!({})),
        Err(err) => handle.event("driver.appium_stop", "failed", json!({ "error": err.to_string() })),
    }

    match run_result {
        Ok(()) => {
            suite.record_test(handle, "passed", None);
            stop_result?;
            Ok(())
        }
        Err(err) => {
            let msg = err.to_string();
            suite.record_test(handle, "failed", Some(msg.clone()));
            let _ = stop_result;
            Err(err)
        }
    }
}

async fn run_search_case(suite: &mut SuiteReport) -> UtoResult<()> {
    let mut handle = suite.begin_test("mobile: search intent resolves from settings");

    let setup = match prepare_mobile_environment(&MobileSetupOptions {
        require_online_device: true,
        ..MobileSetupOptions::default()
    }) {
        Ok(setup) => {
            handle.event(
                "env.mobile_setup",
                "ok",
                json!({
                    "android_sdk_root": setup.android_sdk.root.display().to_string(),
                    "appium_path": setup.appium_path.display().to_string(),
                    "device_serial": setup.device_serial,
                }),
            );
            setup
        }
        Err(err) => {
            let msg = err.to_string();
            handle.event("env.mobile_setup", "failed", json!({ "error": &msg }));
            suite.record_test(handle, "failed", Some(msg.clone()));
            return Err(err);
        }
    };

    let appium = match driver::start_appium(&setup.appium_path).await {
        Ok(appium) => {
            handle.event(
                "driver.appium_start",
                "ok",
                json!({ "url": appium.url, "port": appium.port }),
            );
            appium
        }
        Err(err) => {
            let msg = err.to_string();
            handle.event("driver.appium_start", "failed", json!({ "error": &msg }));
            suite.record_test(handle, "failed", Some(msg.clone()));
            return Err(err);
        }
    };

    let caps = MobileCapabilities::android(
        setup
            .device_serial
            .unwrap_or_else(|| "emulator-5554".to_string()),
    );

    let run_result: UtoResult<()> = async {
        let session = MobileSession::new(&appium.url, caps).await?;
        handle.event("session.mobile_create", "ok", json!({ "driver_url": appium.url }));

        session.launch_activity("com.android.settings", ".Settings").await?;
        handle.event(
            "session.launch_activity",
            "ok",
            json!({ "activity": "Settings" }),
        );

        for label in ["Search settings", "Search", "Rechercher", "Buscar", "Suchen"] {
            match session.select(label).await {
                Ok(_) => {
                    handle.event(
                        "intent.select",
                        "ok",
                        json!({ "label": label, "strategy": "mobile-accessibility" }),
                    );
                    Box::new(session).close().await?;
                    return Ok(());
                }
                Err(err) => {
                    handle.event(
                        "intent.select",
                        "failed",
                        json!({ "label": label, "error": err.to_string() }),
                    );
                }
            }
        }

        let _ = Box::new(session).close().await;
        Err(UtoError::SessionCommandFailed(
            "Phase 3 mobile objective failed: no search label resolved".to_string(),
        ))
    }
    .await;

    let stop_result = appium.stop();
    match &stop_result {
        Ok(_) => handle.event("driver.appium_stop", "ok", json!({})),
        Err(err) => handle.event("driver.appium_stop", "failed", json!({ "error": err.to_string() })),
    }

    match run_result {
        Ok(()) => {
            suite.record_test(handle, "passed", None);
            stop_result?;
            Ok(())
        }
        Err(err) => {
            let msg = err.to_string();
            suite.record_test(handle, "failed", Some(msg.clone()));
            let _ = stop_result;
            Err(err)
        }
    }
}