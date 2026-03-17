# UTO

Vision-first, cross-platform automation engine in Rust.

UTO (Unified Testing Object) aims to make web and mobile automation feel less brittle by combining a zero-config infrastructure layer with a shared session abstraction, then moving toward vision-driven interaction.

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
- `poc`: executable demos for Phase 1 and Phase 2
- `uto-site`: static site generator for the project landing site
- `docs`: ADRs and project direction documents

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

## Run Demos

### Phase 1: Verify/Provision Drivers

```bash
cargo run -p uto-poc --bin phase1_verify_or_deploy_drivers
```

### Phase 2: Web Session Demo

```bash
cargo run -p uto-poc --bin phase2_interact_with_session
```

### Phase 2: Mobile Session Demo (Appium)

```bash
UTO_DEMO=mobile cargo run -p uto-poc --bin phase2_interact_with_session
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

See ADRs in `docs/` and project context in `GEMINI.md`.
