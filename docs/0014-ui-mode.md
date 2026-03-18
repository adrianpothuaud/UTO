# ADR 0014: UTO UI Mode — Interactive Test Debugging and Visualization

Date: 2026-03-18

## Status

Proposed

## Context

UTO Phase 4 delivers a first-class CLI lifecycle (`init`, `run`, `report`), structured JSON/HTML reporting, and a reporting-first observability model. With this foundation in place, the next logical step for developer experience is an **interactive UI mode** — a local browser-based interface that allows test authors to run, watch, filter, and debug their UTO test suites with full visual context.

Cypress and Playwright both ship a UI mode that has become a key productivity differentiator for their communities:

- **Cypress interactive runner** — a desktop Electron app with real-time test execution, command log, DOM snapshots, and time-travel debugging.
- **Playwright UI mode** (`npx playwright test --ui`) — a browser-based UI with test tree, watch mode, trace viewer, network panel, and screenshot/video inspection per step.

UTO must offer a comparable experience that:
- extends the reporting-first philosophy already established in Phase 4;
- is **platform-agnostic** (works for both web and mobile test runs, consuming the same `uto-report/v1` and `uto-suite/v1` event streams);
- is **CLI-native** — launched with `uto ui` from any Phase 4+ project directory;
- is **offline-first** with no external CDN or network dependencies.

## Decision

UTO will implement a **UI Mode** as its Phase 5 developer-experience milestone. The feature will be delivered as a new `uto-ui` crate that:

1. embeds a lightweight local HTTP + WebSocket server (Axum or Hyper) to serve the UI and stream live test events;
2. provides a browser-based interactive interface (single-page app with inline HTML/CSS/JS, no bundler required);
3. reuses the existing `uto-reporter` schema and event model as its data layer — no new protocol invented;
4. integrates with the `uto-cli` entrypoint via a new `uto ui` command.

### Scope and Feature Set

#### Phase 5 MVP (first delivery)

| Feature | Description |
|---|---|
| **Test tree panel** | Hierarchical list of all test files and test cases discovered in the project. |
| **Run / Stop controls** | Trigger a full suite run or a filtered subset from the UI; stop a running suite. |
| **Live event stream** | Real-time display of step-level events as they are emitted by `uto-reporter`. |
| **Pass / Fail summary** | Visual indicator per test (pass ✅, fail ❌, skip ⏭️) updated live as tests complete. |
| **Watch mode** | Automatically re-run affected tests when source files change (filesystem watcher). |
| **Report replay** | Load and inspect any previously saved `uto-suite/v1` JSON artifact without re-running. |
| **Screenshot timeline** | Display captured screenshots attached to report events in chronological order. |
| **Step detail panel** | Expand individual step events to see full context: intent label, resolved candidates, driver payload excerpt. |
| **Platform badge** | Show whether a test ran on `web` or `mobile` target, matching the `uto-report/v1` metadata. |

#### Post-MVP (Phase 6+)

| Feature | Description |
|---|---|
| **Time-travel debugging** | Step backward/forward through the report event stream, restoring UI state at each step. |
| **Network request inspector** | Surfacing WebDriver-level request/response pairs from the report for failure triage. |
| **Console log viewer** | Show browser or device console output attached to test runs. |
| **Diff comparison** | Compare two `uto-suite/v1` artifacts side-by-side (baseline vs. current run). |
| **Plugin API** | Allow projects to register custom event renderers and sidebar panels. |

### Architecture

#### Crate responsibility: `uto-ui`

```
uto-ui/
  src/
    server.rs         — Axum/Hyper server, static-asset handler, WebSocket endpoint
    watcher.rs        — filesystem watcher triggering re-runs on file change
    runner.rs         — subprocess bridge: spawns `uto run` and streams its stdout/events
    assets/
      index.html      — full SPA (inline CSS + JS, no external deps)
      ...             — additional embedded static assets if needed
  Cargo.toml
```

The crate keeps the same philosophy as `uto-reporter`: **no external runtime CDN dependencies**. All HTML/CSS/JS assets are embedded at compile time via `include_str!` / `include_bytes!`.

#### CLI integration: `uto ui`

The `uto-cli` crate gains a `ui` sub-command:

```
uto ui [OPTIONS]
    --project <path>    Path to the UTO project directory (default: .)
    --port <port>       Local port for the UI server (default: 4000)
    --open              Automatically open the browser after startup
    --watch             Enable watch mode (re-run on file change)
    --report <path>     Load an existing report artifact instead of running
```

Startup sequence:

1. Validate `uto.json` in the project directory (reuse `uto-cli` validation logic).
2. Start the embedded HTTP + WebSocket server on `localhost:<port>`.
3. Serve the embedded SPA at `http://localhost:<port>`.
4. If `--report` is provided, load the artifact and stream events to the UI.
5. Otherwise wait for a run trigger from the UI (or run immediately if `--watch` is set).
6. On run trigger: spawn `uto run` in a subprocess, capture event stream, relay over WebSocket.

#### Event protocol

The UI consumes the existing `uto-report/v1` and `uto-suite/v1` event formats emitted by `uto-reporter`. The WebSocket channel transports individual `ReportEvent` payloads as newline-delimited JSON (NDJSON), identical to the JSON file output. No new event types are introduced at MVP; the schema is extended only for post-MVP features.

#### Platform-agnostic design

