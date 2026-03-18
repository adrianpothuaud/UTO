# Phase 5 — UI Mode Reference Project

This is the committed reference project for **UTO Phase 5 (UI Mode)**.

It demonstrates how a UTO project integrates with `uto ui` — the interactive
browser-based test runner and debugger.

## Quick start

```bash
# 1. Generate a suite report
cargo run --bin uto_project_runner -- --target web --json \
  --report-file .uto/reports/last-run.json

# 2. Launch the UTO UI to inspect the report
uto ui --project . --report .uto/reports/last-run.json --open
```

Or via the CLI from the repository root:

```bash
uto run --project examples/phases/phase5-ui-mode --target web
uto ui --project examples/phases/phase5-ui-mode \
       --report examples/phases/phase5-ui-mode/.uto/reports/last-run.json \
       --open
```

### Live run from the UI

Start the UI without a pre-loaded report; click **▶ Run** in the browser toolbar to trigger a live run:

```bash
uto ui --project examples/phases/phase5-ui-mode --open
```

### Watch mode

Pass `--watch` to automatically re-run whenever test files change:

```bash
uto ui --project examples/phases/phase5-ui-mode --watch --open
```

## What this project demonstrates

| Feature | Description |
|---------|-------------|
| `uto ui --report` | Load a saved `uto-suite/v1` artifact and replay it in the UI |
| Test tree panel | Hierarchical list of test cases rendered from the report |
| Pass / fail summary | Visual indicators updated from report data |
| Platform badge | Shows `web` or `mobile` target per test |
| Report replay | `⏮ Replay` button replays the loaded artifact without re-running |
| **Live run** | `▶ Run` button spawns `cargo run --bin uto_project_runner` and streams stdout/stderr as log events |
| **Stop control** | `■ Stop` button terminates the running subprocess |
| **Watch mode** | `--watch` re-runs automatically when `tests/` files change (300 ms debounce) |
| Dark / light theme | Theme toggle persisted to `localStorage` |

## Architecture

This project follows the same pattern as `phase4-framework`:

- `src/bin/uto_project_runner.rs` — Suite runner that emits `uto-suite/v1` JSON
- `tests/smoke_test.rs` — Schema compatibility tests (no driver required)
- `uto.json` — Project configuration consumed by `uto ui` for project name

## Phase 5 delivery plan

| Iteration | Status | Description |
|-----------|--------|-------------|
| **5.1** | ✅ | `uto-ui` crate scaffold + `uto ui` command + embedded SPA |
| **5.2** | ✅ | Report replay + test tree + WebSocket event stream |
| **5.3** | ✅ | Live run integration — subprocess bridge + run/stop controls |
| **5.4** | ✅ | Watch mode (`--watch`) + filesystem watcher |

See `docs/0014-ui-mode.md` for the full Phase 5 ADR.
