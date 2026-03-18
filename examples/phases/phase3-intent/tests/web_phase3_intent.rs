use uto_core::session::web::WebSession;
use uto_core::session::UtoSession;

#[tokio::test]
async fn web_phase3_intent_flow_or_skip() {
    let chromedriver = match which::which("chromedriver") {
        Ok(path) => path,
        Err(_) => {
            println!("Skipping web Phase 3 example: chromedriver not available");
            return;
        }
    };

    let driver = match uto_core::driver::start_chromedriver(&chromedriver).await {
        Ok(proc) => proc,
        Err(err) => {
            println!("Skipping web Phase 3 example: could not start chromedriver: {err}");
            return;
        }
    };

    let session = WebSession::new_with_args(
        &driver.url,
        &["--headless=new", "--no-sandbox", "--disable-dev-shm-usage"],
    )
    .await
    .expect("web session");

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

    Box::new(session).close().await.expect("close");
    driver.stop().expect("stop driver");
}
