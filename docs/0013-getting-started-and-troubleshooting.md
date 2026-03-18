# UTO Getting Started and Troubleshooting

Date: 2026-03-18

## Purpose

This guide is the Phase 4.4 onboarding baseline for new contributors and test authors.

It shows how to:

1. initialize a project
2. run the first test
3. inspect JSON + HTML reports
4. resolve common host-environment issues without deep architecture knowledge

## Prerequisites

- Rust toolchain installed (`rustup`, `cargo`)
- Google Chrome installed (for web path)
- Optional mobile path: Android SDK + `adb` + Appium
- A clone of the UTO repository

UTO is discover-or-deploy by design. If tools are missing, UTO attempts best-effort setup where possible and otherwise returns actionable errors.

## 30-Minute Quick Start

From repository root:

```sh
# 1) Build CLI
cargo build -p uto-cli

# 2) Scaffold a project
cargo run -p uto-cli -- init ./my-uto-tests --template web --uto-root "$PWD"

# 3) Run web tests and emit report JSON
cargo run -p uto-cli -- run \
  --project ./my-uto-tests \
  --target web \
  --report-json ./my-uto-tests/.uto/reports/last-run.json

# 4) Print report summary and generate HTML view
cargo run -p uto-cli -- report \
  --project ./my-uto-tests \
  --html
```

Expected artifacts:

- `./my-uto-tests/.uto/reports/last-run.json`
- `./my-uto-tests/.uto/reports/last-run.html`

Open the HTML file directly in a browser. It is static/offline-readable and derived from `uto-report/v1` JSON.

## Project Structure You Get From `uto init`

- `uto.json`: project configuration
- `src/bin/uto_project_runner.rs`: local runner entrypoint used by `uto run`
- `tests/web_example.rs` + `tests/mobile_example.rs`: starter tests
- `.uto/reports/`: report output directory

## First Test Authoring Pattern

Generated examples and reference projects use `uto-test` helpers for concise setup while preserving setup/session logs.

```rust
#[tokio::test]
async fn web_smoke() -> uto_core::error::UtoResult<()> {
    let session = uto_test::startNewSession("chrome").await?;
    session.goto("https://example.com").await?;
    let title = session.title().await?;
    assert!(!title.is_empty());
    session.close().await
}
```

For mobile:

```rust
#[tokio::test]
async fn mobile_smoke() {
    let session = match uto_test::startNewSessionWithArg("android", 16).await {
        Ok(s) => s,
        Err(err) => {
            println!("Skipping mobile smoke: {err}");
            return;
        }
    };

    let _ = session
        .launch_android_activity("com.android.settings", ".Settings")
        .await;

    let _ = session.close().await;
}
```

## Understanding Reports

UTO keeps JSON as the source of truth and can render an HTML summary from it.

### JSON (`uto-report/v1`)

Includes:

- run metadata (`run_id`, mode, status, timeline)
- ordered step events
- error block when failures occur

### HTML (`uto report --html`)

Includes:

- run header (id/mode/status/start/end/duration)
- timeline table (stage/status/details)
- error section when present
- schema metadata footer

## Troubleshooting

### `uto init` fails with workspace-root errors

Symptom:

- generated project cannot resolve local crates (`uto-core`, `uto-test`, `uto-runner`)

Fix:

- pass `--uto-root <path-to-UTO-repo>` explicitly
- confirm path points to repository root containing workspace `Cargo.toml`

### `uto run --target web` fails to create browser session

Symptom:

- session creation fails or ChromeDriver not reachable

Fix:

- ensure Chrome is installed
- run `cargo run -p uto-poc --bin phase1_verify_or_deploy_drivers` to validate discovery/provisioning
- rerun with `RUST_LOG=info` for setup/driver logs

### Mobile tests skip or fail on environment preparation

Symptom:

- Appium unavailable
- no Android device/emulator found
- UiAutomator2 missing

Fix:

- verify `adb devices` shows an online device or emulator
- ensure Appium is installed and reachable from PATH
- if needed, install Appium UiAutomator2 driver
- keep tests graceful: skip mobile flows when session bootstrap fails

### `uto report` rejects report file

Symptom:

- schema validation error

Fix:

- ensure report `schema_version` is `uto-report/v1`
- regenerate report via `uto run`
- avoid manual edits to the JSON artifact

### HTML report not generated

Symptom:

- JSON exists but HTML file is missing

Fix:

- run `uto report --project <path> --html`
- or set explicit output path: `--html-output <path>/report.html`

## Validation Commands

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
./examples/validate-cli.sh
```

Optional mobile-inclusive CLI smoke check:

```sh
WITH_MOBILE=1 ./examples/validate-cli.sh
```

## Related Documents

- `docs/0010-phase-3-completion-and-phase-4-planning.md`
- `docs/0011-uto-test-crate-and-clean-soc-guidelines.md`
- `docs/0012-uto-test-api-usage-guide.md`
- `README.md`