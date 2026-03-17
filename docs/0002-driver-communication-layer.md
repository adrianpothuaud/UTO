# 2. Driver Communication Layer

Date: 2026-03-17

## Status

Accepted

## Context

Phase 1 (Zero-Config Infrastructure) established a fully working "Discover or Deploy" engine for both web and mobile drivers:

- **Web:** Chrome version detection, ChromeDriver download/extraction into a local cache, process lifecycle management using OS process groups.
- **Mobile:** Android SDK discovery, Appium discovery, Appium process lifecycle management.

The next challenge is **communication** — once a driver is running, UTO needs to send commands to it (navigate, find elements, click, type) and receive responses. The key design question is: how should UTO interact with ChromeDriver and Appium?

Both drivers implement the **W3C WebDriver protocol** — a JSON-over-HTTP standard for remote browser/device control. This creates an opportunity to use a single client library for both platforms.

## Decision

1. **Use `thirtyfour` as the unified W3C WebDriver client** for both ChromeDriver (web) and Appium (mobile).  
   - `thirtyfour` is a mature, async Rust WebDriver client.
   - Appium implements the W3C WebDriver spec, so the same HTTP transport works for both.
   - The key difference between web and mobile sessions lies entirely in the *capabilities* object sent at session creation time.

2. **Introduce a `session` module** with three components:
   - `session/web.rs` — `WebSession`: connects to ChromeDriver using `ChromeCapabilities`.
   - `session/mobile.rs` — `MobileSession`: connects to Appium using `MobileCapabilities` (a builder for the Appium W3C capability set including `platformName`, `appium:deviceName`, `appium:automationName`, etc.).
   - `session/mod.rs` — the `UtoSession` trait: a platform-agnostic async trait exposing `goto`, `find_element`, `click`, `type_text`, `get_text`, `screenshot`, and `close`.

3. **Mobile-specific gestures** (`tap`, `swipe`) are implemented on `MobileSession` via the W3C Actions API (using `thirtyfour`'s `ActionChain`). The `UtoSession` trait covers the common surface; native gesture support extends it.

4. **`async-trait`** is used to allow `UtoSession` to be used as a trait object (`Box<dyn UtoSession>`), enabling test logic to be written against the unified API without caring whether it runs on a browser or a device.

5. **Appium base-path compatibility is handled in the session layer.**
   - `MobileSession::new` first attempts session creation with the URL provided by the managed driver process.
   - If Appium responds with an `unknown command` + HTTP 404 signature (common when `/wd/hub` vs root base paths differ between Appium versions/configuration), UTO retries once using the alternate base path.
   - This keeps `driver` focused on lifecycle concerns while preserving robust cross-version communication behavior in `session`.

## Module Structure

```
uto-core/src/
├── error.rs          — UtoError / UtoResult
├── env/
│   ├── platform.rs   — Chrome version detection, Android SDK discovery, Appium discovery
│   ├── mobile_setup.rs — Android/Appium auto-fix (adb start, emulator boot, Appium + UiAutomator2 install)
│   └── provisioning.rs — ChromeDriver download / extraction from Chrome for Testing API
├── driver/
│   └── mod.rs        — DriverProcess: start/stop ChromeDriver and Appium (process groups)
└── session/          ← NEW in Phase 2
    ├── mod.rs        — UtoSession trait, UtoElement, ElementHandle
    ├── web.rs        — WebSession (ChromeDriver)
    └── mobile.rs     — MobileSession (Appium), MobileCapabilities, MobilePlatform
```

## Consequences

**Positive:**
- **Unified API:** Test code can interact with both Chrome and Android/iOS via a single `UtoSession` trait, enabling true cross-platform tests.
- **Clean separation:** The `session` layer is independent of the `driver` layer; the caller starts a driver, then creates a session pointing at it.
- **Extensible:** New platforms (Firefox, Safari, iOS) only need a new capabilities builder — the `UtoSession` implementation is reused.

**Negative:**
- **Appium nuance:** While both use W3C WebDriver, Appium has additional endpoints and capabilities (e.g. `mobile:` commands). These are accessible via `MobileSession`-specific methods.
- **Environment-dependent mobile readiness:** Mobile integration tests can only create a full session when Appium plus a compatible automation backend/device are available; tests therefore skip gracefully when those host dependencies are missing.
- **Toolchain assumptions for auto-fix:** Automatic Appium installation depends on a working Node.js/npm toolchain, and emulator auto-start requires at least one existing AVD image on the host.
- **No BiDi yet:** Real-time push events (WebDriver BiDi / CDP) are a future improvement.
