# Gemini Code Understanding (GEMINI.md)

## Project Overview

This project, "UTO" (Unified Testing Object), is a next-generation, cross-platform automation engine written in Rust. Its goal is to provide a "Vision-First, Human-Centric" alternative to traditional automation frameworks like Selenium and Appium.

The core philosophy is to interact with UIs (Web and Mobile) as a human would, by perceiving visual elements rather than relying solely on brittle DOM/Accessibility tree selectors.

### Key Technologies

*   **Core:** Rust
*   **Async Runtime:** `tokio`
*   **WebDriver:** `thirtyfour`
*   **Vision Engine (Future):** ONNX Runtime
*   **Communication (Future):** gRPC (`tonic`) and WebSockets (`tokio-tungstenite`)

### Architecture

The project is designed around five pillars:
1.  **Zero-Config Infrastructure (`uto-env`):** Automatically discovers browsers/SDKs and provisions the necessary drivers in isolated environments.
2.  **The Recognition Loop (`uto-vision`):** Uses ML to identify UI components visually, anchored by traditional accessibility data for resilience.
3.  **Human-Centric Interaction (`uto-api`):** Provides a high-level API focused on user intent (e.g., `select("Add to Cart")`).
4.  **Simplicity by Default (`uto-simplicity`):** Hides routine automation mechanics (for example iframe context switching on web, and scroll/fling handling on mobile) behind predictable defaults.
5.  **The Hybrid Orchestrator (`uto-link`):** A high-performance Rust backbone for orchestrating complex, multi-device tests.

Product direction now also includes a framework-facing UX objective:

- first-class CLI lifecycle (`init`, `run`, `report`)
- reporting-first execution visibility from setup to assertion outcomes
- structured report output for CI and diagnostics
- Phase 5 interactive UI mode (`uto ui`) — a local browser-based interface for running, watching, and debugging test suites (see ADR 0014)

The current implementation covers the **Zero-Config Infrastructure** (Phase 1) and the **Driver Communication Layer** (Phase 2):

- `src/env/` — browser/SDK discovery and ChromeDriver provisioning.
- `src/driver/` — process lifecycle management for ChromeDriver and Appium.
- `src/session/` — W3C WebDriver communication for web (`WebSession`) and mobile (`MobileSession`), unified behind the `UtoSession` trait.

Separation-of-concerns rule in the current architecture:

- `env` remains responsible for discovery/provisioning decisions only.
- `driver` remains responsible for process lifecycle/readiness/cleanup only.
- `session` remains responsible for WebDriver protocol orchestration only.
- mobile intent/accessibility matching strategy is implemented in dedicated session helpers (`src/session/mobile_accessibility.rs`).
- Appium session bootstrap/base-path retry logic is isolated in `src/session/mobile_connection.rs`.
- `vision` remains responsible for detection/ranking/resolution logic and latency guardrails.

POC phase isolation rule:

- `phase1_verify_or_deploy_drivers` is infrastructure-only (no session intent flow).
- `phase2_interact_with_session` is communication-layer-only (no phase3 intent pipeline).
- `phase3_intent_poc` is intent/resolution/reporting demonstration.
- New experiments must not collapse phase responsibilities into a single POC binary.

Framework-facing workflow components now include:

- `uto-core/` — core engine (env discovery/provisioning, driver lifecycle, W3C session protocol, vision/intent)
- `uto-test/` — end-user test helper crate (simple session start/close API + `Suite` multi-test runner)
- `uto-reporter/` — structured `uto-report/v1` schema, HTML/JSON emission
- `uto-logger/` — modern structured logging + loader/spinner utilities for long-running tasks
- `uto-runner/` — CLI option parsing for generated-project runners
- `uto-ui/` — interactive UI mode server: embedded HTTP + WebSocket server + browser SPA (`uto ui` command)
- `uto-cli/` — framework CLI entrypoint for project lifecycle commands (`init`, `run`, `report`, `ui`)
- `examples/` — generated-project validation flow for CLI smoke testing

For mobile readiness, `src/env/` now also performs best-effort auto-fixes:

