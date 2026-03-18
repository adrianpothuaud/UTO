use serde_json::json;
use uto_core::{
    driver,
    env::{platform::find_chrome_version, provisioning::find_or_provision_chromedriver},
    error::{UtoError, UtoResult},
    session::{web::WebSession, UtoSession},
};
use uto_reporter::SuiteReport;

pub async fn run_suite(suite: &mut SuiteReport) -> UtoResult<()> {
    let mut failed = false;

    if run_fill_and_click_case(suite).await.is_err() {
        failed = true;
    }
    if run_ranking_case(suite).await.is_err() {
        failed = true;
    }

    if failed {
        Err(UtoError::SessionCommandFailed(
            "Phase 3 web suite failed".to_string(),
        ))
    } else {
        Ok(())
    }
}

async fn run_fill_and_click_case(suite: &mut SuiteReport) -> UtoResult<()> {
    let mut handle = suite.begin_test("web: fill and click intent updates output");

    let chrome_version = match find_chrome_version() {
        Ok(version) => {
            handle.event(
                "env.chrome_discovery",
                "ok",
                json!({ "chrome_version": version }),
            );
            version
        }
        Err(err) => {
            let msg = err.to_string();
            handle.event("env.chrome_discovery", "failed", json!({ "error": &msg }));
            suite.record_test(handle, "failed", Some(msg.clone()));
            return Err(err);
        }
    };

    let chromedriver_path = match find_or_provision_chromedriver(&chrome_version).await {
        Ok(path) => {
            handle.event(
                "env.chromedriver_provision",
                "ok",
                json!({ "path": path.display().to_string() }),
            );
            path
        }
        Err(err) => {
            let msg = err.to_string();
            handle.event(
                "env.chromedriver_provision",
                "failed",
                json!({ "error": &msg }),
            );
            suite.record_test(handle, "failed", Some(msg.clone()));
            return Err(err);
        }
    };

    let driver_proc = match driver::start_chromedriver(&chromedriver_path).await {
        Ok(proc) => {
            handle.event(
                "driver.chromedriver_start",
                "ok",
                json!({ "url": proc.url, "port": proc.port }),
            );
            proc
        }
        Err(err) => {
            let msg = err.to_string();
            handle.event(
                "driver.chromedriver_start",
                "failed",
                json!({ "error": &msg }),
            );
            suite.record_test(handle, "failed", Some(msg.clone()));
            return Err(err);
        }
    };

    let run_result: UtoResult<()> = async {
        let session = WebSession::new_with_args(
            &driver_proc.url,
            &["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"],
        )
        .await
        .map_err(|err| UtoError::SessionCommandFailed(err.to_string()))?;

        handle.event("session.web_create", "ok", json!({ "driver_url": driver_proc.url }));

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
        handle.event("session.goto", "ok", json!({ "target": "inline_form" }));

        session.fill_intent("Email Address", "phase3@uto.dev").await?;
        handle.event(
            "intent.fill",
            "ok",
            json!({ "label": "Email Address", "value": "phase3@uto.dev" }),
        );

        session.click_intent("Submit Order").await?;
        handle.event("intent.click", "ok", json!({ "label": "Submit Order" }));

        let output = session.find_element("#out").await?;
        let text = session.get_text(&output).await?;
        if text != "phase3@uto.dev" {
            return Err(UtoError::SessionCommandFailed(format!(
                "Phase 3 web objective mismatch: expected 'phase3@uto.dev', got '{text}'"
            )));
        }
        handle.event("assert.output", "ok", json!({ "text": text }));

        Box::new(session).close().await?;
        Ok(())
    }
    .await;

    let stop_result = driver_proc.stop();
    match &stop_result {
        Ok(_) => handle.event("driver.chromedriver_stop", "ok", json!({})),
        Err(err) => handle.event(
            "driver.chromedriver_stop",
            "failed",
            json!({ "error": err.to_string() }),
        ),
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

async fn run_ranking_case(suite: &mut SuiteReport) -> UtoResult<()> {
    let mut handle = suite.begin_test("web: ranking diagnostics stay readable");

    let chrome_version = match find_chrome_version() {
        Ok(version) => {
            handle.event(
                "env.chrome_discovery",
                "ok",
                json!({ "chrome_version": version }),
            );
            version
        }
        Err(err) => {
            let msg = err.to_string();
            handle.event("env.chrome_discovery", "failed", json!({ "error": &msg }));
            suite.record_test(handle, "failed", Some(msg.clone()));
            return Err(err);
        }
    };

    let chromedriver_path = match find_or_provision_chromedriver(&chrome_version).await {
        Ok(path) => {
            handle.event(
                "env.chromedriver_provision",
                "ok",
                json!({ "path": path.display().to_string() }),
            );
            path
        }
        Err(err) => {
            let msg = err.to_string();
            handle.event(
                "env.chromedriver_provision",
                "failed",
                json!({ "error": &msg }),
            );
            suite.record_test(handle, "failed", Some(msg.clone()));
            return Err(err);
        }
    };

    let driver_proc = match driver::start_chromedriver(&chromedriver_path).await {
        Ok(proc) => {
            handle.event(
                "driver.chromedriver_start",
                "ok",
                json!({ "url": proc.url, "port": proc.port }),
            );
            proc
        }
        Err(err) => {
            let msg = err.to_string();
            handle.event(
                "driver.chromedriver_start",
                "failed",
                json!({ "error": &msg }),
            );
            suite.record_test(handle, "failed", Some(msg.clone()));
            return Err(err);
        }
    };

    let run_result: UtoResult<()> = async {
        let session = WebSession::new_with_args(
            &driver_proc.url,
            &["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"],
        )
        .await
        .map_err(|err| UtoError::SessionCommandFailed(err.to_string()))?;

        handle.event(
            "session.web_create",
            "ok",
            json!({ "driver_url": driver_proc.url }),
        );

        session
            .goto(concat!(
                "data:text/html,",
                "<html><head><title>UTO Intent Ranking</title></head><body>",
                "<button aria-label='Checkout'>Checkout</button>",
                "<button aria-label='Cancel'>Cancel</button>",
                "</body></html>"
            ))
            .await?;
        handle.event("session.goto", "ok", json!({ "target": "ranking_page" }));

        let ranking = session.debug_select_ranking("Checkout", 3).await?;
        if !ranking.contains("Checkout") {
            return Err(UtoError::SessionCommandFailed(format!(
                "ranking assertion failed: expected Checkout in ranking summary, got '{ranking}'"
            )));
        }
        handle.event(
            "intent.ranking",
            "ok",
            json!({ "label": "Checkout", "summary": ranking }),
        );

        let title = session.title().await?;
        if title != "UTO Intent Ranking" {
            return Err(UtoError::SessionCommandFailed(format!(
                "title assertion failed: expected 'UTO Intent Ranking', got '{title}'"
            )));
        }
        handle.event("assert.title", "ok", json!({ "title": title }));

        Box::new(session).close().await?;
        Ok(())
    }
    .await;

    let stop_result = driver_proc.stop();
    match &stop_result {
        Ok(_) => handle.event("driver.chromedriver_stop", "ok", json!({})),
        Err(err) => handle.event(
            "driver.chromedriver_stop",
            "failed",
            json!({ "error": err.to_string() }),
        ),
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
