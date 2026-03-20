use std::path::PathBuf;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use uto_ui::{start_server, UiOptions};

fn free_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind should succeed");
    let port = listener.local_addr().expect("local addr").port();
    drop(listener);
    port
}

async fn wait_for_server(port: u16) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    while tokio::time::Instant::now() < deadline {
        if TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(30)).await;
    }
    panic!("server did not become reachable on port {port}");
}

async fn launch_server(report: Option<PathBuf>) -> (u16, JoinHandle<Result<(), String>>) {
    launch_server_with_project(PathBuf::from("."), report).await
}

async fn launch_server_with_project(
    project: PathBuf,
    report: Option<PathBuf>,
) -> (u16, JoinHandle<Result<(), String>>) {
    let port = free_port();
    let opts = UiOptions {
        project,
        port,
        open: false,
        watch: false,
        report,
        studio: false,
    };

    let handle = tokio::spawn(async move { start_server(opts).await });
    wait_for_server(port).await;
    (port, handle)
}

async fn ws_connect(
    port: u16,
) -> tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>> {
    let url = format!("ws://127.0.0.1:{port}/ws");
    let (stream, _) = connect_async(url).await.expect("ws connect should succeed");
    stream
}

async fn ws_next_json(
    ws: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
) -> serde_json::Value {
    let msg = timeout(Duration::from_secs(5), ws.next())
        .await
        .expect("timed out waiting for ws message")
        .expect("ws stream ended")
        .expect("ws message error");

    match msg {
        Message::Text(text) => serde_json::from_str(text.as_str()).expect("valid JSON ws payload"),
        other => panic!("expected text ws message, got: {other:?}"),
    }
}

async fn http_get(port: u16, path: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port))
        .await
        .expect("tcp connect");
    let req = format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
    stream
        .write_all(req.as_bytes())
        .await
        .expect("write request");

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.expect("read response");
    String::from_utf8(buf).expect("utf8 response")
}

fn http_response_body(resp: &str) -> &str {
    resp.split("\r\n\r\n").nth(1).unwrap_or("")
}

#[tokio::test]
async fn ws_client_receives_report_on_connect() {
    let tmp = tempfile::NamedTempFile::new().expect("tmp report file");
    let report_json = serde_json::json!({
        "schema_version": "uto-report/v1",
        "status": "passed",
        "run_id": "integration-report"
    });
    std::fs::write(tmp.path(), report_json.to_string()).expect("write report");

    let (port, server) = launch_server(Some(tmp.path().to_path_buf())).await;

    let mut ws = ws_connect(port).await;
    let msg = ws_next_json(&mut ws).await;

    assert_eq!(msg["type"], "report");
    assert_eq!(msg["payload"]["run_id"], "integration-report");

    server.abort();
}

#[tokio::test]
async fn ws_trigger_run_emits_started_and_finished() {
    let (port, server) = launch_server(None).await;
    let mut ws = ws_connect(port).await;

    ws.send(Message::Text(
        serde_json::json!({ "type": "trigger_run" })
            .to_string()
            .into(),
    ))
    .await
    .expect("send trigger_run");

    let started = ws_next_json(&mut ws).await;
    assert_eq!(started["type"], "run_started");
    assert_eq!(started["payload"]["ts_ms"], 0);

    let mut saw_finished = false;
    for _ in 0..40 {
        let msg = ws_next_json(&mut ws).await;
        if msg["type"] == "run_finished" {
            saw_finished = true;
            assert!(msg["payload"]["ts_ms"].as_u64().is_some());
            break;
        }
    }

    assert!(saw_finished, "run_finished was not observed");
    server.abort();
}

#[tokio::test]
async fn ws_second_trigger_while_active_sends_run_ignored() {
    let (port, server) = launch_server(None).await;
    let mut ws = ws_connect(port).await;

    ws.send(Message::Text(
        serde_json::json!({ "type": "trigger_run" })
            .to_string()
            .into(),
    ))
    .await
    .expect("send first trigger_run");

    ws.send(Message::Text(
        serde_json::json!({ "type": "trigger_run" })
            .to_string()
            .into(),
    ))
    .await
    .expect("send second trigger_run");

    let mut saw_ignored = false;
    for _ in 0..20 {
        let msg = ws_next_json(&mut ws).await;
        if msg["type"] == "run_ignored" {
            saw_ignored = true;
            assert_eq!(msg["payload"]["reason"], "run_already_active");
            break;
        }
    }

    assert!(saw_ignored, "run_ignored was not observed");
    server.abort();
}

