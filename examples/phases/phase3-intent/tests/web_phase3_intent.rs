#[tokio::test]
async fn web_phase3_intent_flow_or_skip() {
    let session = match uto_test::startNewSession("chrome").await {
        Ok(session) => session,
        Err(err) => {
            println!("Skipping web Phase 3 example: could not create web session: {err}");
            return;
        }
    };

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

    session.close().await.expect("close");
}
