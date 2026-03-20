# Phase 6 Studio — Reference Project

This is the UTO Phase 6 reference project demonstrating UTO Studio capabilities.

## What's in this project

- `tests/studio_workflow.rs` — tests validating the Studio recording and code generation pipeline
- `tests/schema_compat.rs` — schema compatibility test (suite report round-trip)
- `src/lib.rs` — Studio integration helpers

## Running

```sh
# From the workspace root:
cargo test -p phase6-studio

# Launch Studio mode in the UI server:
cargo run -p uto-cli -- ui --project examples/phases/phase6-studio --studio --port 4006 --open
```

## What is UTO Studio?

UTO Studio is the Phase 6 visual test authoring environment. Activated via
`uto ui --studio`, it enables:

1. **Live interaction recording** — click, type, navigate, and swipe interactions
   are captured as semantic `uto-test` intent steps (no CSS selectors).
2. **Rust code generation** — the recorded steps are rendered into a complete,
   runnable Rust test function using the `uto-test` helper API.
3. **Cross-platform** — the same recording session can target web (Chrome) or
   mobile (Android/iOS via Appium).
4. **REST API scaffold** — `GET /api/studio/status`, `POST /api/studio/start`,
   `POST /api/studio/stop`, `POST /api/studio/step`.

## Studio REST API

```
GET  /api/studio/status   — Current recording state
POST /api/studio/start    — Begin recording (optional body: { "test_name": "..." })
POST /api/studio/stop     — Stop recording and receive generated Rust code
POST /api/studio/step     — Append a step: { kind, target, value, ts_ms }
```

## Generated code example

A recording of a login flow produces:

```rust
#[tokio::test]
async fn login_flow() -> uto_core::error::UtoResult<()> {
    let s = uto_test::startNewSession("chrome").await?;
    s.goto("https://example.com/login").await?;
    s.fill_intent("Username", "alice").await?;
    s.fill_intent("Password", "secret").await?;
    s.click_intent("Sign in").await?;
    s.assert_visible("Dashboard").await?;
    s.close().await
}
```

No CSS selectors. No XPath. No fragile DOM lookups.
