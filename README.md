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

Phases 1–5 are complete. The framework ships a full CLI lifecycle, structured JSON/HTML reporting, mobile parity, and an interactive UI mode:

- Zero-config browser and SDK discovery + driver provisioning
- Web and mobile automation via a unified session API
- Vision-driven element recognition with accessibility-tree anchoring
- First-class CLI: `uto init`, `uto run`, `uto report`, `uto ui`
- Structured `uto-report/v1` and `uto-suite/v1` schemas with native HTML rendering
- Interactive `uto ui` mode with real-time event stream, watch mode, and report replay

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
uto init ./my-tests --template web
uto run  --project ./my-tests --target web
uto report --project ./my-tests --html
uto ui   --project ./my-tests
```

## Build & Test (from source)

```bash
cargo build --workspace
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Roadmap

- Phase 1 & 2: Zero-config infrastructure + driver communication ✅
- Phase 3: Vision recognition loop + intent API ✅
- Phase 4: Framework CLI, structured reporting, mobile parity ✅
- Phase 5: Interactive UI mode (`uto ui`) ✅
- Phase 6: UTO Studio — visual, selector-free test authoring for web and mobile 🎯
- Phase 7+: Self-healing tests, CI/CD ecosystem integrations, cloud reporting

See `docs/` for architecture decision records and `GEMINI.md` for internal project context.
