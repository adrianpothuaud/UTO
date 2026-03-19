# uto-test

**Test authoring library for the UTO automation framework.**

`uto-test` provides a high-level API for writing web and mobile automation tests in Rust with minimal boilerplate. It handles driver provisioning, session lifecycle, and report generation automatically.

## Features

- **Unified session API** — Single `ManagedSession` type works across web (Chrome) and mobile (Android) targets
- **Zero-config provisioning** — Automatically discovers and provisions required drivers (ChromeDriver, Appium)
- **Managed lifecycle** — Automatic cleanup of sessions and driver processes
- **Library-first design** — Use as a Rust library without CLI scaffolding
- **Framework integration** — Works seamlessly with `uto init` generated projects

## Quick Start

### As a Library

Add `uto-test` to your `Cargo.toml`:

```toml
[dependencies]
uto-test = "0.1"
tokio = { version = "1", features = ["full"] }
```

Write a test:

```rust
use uto_test::*;

#[uto_test(target = "web")]
#[tokio::test]
async fn test_example_site() {
    let session = startNewSession("web").await.unwrap();
    session.goto("https://example.com").await.unwrap();
    let title = session.title().await.unwrap();
    assert!(title.contains("Example"));
}
```

Run with `cargo test`.

### With UTO CLI

Generate a project:

```bash
uto init my-project
cd my-project
```

Run tests:

```bash
cargo test
# or
uto run
```

## API Overview

### Session Creation

- `startNewSession(target)` — Creates a sessionfor "web" or "mobile"
- `startNewSessionWithArg(target, hint)` — Creates a session with optional hint (e.g., AVD boot timeout)

### Session Methods

All methods work across web and mobile targets:

- `goto(url)` — Navigate to URL or deep link
- `title()` — Get page/activity title
- `find_element(selector)` — Find element by CSS selector (web) or accessibility ID (mobile)
- `select(label)` — Find element by human-readable label
- `click(label)` — Click element by label
- `type_text(label, text)` — Type text into element
- `assert_visible(label)` — Assert element is visible
- `close()` — Close session and cleanup driver

## Requirements

### Web (Chrome)
- Chrome/Chromium browser installed
- ChromeDriver is auto-provisioned

### Mobile (Android)
- Android SDK with `adb` in PATH
- Appium is auto-provisioned via npm
- At least one AVD or physical device available

## Related Crates

- **uto-core** — Core driver and session implementation
- **uto-cli** — CLI for project generation and test execution
- **uto-reporter** — JSON/HTML report generation
- **uto-logger** — Structured logging and progress indicators

## License

MIT or Apache-2.0 (dual-licensed)
