# uto-test API Usage Guide

Date: 2026-03-18

## Purpose

This guide documents the public helper API exposed by `uto-test` for authored tests and generated projects.

Use `uto-test` when you want concise setup and teardown with one managed session object while keeping environment and driver behavior observable through logs.

## Public API

Session bootstrap:

- `start_new_session(target: &str) -> UtoResult<ManagedSession>`
- `start_new_session_with_hint(target: &str, android_api_level_hint: u16) -> UtoResult<ManagedSession>`
- `startNewSession(target: &str) -> UtoResult<ManagedSession>`
- `startNewSessionWithArg(target: &str, arg: u16) -> UtoResult<ManagedSession>`

Managed session methods:

- `target(&self) -> &'static str`
- `goto(&self, url: &str) -> UtoResult<()>`
- `title(&self) -> UtoResult<String>`
- `find_element(&self, selector: &str) -> UtoResult<UtoElement>`
- `select(&self, label: &str) -> UtoResult<UtoElement>`
- `click_intent(&self, label: &str) -> UtoResult<()>`
- `fill_intent(&self, label: &str, value: &str) -> UtoResult<()>`
- `get_text(&self, element: &UtoElement) -> UtoResult<String>`
- `launch_android_activity(&self, app_package: &str, app_activity: &str) -> UtoResult<()>`
- `close(self) -> UtoResult<()>`

## Supported Targets

Accepted target aliases:

- Web: `chrome`, `web`
- Mobile: `android`, `mobile`

Unknown targets return `UtoError::SessionCommandFailed` with a clear message.

## Minimal Patterns

### Web

```rust
#[tokio::test]
async fn web_flow() {
    let session = match uto_test::startNewSession("chrome").await {
        Ok(session) => session,
        Err(err) => {
            println!("Skipping web flow: {err}");
            return;
        }
    };

    session.goto("https://example.com").await.expect("goto");
    let title = session.title().await.expect("title");
    assert!(!title.is_empty());

    session.close().await.expect("close session");
}
```

### Mobile

```rust
#[tokio::test]
async fn mobile_flow() {
    let session = match uto_test::startNewSessionWithArg("android", 16).await {
        Ok(session) => session,
        Err(err) => {
            println!("Skipping mobile flow: {err}");
            return;
        }
    };

    let _ = session
        .launch_android_activity("com.android.settings", ".Settings")
        .await;

    session.close().await.expect("close session");
}
```

### Intent-Oriented Interaction

```rust
let email = "phase4@uto.dev";
session.fill_intent("Email Address", email).await?;
session.click_intent("Submit Order").await?;
let out = session.find_element("#out").await?;
let text = session.get_text(&out).await?;
assert_eq!(text, email);
```

## Generated Project Compatibility

CLI-generated projects now include both helper and runner crates:

- `uto-test` for authored tests
- `uto-runner` for structured report lifecycle in `src/bin/uto_project_runner.rs`

Compatibility is validated by integration tests in `uto-cli/tests/generated_project_compat.rs` using real `uto init` scaffolding plus `cargo check --tests`.

## Relation To ADRs

- ADR 0011 defines the `uto-test` extraction and the backlog completed by this guide.
- ADR 0010 tracks Phase 4.1 framework maturity progress.