Because `uto-reporter` already attaches `platform` metadata (`web` / `mobile`) to each test case result and the top-level suite report, the UI can render platform-appropriate context without any new logic. Mobile runs display Appium-sourced screenshots and device metadata; web runs display browser screenshots and DOM context. Both share the same step-level event panel.

### Compatibility

- **Minimum project version:** Any Phase 4+ UTO project that emits `uto-suite/v1` JSON artifacts is compatible with the UI mode.
- **Backward compatibility:** The UI mode is read-only with respect to the project structure. It does not modify `uto.json`, source files, or existing report artifacts.
- **Forward compatibility:** The event stream protocol is versioned via the `schema_version` field already present in `uto-report/v1`.

### Technology choices

| Component | Choice | Rationale |
|---|---|---|
| HTTP server | `axum` (already a likely transitive dep via `tokio`) | Lightweight, async-native, Rust-idiomatic |
| WebSocket | `axum`'s built-in WebSocket upgrade | Avoids additional dependency |
| Filesystem watcher | `notify` crate | Cross-platform, well-maintained |
| Frontend | Vanilla HTML/CSS/JS (no framework) | Matches existing `uto-reporter` HTML approach; no build toolchain required |

All new crate dependencies must pass advisory-database security review before addition.

## Phase 5 Delivery Plan

### Iteration 5.1 — `uto-ui` crate scaffold and `uto ui` CLI command

1. Create `uto-ui` crate with `axum`-based server and embedded SPA skeleton.
2. Add `uto ui` sub-command to `uto-cli` with project validation and server startup.
3. Serve a static placeholder page at `http://localhost:4000`.
4. Add unit tests for server startup, port binding, and graceful shutdown.

### Iteration 5.2 — Report replay and test tree

1. Implement report artifact loading (`--report` flag).
2. Build the SPA test tree panel from `uto-suite/v1` test case list.
3. Implement pass/fail/skip summary rendering from existing JSON artifacts.
4. Add event stream replay via WebSocket on artifact load.

### Iteration 5.3 — Live run integration

1. Implement subprocess bridge: spawn `uto run`, capture NDJSON event stream.
2. Stream events over WebSocket to the SPA in real time.
3. Add Run / Stop controls in the SPA.
4. Add platform badge and screenshot timeline panel.

### Iteration 5.4 — Watch mode and Phase 5 validation

1. Implement filesystem watcher (`notify`) for test-source file changes.
2. Auto-trigger re-run on detected change (with debounce).
3. Add `--watch` and `--open` CLI flags.
4. Validate end-to-end on macOS, Linux, and Windows.
5. Add Phase 5 reference project under `examples/phases/phase5-ui-mode/`.

## Done Criteria for Phase 5 MVP

- [ ] `uto ui` starts without error on a Phase 4+ project directory.
- [ ] Navigating to `http://localhost:4000` shows the UTO UI with the test tree.
- [ ] Loading a saved `uto-suite/v1` artifact via `--report` replays events in the UI.
- [ ] Triggering a run from the UI spawns `uto run` and streams events live.
- [ ] Watch mode re-runs on file change.
- [ ] Screenshots attached to report events are displayed in the timeline panel.
- [ ] Platform badge correctly shows `web` or `mobile` per test.
- [ ] All tests pass on macOS, Linux, and Windows CI.
- [ ] A committed Phase 5 reference project exists under `examples/phases/phase5-ui-mode/`.

## Consequences

### Positive

- UTO developer experience becomes comparable to Playwright/Cypress for test authors.
- The observability-first philosophy established in Phase 4 gains its most visible surface.
- Platform-agnostic design means mobile test authors benefit equally — a differentiator vs. Cypress (web-only).
- The CLI-native approach (no Electron, no separate desktop app) keeps UTO lean.
- Offline-first, embeddable assets maintain the zero-external-dependency policy.

### Negative

- New `axum` + `notify` crate dependencies increase compile-time surface.
- The embedded SPA requires maintenance when the `uto-reporter` schema evolves.
- WebSocket-based live event streaming adds process lifecycle complexity (clean shutdown of all child processes via the existing clean-hook model is essential).
- A new `uto-ui` crate increases workspace size and CI build time.

### Architectural Implications

Phase 5 **does not change core layer boundaries:**

- `env`, `driver`, `session`, `vision` remain unchanged.
- `uto-reporter` schema is consumed, not extended at MVP.
- `uto-ui` is purely an orchestration and presentation layer.
- `uto-cli` gains one new sub-command; existing commands remain unchanged.

## Validation Approach

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` on macOS, Linux, and Windows (existing CI matrix)
- Integration test: `uto ui --report <artifact>` serves a page with expected DOM structure (headless browser or HTML string assertion)
- Smoke test: `uto ui --project examples/phases/phase5-ui-mode --port 4001` starts without error and responds to `GET /`

---

## References

- ADR 0009: Framework Product Direction — CLI and Reporting-First Experience
- ADR 0010: Phase 3 Completion Assessment and Phase 4 Planning
- ADR 0011: Shared `uto-test` Crate and Clean SoC Guidelines
- ADR 0013: Getting Started and Troubleshooting
- `uto-reporter/src/schema.rs` — existing `UtoReportV1` and `UtoSuiteReportV1` schemas
- Playwright UI Mode: https://playwright.dev/docs/test-ui-mode
- Cypress interactive runner: https://docs.cypress.io/guides/core-concepts/cypress-app
