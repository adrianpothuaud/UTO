+++
title = "Getting Started"
description = "Bootstrap a UTO project, run tests, and generate JSON/HTML reports in under 30 minutes."
template = "page"
slug = "getting-started"
+++

# Getting Started

This page mirrors the repository onboarding flow for UTO Phase 4.4.

## Prerequisites

- Rust toolchain (`cargo`)
- Chrome (web target)
- Optional mobile: Android SDK + `adb` + Appium

## Quick Start

```sh
# Build CLI
cargo build -p uto-cli

# Initialize project
cargo run -p uto-cli -- init ./my-uto-tests --template web --uto-root "$PWD"

# Run web target + JSON report
cargo run -p uto-cli -- run \
  --project ./my-uto-tests \
  --target web \
  --report-json ./my-uto-tests/.uto/reports/last-run.json

# Summarize and render HTML report
cargo run -p uto-cli -- report --project ./my-uto-tests --html
```

Artifacts:

- `.uto/reports/last-run.json`
- `.uto/reports/last-run.html`

## Authoring Pattern

```rust
#[tokio::test]
async fn web_smoke() -> uto_core::error::UtoResult<()> {
    let session = uto_test::startNewSession("chrome").await?;
    session.goto("https://example.com").await?;
    session.close().await
}
```

## Related Docs

- `docs/0013-getting-started-and-troubleshooting.md`
- `docs/0012-uto-test-api-usage-guide.md`