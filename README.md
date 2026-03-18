# UTO

Vision-first, cross-platform automation engine in Rust.

UTO (Unified Testing Object) aims to make web and mobile automation feel less brittle by combining a zero-config infrastructure layer with a shared session abstraction, then moving toward vision-driven interaction.

The project is also moving toward a framework-style UX with a first-class CLI lifecycle (`init`, `run`, `report`) and reporting-first diagnostics.

## Why UTO

- Zero-config mindset: discover first, provision when missing.
- Cross-platform by design: macOS, Linux, Windows.
- Unified execution model: web (ChromeDriver) and mobile (Appium) behind one session trait.
- Clean process lifecycle: managed driver processes with explicit shutdown behavior.
- Simplicity by default: common pain points (iframes, scroll/fling, context transitions) are handled as reusable defaults.

## Current Status

Phase 1 and Phase 2 are operational:

- Environment discovery and provisioning (`uto-core/src/env`)
- Driver lifecycle management (`uto-core/src/driver`)
- Web and mobile WebDriver communication (`uto-core/src/session`)
- Working POC binaries (`poc/src/bin`)
- In-repo static site generator (`uto-site`)

## Workspace Layout

- `uto-core`: core automation engine (env, driver, session, vision foundation)
- `uto-cli`: framework command-line interface (`uto init`, `uto run`, `uto report`)
- `poc`: executable demos for Phase 1 and Phase 2
- `uto-site`: static site generator for the project landing site
- `docs`: ADRs and project direction documents
- `examples`: CLI-generated smoke projects plus committed per-phase reference projects

## Quick Start

### 1. Build

```bash
cargo build --workspace
```

### 2. Validate

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## AI Contributor Config Sync

UTO supports both GitHub Copilot (`.github/`) and Gemini CLI (`.gemini/`) contributor workflows.

Copilot files are the canonical source, and Gemini files are generated to stay aligned:

```bash
./scripts/sync_ai_configs.sh
```

CI validates this parity with:

```bash
./scripts/check_ai_config_sync.sh
```

## Run Demos

### Phase 1: Verify/Provision Drivers

```bash
cargo run -p uto-poc --bin phase1_verify_or_deploy_drivers
```

### Phase 2: Web Session Demo

```bash
cargo run -p uto-poc --bin phase2_interact_with_session
```

### Phase 3: Intent Resolution POC (Web and Mobile)

**Web (default):**

```bash
cargo run -p uto-poc --bin phase3_intent_poc
```

**Mobile:**

```bash
UTO_DEMO=mobile cargo run -p uto-poc --bin phase3_intent_poc
```

**JSON Report Output:**

```bash
UTO_REPORT_FORMAT=json cargo run -p uto-poc --bin phase3_intent_poc
UTO_REPORT_FORMAT=json UTO_REPORT_FILE=./report.json cargo run -p uto-poc --bin phase3_intent_poc
```

## Framework CLI

UTO provides a first-class CLI for project scaffolding, execution, and reporting:

### Init: Create a New Test Project

```bash
cargo run -p uto-cli -- init ./my-tests --template web --uto-root "$PWD"
```

Generates:
- `Cargo.toml` with uto-core dependency
- `src/bin/uto_project_runner.rs` (local test runner)
- `tests/web_example.rs` and `tests/mobile_example.rs` (sample tests)
- `uto.json` (project config)
- `.uto/reports/` (report directory)

### Run: Execute Tests

```bash
cargo run -p uto-cli -- run --project ./my-tests --target web --report-json ./my-tests/.uto/reports/last-run.json
```

### Report: View Results

```bash
cargo run -p uto-cli -- report --project ./my-tests
```

### Validate CLI End-to-End

```bash
./examples/validate-cli.sh
```

## Committed Phase Examples

UTO keeps one committed example project per development phase under `examples/phases/`.
These projects are intended as stable references, similar to the executable binaries in `poc/src/bin`.

- Phase 3 reference project: `examples/phases/phase3-intent`

### Phase 2: Mobile Session Demo (Appium)

```bash
UTO_DEMO=mobile cargo run -p uto-poc --bin phase2_interact_with_session
```

### Phase 3: Intent API POC

```bash
# Web intent demo
cargo run -p uto-poc --bin phase3_intent_poc

# Mobile intent demo
UTO_DEMO=mobile cargo run -p uto-poc --bin phase3_intent_poc

# JSON report output (stdout)
UTO_REPORT_FORMAT=json cargo run -p uto-poc --bin phase3_intent_poc

# JSON report output to file
UTO_REPORT_FORMAT=json UTO_REPORT_FILE=./phase3-report.json cargo run -p uto-poc --bin phase3_intent_poc
```

## Framework CLI

```bash
# Show help
cargo run -p uto-cli -- help

# Initialize a project
cargo run -p uto-cli -- init ./my-uto-tests --template web --uto-root "$PWD"

# Run tests with JSON report output
cargo run -p uto-cli -- run --project ./my-uto-tests --target web --report-json ./my-uto-tests/.uto/reports/last-run.json

# Summarize the last report
cargo run -p uto-cli -- report --project ./my-uto-tests
```

Validate the end-to-end CLI workflow with generated projects:

```bash
./examples/validate-cli.sh
```

The mobile flow now includes:

- Android/Appium readiness checks
- UiAutomator2 driver verification and doctor checks
- Automatic setup actions when possible
- Appium session creation and Android Settings launch demo

## Static Site (Local Development)

Generate and serve the landing site locally with OS-specific scripts:

- macOS: `./static_site.local.mac.sh`
- Linux: `./static_site.local.linux.sh`
- Windows PowerShell: `./static_site.local.win.ps1`

Optional port argument:

- macOS/Linux: `./static_site.local.mac.sh 5000`
- Windows: `./static_site.local.win.ps1 -Port 5000`

Site source is under `uto-site/`, generated output goes to `uto-site/dist/`.

## Architecture Snapshot

- `env`: host discovery + provisioning + mobile readiness
- `driver`: managed WebDriver-compatible processes
- `session`: W3C protocol communication (`WebSession`, `MobileSession`, `UtoSession`)
- `vision`: Phase 3 foundation (image preprocessing + detection scaffolding)
- `simplicity` (pillar): codified helpers for recurring automation friction (iframe targeting, long-list navigation, stale element recovery)

## Roadmap

- Phase 3: ONNX-backed UI detection and accessibility fusion
- Phase 4: intent-centric API (`select`, `fill`, etc.)
- Framework UX: CLI lifecycle (`init`, `run`, `report`) and reporting-first test execution visibility

See ADRs in `docs/` and project context in `GEMINI.md`.