- starts `adb` and verifies online Android devices
- attempts to boot an available Android emulator AVD when no device is connected
- installs Appium via npm when missing
- installs the Appium `uiautomator2` driver when missing

## Current Status

The Phase 2 POC for the `uto-env` + `uto-session` pillars is complete. The `main` branch contains a working implementation that can:

1.  **Discover:** Automatically find the installed version of Google Chrome and the Android SDK on the host system.
2.  **Provision:** Download the matching version of ChromeDriver from the official Google Chrome for Testing repository.
3.  **Execute:** Launch both ChromeDriver and Appium processes in OS-level process groups (clean hook).
4.  **Communicate (Web):** Create a `WebSession` via ChromeDriver and navigate/interact with Chrome using the W3C WebDriver protocol.
5.  **Communicate (Mobile):** Create a `MobileSession` via Appium and interact with an Android/iOS device using the same W3C WebDriver protocol.

Both session types implement the `UtoSession` trait, which provides a platform-agnostic API for cross-platform test logic.

Phase 3 MVP is now **complete** in `src/vision/` and `src/session/` with:

- **Vision Foundation (3.1):** deterministic preprocessing and post-processing (including NMS) with ONNX inference abstraction
- **Weighted Consensus Resolver (3.2):** fusion of vision confidence + accessibility metadata with explicit mismatch diagnostics
- **Latency Guardrails (3.3):** median/P95 tracking with phase-specific SLA enforcement (≤50ms median, ≤100ms p95 for vision-only; ≤60ms/≤120ms with accessibility)
- **Intent-Style Session API:** `select(label)`, `click_intent(label)`, `fill_intent(label, value)` with web-first resolver and mobile fallback
- **Cross-platform:** web and mobile via `UtoSession` trait with graceful skip behavior when host tools unavailable

## Phase 3 Deliverables

All five **Phase 3 MVP completion criteria** met:

1. ✅ **Deterministic recognition:** preprocessing + NMS + consensus ranking with unit tests
2. ✅ **Accessibility-boosted resolution:** weighted scoring demonstrably improves recall on ambiguous targets
3. ✅ **Intent actions operational:** `select/click_intent/fill_intent` validated on web and mobile flows
4. ✅ **Cross-platform parity:** mobile path uses same resolver+fallback, skips gracefully when Appium unavailable
5. ✅ **CI stability:** 94 unit tests green, latency SLA tests deterministic, no host tool dependencies in core tests

## Phase 4: Framework Maturity and Reporting-First Experience

**Status:** Complete (4.1 CLI Scaffolding ✅, 4.2 HTML Reporting ✅, 4.3 Mobile Hardening ✅, 4.4 Onboarding/Examples ✅)

Phase 4 refocuses UTO from core engine capability toward end-user framework experience. Main objectives:

1. **CLI Lifecycle Foundation** — stabilize `uto init`, `uto run`, `uto report` interface ✅
2. **Structured Reporting** — machine-readable execution traces with latency instrumentation and native HTML rendering ✅
3. **Mobile Parity Hardening** — production-ready intent resolution on Android via Appium ✅
4. **Framework Documentation** — "Getting Started" guide, troubleshooting, end-to-end examples ✅

**Key Design Principles:**
- CLI orchestrates `uto-core` APIs (no core layer changes)
- Report schema is versioned for forward/backward compatibility
- Mobile path uses same resolver + fallback as web (no platform divergence)
- Framework documentation becomes product-level responsibility
- Phase 1/2/3 layer boundaries remain unchanged

**Iteration 4.1 Completion (CLI Scaffolding):**
1. Implemented strict CLI argument validation and unknown-option handling in `uto-cli`
2. Added `uto.json` schema validation and project-structure preflight checks before execution
3. Added `uto-report/v1` artifact validation in `uto report`
4. Added CLI unit tests plus integration-style binary workflow tests in `uto-cli/tests/cli_workflow.rs`
5. Added shared `uto-test` helper API so end-user tests can start sessions with one call (`startNewSession("chrome")`, `startNewSessionWithArg("android", 16)`) while retaining setup/session logs
6. Added `uto-runner` crate so generated/reference project runners avoid duplicated orchestration/report code
7. Split CLI responsibilities into focused modules (`commands`, `config`, `parsing`, `templates`, `io`)
8. Added generated-project compatibility tests validating `uto init` output compiles with `cargo check --tests`

