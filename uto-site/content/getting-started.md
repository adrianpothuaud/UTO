+++
title = "Getting Started"
description = "Install UTO with a one-line command, scaffold a project, run tests, and generate JSON/HTML reports in under 30 minutes."
template = "page"
slug = "getting-started"
+++

# Getting Started

## Install UTO

The fastest way to install the `uto` CLI (macOS / Linux):

```sh
curl -sSf https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.ps1 | iex
```

The installer checks whether Rust is present (and installs it via [rustup](https://rustup.rs) if not), builds the `uto` binary from source, and prints getting-started instructions when it completes.

## Prerequisites

- Chrome (for the `web` target)
- Optional mobile: Android SDK + `adb` + Appium

## Quick Start

```sh
# 1. Scaffold a new test project
uto init ./my-tests --template web

# 2. Run your tests
uto run --project ./my-tests --target web

# 3. Open a structured HTML report
uto report --project ./my-tests --html

# 4. Launch the interactive UI
uto ui --project ./my-tests
```

Artifacts produced:

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

## Need Help?

See the [Troubleshooting](../troubleshooting/) page for common setup and run issues, or [request early access](../early-access/) to get direct support from the team.