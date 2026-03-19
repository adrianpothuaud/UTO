# UTO

**The vision-first, cross-platform test automation engine built to replace Cypress and Playwright.**

UTO (Unified Testing Object) combines a zero-config infrastructure layer, a vision-driven recognition loop, and a unified web + mobile session model into one framework — with a first-class CLI lifecycle (`init`, `run`, `report`, `ui`) and reporting-first diagnostics.

Where Cypress is web-only and Playwright is selector-brittle, UTO delivers selector-free, self-describing tests that run unchanged across Chrome, Android, and iOS.

## Why UTO

- **Selector-free tests** — tests target user intent, not CSS selectors or XPath. Survive design system refactors.
- **Web + mobile in one framework** — same CLI, same API, same report format for Chrome and Appium targets.
- **Zero-config setup** — discover first, provision when missing. No manual driver versioning or PATH configuration.
- **Cross-platform by design** — macOS, Linux, Windows. Clean process lifecycle with explicit shutdown semantics.
- **Reporting-first observability** — structured JSON + HTML reports from every run, from setup through assertion outcomes.
- **Interactive UI mode** — `uto ui` provides a local browser-based interface for running, watching, and debugging tests.
- **Visual test authoring** — UTO Studio (Phase 6) records vision-first, selector-free test code for web and mobile.

## Competitive Position

| Capability | Cypress | Playwright | **UTO** |
|---|---|---|---|
| Web automation | ✅ | ✅ | ✅ |
| Mobile automation | ❌ | ❌ | ✅ |
| Vision-first recognition | ❌ | ❌ | ✅ |
| Selector-free tests | ❌ | ❌ | ✅ |
| Zero-config setup | ❌ | ❌ | ✅ |
| Visual test recorder | Stalled | CLI-only | 🎯 Phase 6 |
| Cross-platform reporting | ❌ | ❌ | ✅ |
| Compiled performance | ❌ | ❌ | ✅ |

## Current Status

Phases 1–5.5 are complete. The framework ships a full CLI lifecycle, structured JSON/HTML reporting, mobile parity, an interactive UI mode with rich test observation capabilities, and library-first ergonomics:

- Zero-config browser and SDK discovery + driver provisioning
- Web and mobile automation via a unified session API
- Vision-driven element recognition with accessibility-tree anchoring
- First-class CLI: `uto init`, `uto run`, `uto report`, `uto ui`
- Structured `uto-report/v1` and `uto-suite/v1` schemas with native HTML rendering
- Interactive `uto ui` mode with real-time event stream, watch mode, report replay, selective test execution, per-step duration, step timeline, and keyboard shortcuts
- **Library-first design**: Use UTO as a Rust library without CLI scaffolding (Phase 5.5)
- **CLI convenience**: Optional `--project` argument with CWD inference (Phase 5.5)

Phase 6 (UTO Studio — visual test authoring) is next.

## Install

macOS / Linux:

```sh
curl -sSf https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.sh | sh
```

Windows (PowerShell):

```powershell
irm https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.ps1 | iex
```

After install, scaffold your first project:

```sh
# Production mode (default): uses crates.io dependencies
uto init ./my-tests --template web

# Development mode: uses path dependencies to UTO source (for UTO contributors)
uto init ./my-tests --template web --dev

# Run tests
uto run  --project ./my-tests --target web
uto report --project ./my-tests --html
uto ui   --project ./my-tests
```

**Note:** Production mode requires UTO crates to be published to crates.io (Phase 5.5.3). Until then, use `--dev` flag to create projects with path dependencies to the UTO source tree.

## Build & Test (from source)

```bash
cargo build --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Local Development Install

To install UTO from your local project directory (useful for testing changes before pushing):

macOS / Linux:

```sh
./install-local.sh
```

Windows (PowerShell):

```powershell
.\install-local.ps1
```

The local installer will:
- Verify you're in the UTO project root
- Check if the current installed version matches your local version
- Skip installation if versions match (use `UTO_FORCE=1` to override)
- Build and install the `uto` binary from `./uto-cli`

Environment variables:
- `UTO_FORCE=1` — Force reinstall even if versions match
- `UTO_INSTALL_DIR` — Override the install directory (default: `$HOME/.cargo`)
- `UTO_SKIP_RUSTUP=1` — Skip automatic Rust installation

## Roadmap

- Phase 1 & 2: Zero-config infrastructure + driver communication ✅
- Phase 3: Vision recognition loop + intent API ✅
- Phase 4: Framework CLI, structured reporting, mobile parity ✅
- Phase 5: Interactive UI mode (`uto ui`) ✅
- Phase 5bis: UTO UI improvements — information richness, streaming time preview, E2E coverage, reportless discovery ✅
- Phase 5.5: Library ergonomics + CLI flexibility (UTO as standalone Rust library, CWD inference) ✅
- Phase 6: UTO Studio — visual, selector-free test authoring for web and mobile 🎯
- Phase 7+: Self-healing tests, CI/CD ecosystem integrations, cloud reporting

See `docs/` for architecture decision records and `GEMINI.md` for internal project context.

## Learning Rust & Contributing

New to Rust? UTO is designed to be beginner-friendly with extensive documentation:

### 📚 Beginner Resources

- **[Learning Path](docs/LEARNING_PATH.md)** — Structured 4-stage guide from Rust basics to UTO contributions
- **[Rust Beginner Guide](docs/rust-beginner-guide.md)** — Comprehensive Rust tutorial with UTO-specific examples
- **[Rust Patterns Cheatsheet](docs/rust-patterns-cheatsheet.md)** — Quick reference for common Rust patterns
- **[Project Structure Guide](docs/project-structure-guide.md)** — Understanding the UTO codebase organization
- **[Testing Guide](docs/testing-guide.md)** — Writing and running tests in UTO

### 🔍 Code Learning Features

The UTO codebase includes educational comments explaining complex Rust patterns:

- **Async traits and trait bounds** — `uto-test/src/suite.rs`
- **Process lifecycle and Drop trait** — `uto-core/src/driver/mod.rs`
- **Error handling patterns** — `uto-core/src/error.rs`
- **Trait objects and dynamic dispatch** — `uto-core/src/session/mod.rs`
- **Test patterns and assertions** — `uto-core/tests/session_integration.rs`

### 🎯 Good First Issues

Look for issues tagged with `good-first-issue` for:
- Documentation improvements
- Test coverage additions
- Error message enhancements
- CLI help text improvements

Start with the [Learning Path](docs/LEARNING_PATH.md) and work through the structured curriculum!