**Iteration 4.2 Completion (Report Schema and HTML Rendering):**
1. Defined typed `uto-report/v1` schema in `uto-reporter/src/schema.rs` with `UtoReportV1`, `ReportEvent`, `ReportTimeline` structs
2. Extended reporting with typed `uto-suite/v1` schema for multi-test runs (`UtoSuiteReportV1`, `TestCaseResult`, `SuiteSummary`)
2. Implemented `uto-reporter` crate with standalone report accumulation (`Report` impl with payload serialization)
3. Implemented `SuiteReport` accumulator for named per-test timelines and event streams
3. Implemented deterministic offline HTML rendering in `uto-reporter/src/html.rs` with security hardening (XSS-safe entity escaping)
4. Upgraded native HTML UX with theme toggle persistence, live search/filter, suite test filtering, and expand/collapse controls
4. Integrated `uto-reporter` into `uto-cli` for JSON/HTML report generation via `uto report` command
5. Added `--html` and `--html-output` flags to `uto report` command for artifact generation
6. Added integration tests validating HTML output structure and XSS protection
7. All tests passing including schema round-trip and HTML rendering tests

**Iteration 4.3 Completion (Mobile Hardening and Logging):**
1. Implemented `uto-logger` crate with tracing-based structured logging backend + indicatif spinner/loader utilities
2. Added `LoaderManager` for managing concurrent progress spinners across setup phases (discover, provision, startup)
3. Integrated `uto-logger::init("component-name")` in all POC binaries and reference projects for unified log format
4. Removed scattered `env_logger` initialization in favor of global, idempotent logger setup
5. Extended `phase3_intent_poc.rs` with `--html` and `--html-file` CLI flags for HTML report emission
6. Updated all reference projects (`phase3-intent`, `phase4-framework`) to use new logging infrastructure
7. Mobile flow now uses same resolver + fallback pattern; graceful skip when Appium unavailable

**Iteration 4.4 Completion (Onboarding, Examples, and Site Navigation):**
1. Created `examples/phases/phase4-framework/` reference project demonstrating Phase 4 capabilities:
   - Loader spinners for long-running setup phases (discover, provision, startup)
   - Modern structured logging with process awareness
   - JSON + HTML report generation workflows
   - Web and mobile execution paths with graceful fallbacks
   - Test examples using `uto-test` helpers (`startNewSession`, `wait_for_intent`)
   - Suite-based project runner with multiple named tests and isolated sessions
2. Updated CLI smoke-test validation script (`examples/validate-cli.sh`) to execute phase4-framework project and verify HTML artifact generation
3. Updated static site navigation in `uto-site/templates/base.html` and `uto-site/templates/index.html`:
   - Added "Getting Started" and "Troubleshooting" links to main navigation
   - Reordered hero CTA buttons to prioritize getting-started onboarding
   - Updated status banner to reflect Phase 4.1-4.3 completion status with direct link to getting-started
4. Updated README.md and examples/README.md to list `phase4-framework` alongside `phase3-intent` as committed reference projects
5. Fixed Copilot customization guardrails in `.github/copilot-instructions.md` to prioritize source-code edits over docs-only changes for implementation requests
6. Documented Copilot editing bias issue and mitigation in `docs/0004-copilot-customization.md`

**Architectural Separation of Concerns (Phase 4):**
- `uto-core/` — infrastructure/protocol (env, driver, session, vision, intent)
- `uto-test/` — end-user test helpers (session lifecycle)
- `uto-reporter/` — report schema + JSON/HTML generation (versioned, machine-readable)
- `uto-logger/` — structured logging + progress visualization (process-aware, callable from anywhere)
- `uto-runner/` — CLI option parsing for generated projects (minimal, re-exports from uto-reporter)
- `uto-cli/` — framework orchestration (init, run, report, ui commands)
- Phase examples (`examples/phases/*`) — committed reference projects per phase, durable in-repo

