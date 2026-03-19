# uto-core

**Core driver and session infrastructure for the UTO automation framework.**

`uto-core` provides the foundational primitives for WebDriver communication, driver lifecycle management, and cross-platform environment discovery. It powers `uto-test` and the UTO CLI.

## Features

- **WebDriver protocol** — W3C-compliant WebDriver client for Chrome and Appium
- **Driver lifecycle** — Process management with clean shutdown and process group cleanup
- **Environment discovery** — Cross-platform detection and provisioning of Chrome, ChromeDriver, Android SDK, and Appium
- **Session abstractions** — `WebSession` and `MobileSession` with unified `UtoSession` trait
- **Error handling** — Structured error types for driver, session, and environment failures

## Components

### Driver Management (`driver` module)

- `DriverProcess` — Manages driver process lifecycle (start, readiness check, stop)
- Process group management for clean child process termination (Unix: PGID, Windows: Job Objects)
- Automatic port allocation and readiness polling

### Session Layer (`session` module)

- `WebSession` — Chrome/Chromium session with W3C WebDriver
- `MobileSession` — Android/Appium session with mobile-specific capabilities
- `UtoSession` trait — Unified interface across session types
- Element finding, clicking, typing, navigation, assertions

### Environment Discovery (`env` module)

- `platform` — Chrome/Chromium version detection (macOS, Linux, Windows)
- `provisioning` — ChromeDriver download and caching
- `mobile_setup` — Android SDK discovery, AVD management, Appium installation

## Usage

`uto-core` is typically used through `uto-test`, but can be used directly for advanced scenarios:

```rust
use uto_core::session::WebSession;
use uto_core::driver::DriverProcess;

// Start ChromeDriver
let driver = DriverProcess::start_chromedriver("/path/to/chromedriver", 9515)
    .await
    .unwrap();

// Create session
let session = WebSession::new("http://localhost:9515").await.unwrap();

// Use session
session.goto("https://example.com").await.unwrap();
let title = session.title().await.unwrap();

// Cleanup
driver.stop().await;
```

## Architecture

`uto-core` follows a layered architecture:

1. **Driver layer** — Process management and lifecycle
2. **Transport layer** — HTTP client for WebDriver protocol
3. **Session layer** — High-level API for test authoring
4. **Environment layer** — Platform discovery and provisioning

See [ADR 0001](../docs/0001-zero-config-infrastructure.md) and [ADR 0002](../docs/0002-driver-communication-layer.md) for architectural details.

## Related Crates

- **uto-test** — High-level test authoring library
- **uto-cli** — CLI for project generation and test execution
- **uto-reporter** — JSON/HTML report generation

## License

MIT or Apache-2.0 (dual-licensed)
