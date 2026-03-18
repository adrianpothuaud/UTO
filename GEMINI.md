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

- `uto-cli/` — CLI entrypoint for project lifecycle commands (`init`, `run`, `report`)
- `uto-test/` — end-user test helper crate (simple session start/close API)
- `uto-runner/` — reusable generated-project runner/report infrastructure
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

**Status:** In Progress (Iteration 4.1 baseline complete; Iteration 4.2 next; see ADR 0010)

Phase 4 refocuses UTO from core engine capability toward end-user framework experience. Main objectives:

1. **CLI Lifecycle Foundation** — stabilize `uto init`, `uto run`, `uto report` interface
2. **Structured Reporting** — machine-readable execution traces with latency instrumentation
3. **Mobile Parity Hardening** — production-ready intent resolution on Android via Appium
4. **Framework Documentation** — "Getting Started" guide, troubleshooting, end-to-end examples

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

**Near-term Actions:**
1. Start Iteration 4.2 by defining reusable `uto-report/v1` schema/type surfaces for framework and docs
2. Extend report documentation and examples with stable event semantics and versioning guidance
3. Begin Iteration 4.3 mobile parity hardening and fixture coverage expansion
4. Keep README, static site content, ADRs, and AI instructions synchronized as Phase 4 evolves

See `docs/0010-phase-3-completion-and-phase-4-planning.md` for full Phase 4 planning details, delivery schedule, and success metrics.

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
cargo run --package uto-core
```

This will execute the `main.rs` file, which discovers Chrome, provisions ChromeDriver, and opens a browser window to Google.com for 5 seconds.

For workspace POCs:

```sh
# Phase 2 communication layer POC
cargo run -p uto-poc --bin phase2_interact_with_session

# Phase 3 intent API POC (web)
cargo run -p uto-poc --bin phase3_intent_poc

# Phase 3 intent API POC (mobile)
UTO_DEMO=mobile cargo run -p uto-poc --bin phase3_intent_poc

# Phase 3 intent API POC with JSON report
UTO_REPORT_FORMAT=json cargo run -p uto-poc --bin phase3_intent_poc

# JSON report to file
UTO_REPORT_FORMAT=json UTO_REPORT_FILE=./phase3-report.json cargo run -p uto-poc --bin phase3_intent_poc

# Framework CLI usage
cargo run -p uto-cli -- init ./my-uto-tests --template web --uto-root "$PWD"
cargo run -p uto-cli -- run --project ./my-uto-tests --target web --report-json ./my-uto-tests/.uto/reports/last-run.json
cargo run -p uto-cli -- report --project ./my-uto-tests

# Validate CLI with generated examples
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
*   **Crate split for framework UX:** `uto-core` (infrastructure/protocol), `uto-test` (authored test helpers), `uto-cli` (project orchestration).
*   **Design hygiene:** Prefer small files/functions and strict separation of concerns; keep orchestration, protocol, and user helper responsibilities isolated by crate.
*   **Code Style:** Follow standard Rust conventions and formatting (`rustfmt`).
*   **Error Handling:** The project uses the `thiserror` crate for standardizing application errors.
*   **Linting:** Use `clippy` for identifying common mistakes and improving code quality: `cargo clippy`

## Documentation Habits

*   **`GEMINI.md`:** This file is the primary source of truth for understanding the project at a high level. Keep it updated as the architecture, build process, or core concepts evolve.
*   **GitHub Copilot customization:** Keep `.github/copilot-instructions.md`, `.github/instructions/`, `.github/prompts/`, and `.github/agents/` aligned with `GEMINI.md` and the ADRs as the project evolves.
*   **Gemini/Copilot parity automation:** Run `./scripts/sync_ai_configs.sh` after updating `.github/` customization files, and verify parity with `./scripts/check_ai_config_sync.sh`.
*   **Rustdoc:** All public functions, structs, and enums should be thoroughly documented using standard Rustdoc comments (`///`). This is crucial for generating useful library documentation.
*   **Design Documents:** For significant changes or new features, consider updating or adding to the design documents in the `/docs` directory. This includes the `manifesto.md` and architectural decision records.
*   **Current framework ADRs:** Include ADR 0010 (Phase 4 planning) and ADR 0011 (shared `uto-test` helper crate + clean SoC guidelines).
*   **Commit Messages:** Write clear and concise commit messages that explain the "what" and "why" of a change.
*   **Phase reference examples:** Maintain one committed runnable project per development phase under `examples/phases/` so each phase has a durable implementation reference in-repo.