**Phase 4 Validation:**
- Workspace builds cleanly with all 4 new crates registered
- CLI smoke tests pass: generated projects compile, execute, and emit JSON + HTML reports
- All POC binaries operational with unified logger and HTML reporting
- Reference projects (phase3-intent, phase4-framework) execute cleanly
- Site navigation prominently links Getting Started and Troubleshooting from homepage
- 150+ workspace tests green across all platforms (mac/linux/windows CI)

See `docs/0010-phase-3-completion-and-phase-4-planning.md` for full Phase 4 planning details, delivery schedule, and success metrics.

## Phase 5: UI Mode

**Status:** Complete (Iterations 5.1, 5.2, 5.3, and 5.4 delivered)

Phase 5 delivers the `uto ui` interactive browser-based test runner and debugger.

**Iteration 5.1 + 5.2 Completion:**
1. Created `uto-ui` crate with `axum`-based HTTP + WebSocket server and embedded SPA
2. Added `uto ui` sub-command to `uto-cli` with argument parsing, project validation, and server startup
3. Embedded SPA (single-page app) with dark/light theme, test tree panel, event stream, platform badge, and summary bar
4. Report replay: `--report <path>` loads a `uto-suite/v1` or `uto-report/v1` artifact and streams it to the browser via WebSocket
5. `GET /api/report` REST endpoint returns the loaded report JSON for initial page load
6. `GET /api/status` returns project name and server status for the topbar
7. WebSocket `/ws` endpoint with broadcast channel architecture for live-run integration
8. Added 10 unit and route tests in `uto-ui` verifying index HTML, status API, report API, and project name derivation
9. Added 6 new `parse_ui_args` parsing tests in `uto-cli`
10. Created `examples/phases/phase5-ui-mode/` reference project with schema compatibility tests
11. Fixed pre-existing clippy `let_unit_value` lint in `uto-test/src/managed_session.rs`

**Iteration 5.3 + 5.4 Completion:**
1. Created `uto-ui/src/runner.rs` — subprocess bridge: spawns `cargo run --bin uto_project_runner`, relays stdout/stderr as `log` WebSocket events, broadcasts `run_started` / `run_finished` with updated report
2. Created `uto-ui/src/watcher.rs` — filesystem watcher using `notify` crate with 300 ms debounce; watches `tests/` directory
3. Wired `--watch` flag: on file change, auto-triggers a re-run via `handle_trigger_run`
4. Wired Run / Stop controls: WebSocket `trigger_run` / `stop_run` messages handled by server
5. `AppState` updated: shared `Arc<RwLock<report>>` updated after each live run so `/api/report` returns the latest result; `run_active` `AtomicBool` guards against concurrent runs
6. SPA updated: `log` message type handled by `appendLogLine()` — stdout/stderr lines displayed in the event list during a live run
7. `GET /api/status` now returns `"running"` when a run subprocess is active
8. Added 7 new tests: runner unit tests, watcher tests, `api_status_shows_running_when_active`, `handle_trigger_run_is_idempotent_when_active`, `handle_stop_run_is_noop_when_no_run_active`

**`uto ui` command:**
```
uto ui [OPTIONS]
    --project <path>    Path to the UTO project directory (default: .)
    --port <port>       Local port for the UI server (default: 4000)
    --open              Automatically open the browser after startup
    --watch             Enable watch mode (re-run on file change)
    --report <path>     Load an existing report artifact instead of running
```

**Architectural Separation of Concerns (Phase 5):**
- `uto-ui/` — HTTP + WebSocket server, embedded SPA, report relay, subprocess bridge, filesystem watcher (presentation layer only)
- `uto-ui` does not modify `uto-core`, `uto-reporter`, or any existing layer
- `uto-cli` gains `ui` sub-command; all existing commands remain unchanged
- All Phase 4 layer boundaries remain intact

## Phase 6: UTO Studio — Visual Test Authoring

**Status:** Planned — see ADR 0016

Phase 6 delivers **UTO Studio**: a visual, interactive test recording and authoring environment that surpasses Cypress Studio and Playwright Codegen. Key capabilities:

1. **Live session recorder** — captures click, type, navigate, and swipe interactions as semantic intent steps (no selectors generated).
2. **Vision-first element inspector** — overlays bounding boxes, labels, and confidence scores during hover, powered by the Phase 3 recognition loop.
3. **Cross-platform recording** — same recorder works for web (Chrome) and mobile (Android via Appium, iOS via Appium).
4. **Rust code generation** — emits formatted, runnable `uto-test` Rust test functions with no selector-based constructs.
5. **Assertion builder** — pause recording to add `assert_visible`, `assert_text`, and `assert_gone` steps visually.
6. **Replay validation** — replay the generated test in the same UI before saving to verify correctness.

UTO Studio is an enhanced mode of `uto ui` (launched via `uto ui --studio`), not a separate tool. It layers on the Phase 5 server and SPA infrastructure.

See ADR 0016 for full specification, architecture, and Phase 6 delivery plan.

## Competitive Vision and Exit Strategy

UTO is built to displace Cypress and Playwright as the dominant test automation platforms, then be acquired at peak strategic value.

**Three asymmetric advantages:**
1. **Vision-first resilience** — tests survive design system refactors because they recognize elements visually, not by CSS selectors.
2. **Universal platform parity** — web and mobile in one framework, one CLI, one report format.
3. **Zero maintenance overhead** — zero-config provisioning, self-healing (Phase 7), and visual authoring (Phase 6) minimize ongoing test maintenance cost.

**Target acquirers:** Salesforce/Cypress (mobile + vision layer), Microsoft/Playwright (Rust engine + mobile parity), Sauce Labs/BrowserStack (zero-config developer experience layer).

**Target acquisition window:** 2027 Q3–Q4 following Phase 9 community and ecosystem maturity.

See ADR 0017 for full competitive analysis, acquirer profiles, valuation drivers, and market entry strategy.

## Building and Running

This is a standard Rust project. The main application logic is in the `uto-core` crate.

### Build

To build the project, run the following command from the root directory:

```sh
cargo build
```

### Run

To run the main proof-of-concept application:

```sh
cargo build --workspace

# Phase 1 POC (env discovery only)
cargo run -p uto-poc --bin phase1_verify_or_deploy_drivers

# Phase 2 communication layer POC
cargo run -p uto-poc --bin phase2_interact_with_session

# Phase 3 intent API POC (web)
cargo run -p uto-poc --bin phase3_intent_poc

# Phase 3 intent API POC (mobile)
UTO_DEMO=mobile cargo run -p uto-poc --bin phase3_intent_poc

# Phase 3 POC with JSON report to file
UTO_REPORT_FORMAT=json UTO_REPORT_FILE=./phase3-report.json cargo run -p uto-poc --bin phase3_intent_poc

# Phase 3 POC with JSON + HTML reports
UTO_REPORT_FORMAT=json UTO_REPORT_FILE=./phase3-report.json cargo run -p uto-poc --bin phase3_intent_poc -- --html --html-file ./phase3-report.html

# Framework CLI usage (Phase 4 example)
cargo run -p uto-cli -- init ./my-uto-tests --template web --uto-root "$PWD"
cargo run -p uto-cli -- run --project ./my-uto-tests --target web --report-json ./my-uto-tests/.uto/reports/last-run.json
cargo run -p uto-cli -- report --project ./my-uto-tests --html

# Run Phase 4 framework reference project (demonstrates loaders, HTML reporting, web/mobile parity)
cd examples/phases/phase4-framework && cargo run --bin uto_project_runner

# Phase 5 UI mode — generate a report then launch the UI server
cd examples/phases/phase5-ui-mode && \
  cargo run --bin uto_project_runner -- --target web --json --report-file .uto/reports/last-run.json
cargo run -p uto-cli -- ui \
  --project examples/phases/phase5-ui-mode \
  --report examples/phases/phase5-ui-mode/.uto/reports/last-run.json \
  --port 4000

# Validate CLI with generated and phase example projects
./examples/validate-cli.sh
```

### Test

To run any tests, use:

```sh
cargo test
```

## Continuous Integration

