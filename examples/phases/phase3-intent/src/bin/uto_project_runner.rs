use serde_json::json;
use uto_core::{
    driver,
    error::UtoError,
    env::{
        mobile_setup::{prepare_mobile_environment, MobileSetupOptions},
        platform::find_chrome_version,
        provisioning::find_or_provision_chromedriver,
    },
    session::{
        mobile::{MobileCapabilities, MobileSession},
        web::WebSession,
        UtoSession,
    },
};
use uto_runner::{CliOptions, Report, RunMode};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let options = CliOptions::from_env();
    let mut report = Report::new(options.report_json, options.report_file.clone(), options.mode);

    let result = match options.mode {
        RunMode::Web => run_web(&mut report).await,
        RunMode::Mobile => run_mobile(&mut report).await,
    };

    match &result {
        Ok(_) => report.finish("passed", None),
        Err(err) => report.finish("failed", Some(err.to_string())),
    }

    report.emit();

    if let Err(err) = result {
        eprintln!("Runner failed: {err}");
        std::process::exit(1);
    }
}

async fn run_web(report: &mut Report) -> uto_core::error::UtoResult<()> {
    let chrome_version = find_chrome_version()?;
    report.event(
        "env.chrome_discovery",
        "ok",
        json!({ "chrome_version": chrome_version }),
    );

    let chromedriver = find_or_provision_chromedriver(&chrome_version).await?;
    report.event(
        "env.chromedriver_provision",
        "ok",
        json!({ "path": chromedriver.display().to_string() }),
    );

    let driver = driver::start_chromedriver(&chromedriver).await?;
    report.event(
        "driver.chromedriver_start",
        "ok",
        json!({ "url": driver.url, "port": driver.port }),
    );

    let mut run_result = Ok(());
    match WebSession::new_with_args(
        &driver.url,
        &["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"],
    )
    .await
    {
        Ok(session) => {
            report.event("session.web_create", "ok", json!({ "driver_url": driver.url }));

            let action_result = web_intent_flow(&session, report).await;
            if let Err(err) = action_result {
                run_result = Err(err);
            }

            let _ = Box::new(session).close().await;
        }
        Err(err) => {
            report.event(
                "session.web_create",
                "failed",
                json!({ "error": err.to_string() }),
            );
            run_result = Err(err);
        }
    }

    let _ = driver.stop();
    report.event("driver.chromedriver_stop", "ok", json!({}));

    run_result
}

async fn web_intent_flow(session: &WebSession, report: &mut Report) -> uto_core::error::UtoResult<()> {
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
        .await?;

    report.event("session.goto", "ok", json!({ "target": "data:text/html" }));

    let ranking = session
        .debug_select_ranking("Submit Order", 3)
        .await
        .unwrap_or_else(|_| "<unavailable>".to_string());
    report.event(
        "intent.ranking",
        "ok",
        json!({ "label": "Submit Order", "summary": ranking }),
    );

    session.fill_intent("Email Address", "phase3@uto.dev").await?;
    report.event(
        "intent.fill",
        "ok",
        json!({ "label": "Email Address", "value": "phase3@uto.dev" }),
    );

    session.click_intent("Submit Order").await?;
    report.event(
        "intent.click",
        "ok",
        json!({ "label": "Submit Order" }),
    );

    let output = session.find_element("#out").await?;
    let text = session.get_text(&output).await?;

    if text != "phase3@uto.dev" {
        return Err(UtoError::SessionCommandFailed(format!(
            "Phase 3 web objective mismatch: expected 'phase3@uto.dev', got '{text}'"
        )));
    }

    report.event("assert.output", "ok", json!({ "text": text }));
    Ok(())
}

async fn run_mobile(report: &mut Report) -> uto_core::error::UtoResult<()> {
    let setup = prepare_mobile_environment(&MobileSetupOptions {
        require_online_device: true,
        ..MobileSetupOptions::default()
    })?;

    report.event(
        "env.mobile_setup",
        "ok",
        json!({
            "android_sdk_root": setup.android_sdk.root.display().to_string(),
            "appium_path": setup.appium_path.display().to_string(),
            "device_serial": setup.device_serial
        }),
    );

    let appium = driver::start_appium(&setup.appium_path).await?;
    report.event(
        "driver.appium_start",
        "ok",
        json!({ "url": appium.url, "port": appium.port }),
    );

    let caps = MobileCapabilities::android(
        setup
            .device_serial
            .unwrap_or_else(|| "emulator-5554".to_string()),
    );

    let mut run_result = Ok(());
    match MobileSession::new(&appium.url, caps).await {
        Ok(session) => {
            report.event(
                "session.mobile_create",
                "ok",
                json!({ "driver_url": appium.url }),
            );

            let action_result = mobile_intent_flow(&session, report).await;
            if let Err(err) = action_result {
                run_result = Err(err);
            }

            let _ = Box::new(session).close().await;
        }
        Err(err) => {
            report.event(
                "session.mobile_create",
                "failed",
                json!({ "error": err.to_string() }),
            );
            run_result = Err(err);
        }
    }

    let _ = appium.stop();
    report.event("driver.appium_stop", "ok", json!({}));

    run_result
}

async fn mobile_intent_flow(
    session: &MobileSession,
    report: &mut Report,
) -> uto_core::error::UtoResult<()> {
    session
        .launch_activity("com.android.settings", ".Settings")
        .await?;
    report.event(
        "session.launch_activity",
        "ok",
        json!({ "activity": "Settings" }),
    );

    for label in ["Search settings", "Search", "Rechercher", "Buscar", "Suchen"] {
        match session.select(label).await {
            Ok(_) => {
                report.event(
                    "intent.select",
                    "ok",
                    json!({ "label": label, "strategy": "mobile-accessibility" }),
                );
                return Ok(());
            }
            Err(err) => {
                report.event(
                    "intent.select",
                    "failed",
                    json!({ "label": label, "error": err.to_string() }),
                );
            }
        }
    }

    Err(UtoError::SessionCommandFailed(
        "Phase 3 mobile objective failed: no search label resolved".to_string(),
    ))
}