#[tokio::test]
async fn ws_broadcast_reaches_all_connected_clients() {
    let (port, server) = launch_server(None).await;
    let mut ws1 = ws_connect(port).await;
    let mut ws2 = ws_connect(port).await;

    ws1.send(Message::Text(
        serde_json::json!({ "type": "trigger_run" })
            .to_string()
            .into(),
    ))
    .await
    .expect("send trigger_run");

    let m1 = ws_next_json(&mut ws1).await;
    let m2 = ws_next_json(&mut ws2).await;

    assert_eq!(m1["type"], "run_started");
    assert_eq!(m2["type"], "run_started");

    server.abort();
}

#[tokio::test]
async fn ws_stop_run_reports_stopped_status() {
    let (port, server) = launch_server(None).await;
    let mut ws = ws_connect(port).await;

    let mut saw_stopped = false;
    for _ in 0..8 {
        ws.send(Message::Text(
            serde_json::json!({ "type": "trigger_run" })
                .to_string()
                .into(),
        ))
        .await
        .expect("send trigger_run");

        // Attempt to stop as early as possible to maximize the chance of
        // interrupting before natural process exit.
        ws.send(Message::Text(
            serde_json::json!({ "type": "stop_run" }).to_string().into(),
        ))
        .await
        .expect("send stop_run");

        for _ in 0..20 {
            let msg = ws_next_json(&mut ws).await;
            if msg["type"] == "run_finished" {
                if msg["payload"]["status"] == "stopped" {
                    saw_stopped = true;
                }
                break;
            }
        }

        if saw_stopped {
            break;
        }
    }

    assert!(
        saw_stopped,
        "run_finished with status=stopped was not observed"
    );
    server.abort();
}

#[tokio::test]
async fn ws_late_joiner_receives_preloaded_report() {
    let tmp = tempfile::NamedTempFile::new().expect("tmp report file");
    let report_json = serde_json::json!({
        "schema_version": "uto-report/v1",
        "status": "passed",
        "run_id": "late-join-report"
    });
    std::fs::write(tmp.path(), report_json.to_string()).expect("write report");

    let (port, server) = launch_server(Some(tmp.path().to_path_buf())).await;

    let mut ws1 = ws_connect(port).await;
    let first = ws_next_json(&mut ws1).await;
    assert_eq!(first["type"], "report");
    assert_eq!(first["payload"]["run_id"], "late-join-report");

    let mut ws2 = ws_connect(port).await;
    let second = ws_next_json(&mut ws2).await;
    assert_eq!(second["type"], "report");
    assert_eq!(second["payload"]["run_id"], "late-join-report");

    server.abort();
}

#[tokio::test]
async fn api_report_returns_updated_report_after_run() {
    let project_dir = tempfile::tempdir().expect("temp project dir");
    let report_dir = project_dir.path().join(".uto/reports");
    std::fs::create_dir_all(&report_dir).expect("create report dir");
    let seeded_report = serde_json::json!({
        "schema_version": "uto-report/v1",
        "status": "partial",
        "run_id": "seeded-report"
    });
    std::fs::write(report_dir.join("last-run.json"), seeded_report.to_string())
        .expect("seed report file");

    let (port, server) = launch_server_with_project(project_dir.path().to_path_buf(), None).await;
    let mut ws = ws_connect(port).await;

    ws.send(Message::Text(
        serde_json::json!({ "type": "trigger_run" })
            .to_string()
            .into(),
    ))
    .await
    .expect("send trigger_run");

    // Wait for completion so report_store has been updated from last-run.json.
    for _ in 0..40 {
        let msg = ws_next_json(&mut ws).await;
        if msg["type"] == "run_finished" {
            break;
        }
    }

    let resp = http_get(port, "/api/report").await;
    assert!(resp.contains("HTTP/1.1 200 OK"), "response was not 200 OK");
    let body = http_response_body(&resp);
    let json: serde_json::Value = serde_json::from_str(body).expect("valid api report json");
    assert_eq!(json["schema_version"], "uto-report/v1");
    assert_eq!(json["run_id"], "seeded-report");

    server.abort();
}

#[tokio::test]
async fn index_html_contains_required_structure() {
    let (port, server) = launch_server(None).await;

    let resp = http_get(port, "/").await;
    assert!(resp.contains("HTTP/1.1 200 OK"), "response was not 200 OK");
    assert!(resp.contains("id=\"btn-run\""));
    assert!(resp.contains("id=\"events-list\""));
    assert!(resp.contains("id=\"summary-bar\""));
    assert!(resp.contains("id=\"test-list\""));
    assert!(resp.contains("id=\"event-tools\""));
    assert!(resp.contains("id=\"event-view\""));
    assert!(resp.contains("id=\"event-only-failed\""));
    assert!(resp.contains("id=\"event-search\""));
    assert!(resp.contains("id=\"assets-wrap\""));
    assert!(resp.contains("id=\"assets-grid\""));

    server.abort();
}