GitHub Actions validates the repository with a small Rust CI baseline:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` on `ubuntu-latest`, `macos-latest`, and `windows-latest`

The browser-backed integration tests in `uto-core/tests/session_integration.rs`
skip automatically when `chromedriver` is not available on the runner, so the
cross-platform test job remains stable without custom runner provisioning.

## Development Conventions

*   **Package Management:** Dependencies are managed via `Cargo.toml`.
*   **Project Structure:** The project is a Cargo workspace, with the primary application logic located in the `uto-core` crate.
*   **Crate split for framework UX:** `uto-core` (infrastructure/protocol), `uto-test` (authored test helpers), `uto-runner` (CLI option parsing), `uto-reporter` (report schema + HTML/JSON generation), `uto-logger` (structured logging + spinners), `uto-ui` (interactive UI mode server + embedded SPA), `uto-cli` (project orchestration + UI command).
*   **Design hygiene:** Prefer small files/functions and strict separation of concerns; keep orchestration, protocol, user helper, reporting, and logging responsibilities isolated by crate.
*   **Code Style:** Follow standard Rust conventions and formatting (`rustfmt`).
*   **Error Handling:** The project uses the `thiserror` crate for standardizing application errors.
*   **Linting:** Use `clippy` for identifying common mistakes and improving code quality: `cargo clippy`

### Reporting Architecture (Phase 4+)

- `uto-reporter` owns all report serialization: `UtoReportV1` (single run), `UtoSuiteReportV1` (multi-test suite), JSON round-trip (via serde), and offline HTML rendering
- All generated projects and reference projects depend on `uto-reporter` to emit results
- Report schemas are versioned; see `uto-report/v1` and `uto-suite/v1` constants in `uto-reporter/src/schema.rs`
- HTML rendering is deterministic (no external dependencies, inline CSS, XSS-safe entity escaping)
- Integration points:
   - single-run: `Report::new(enabled, file_path, mode_string)` → `report.event(...)` → `report.finish(...)` → `report.emit()`
   - multi-test: `Suite::new(CliOptions::from_env()).test(...).run().await` → emits `uto-suite/v1` JSON + HTML

### Logging Architecture (Phase 4+)

- `uto-logger` owns all structured logging infrastructure via `tracing` crate
- Global init enforced via `OnceCell`: `let _ = uto_logger::init("component-name")` idempotent per process
- Spinners for long-running phases via `LoaderManager`: `let spinner = loaders.spinner("message"); spinner.success("done")`
- Process awareness built-in: logs include component name and PID for multi-process debugging
- All POC binaries, reference projects, and generated projects initialize logger at startup

## Documentation Habits

*   **`GEMINI.md`:** This file is the primary source of truth for understanding the project at a high level. Keep it updated as the architecture, build process, or core concepts evolve.
*   **GitHub Copilot customization:** Keep `.github/copilot-instructions.md`, `.github/instructions/`, `.github/prompts/`, and `.github/agents/` aligned with `GEMINI.md` and the ADRs as the project evolves.
*   **Gemini/Copilot parity automation:** Run `./scripts/sync_ai_configs.sh` after updating `.github/` customization files, and verify parity with `./scripts/check_ai_config_sync.sh`.
*   **Rustdoc:** All public functions, structs, and enums should be thoroughly documented using standard Rustdoc comments (`///`). This is crucial for generating useful library documentation.
*   **Design Documents:** For significant changes or new features, consider updating or adding to the design documents in the `/docs` directory. This includes the `manifesto.md` and architectural decision records.
*   **Current framework ADRs:** Include ADR 0010 (Phase 4 planning), ADR 0011 (shared `uto-test` helper crate + clean SoC guidelines), ADR 0014 (Phase 5 UI Mode specification), ADR 0015 (downloadable install script and onboarding), ADR 0016 (UTO Studio — visual test authoring), and ADR 0017 (competitive vision and exit strategy).
*   **Commit Messages:** Write clear and concise commit messages that explain the "what" and "why" of a change.
*   **Phase reference examples:** Maintain one committed runnable project per development phase under `examples/phases/` so each phase has a durable implementation reference in-repo.
