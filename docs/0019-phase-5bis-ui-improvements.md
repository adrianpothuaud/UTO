# ADR 0019: Phase 5bis — UTO UI Information Richness, Streaming Time Preview, and E2E Test Coverage

Date: 2026-03-19

## Status

**Complete** — All iterations (5.5, 5.6, 5.7, 5.8) delivered

## Context

Phase 5 delivered UTO's interactive UI mode (`uto ui`) — a local browser-based interface for running, watching, and debugging test suites. All four Phase 5 iterations (5.1–5.4) were completed successfully and the feature is production-ready for its defined MVP scope.

A post-delivery review against Cypress and Playwright UI baselines identified three concrete gaps that prevent the UI from meeting the project's competitive standard before Phase 6 (UTO Studio) begins:

### Gap 1 — Test blocks lack information richness

The sidebar test item shows only the test name and total duration. Event rows show stage / status / the first key of the `detail` map. Specifically missing:

| Missing element | Location | Impact |
|---|---|---|
| Step count + outcome breakdown per test | Sidebar item | Cannot tell at a glance "12 steps, 2 failed" |
| Step-type visual classification | Event rows | Navigate, intent, assert, and setup steps are indistinguishable |
| Per-step duration | Event rows | No per-step timing; only the test-total duration is visible |
| Live active-step highlight | Event rows | Running step looks identical to completed ones during a live run |
| Smart `detail` formatting | Event rows | `fmtDetail()` returns only `keys[0]`; intent candidates, assertion diffs, error payloads are truncated |
| Error summary indicator | Sidebar item | Failed tests require clicking into the test to see the error |

### Gap 2 — No streaming time preview

Cypress shows a horizontal command log with a per-step timeline. Playwright shows a time-ordered waterfall with per-step duration in ms and a current-step highlight. The UTO UI renders all events as identical flat table rows with no temporal dimension:

- Events have no per-row timestamp — only test-level `timeline.duration_ms` is available.
- There is no horizontal timeline strip showing step proportions and pacing.
- There is no live progress indicator showing the currently executing step.
- There is no visual representation of whether a suite is fast or slow at a step level.

During a live run, the event list grows but the only feedback on timing comes from the total duration in the summary bar, which is only populated after the run completes.

### Gap 3 — No end-to-end test coverage

Existing tests in `uto-ui` are:

| Module | Coverage | Gap |
|---|---|---|
| `server.rs` | HTTP route tests: 200 responses, JSON shape, project name derivation | No WebSocket integration; no run lifecycle sequence tested |
| `runner.rs` | Unit tests for `KillHandle`, `status_from_report`, `load_report_file` | No subprocess spawn lifecycle tested |
| `watcher.rs` | Watcher creation smoke test | No debounce or callback-fires test |

There is no test that:
- Creates a live server and connects a real WebSocket client
- Verifies the `trigger_run` → `run_started` → `log*` → `run_finished` event sequence
- Guards the multi-client broadcast relay against regression
- Verifies a late-joining client receives the preloaded report on connect
- Catches SPA structural regressions (missing toolbar buttons, event table headers, summary bar)

Without this coverage, any refactor of `server.rs` or `runner.rs` has no regression net.

## Decision

UTO will deliver **Phase 5bis** across three focused iterations that correct the three gaps above without altering any Phase 1–5 layer boundaries or introducing new top-level crates.

---

## Iteration 5.5 — Test Block Information Richness

### Objectives

1. **Per-step duration in event rows** — add a `Duration` column (⧗ `42ms`) to the event table alongside Stage / Status / Detail.
2. **Step-type visual icons** — classify event rows by stage keyword and render a distinct icon prefix:
   - `setup` / `driver` → ⚙
   - `navigate` / `goto` → 🧭
   - `intent` / `click` / `fill` / `select` → 👆
   - `assert` → ✓
   - `log` → 📋
   - everything else → ◎
3. **Sidebar outcome strip** — each test item in the sidebar shows a compact row of colored dots (one per event, green/red/yellow) so failure position is visible at a glance without clicking.
4. **Live active-step pulse** — when a live run is in progress, the last-appended event row receives a `running` CSS class with a left-border pulse animation; removed when the next event arrives.
5. **Smart `fmtDetail()` formatting** — replace the single-key fallback with stage-aware formatting:
   - `intent` events: show `label → top candidate (confidence%)`
   - `assert` events: show `expected: X  actual: Y`  
   - `error` events: show error message directly
   - `navigate` events: show the URL
   - fallback: existing `keys[0]` behaviour preserved
6. **Error badge in sidebar** — failed test items display a red ❌ indicator and the first line of `t.error` as a tooltip / subtitle.

### Server-side change

None. Iteration 5.5 is SPA-only.

---

## Iteration 5.6 — Streaming Time Preview

### Objectives

1. **Per-event `ts_ms` field** — `runner.rs` attaches a wall-clock offset (milliseconds since run start) to every event it broadcasts:
   - `log` events: `{ "type": "log", "payload": { "line": "…", "ts_ms": 312 } }`
   - `run_started` and `run_finished` events carry `ts_ms: 0` and the final elapsed ms respectively.
   - Report-replay events use the existing `timeline.start_time` / `duration_ms` fields already present in `uto-report/v1` — no schema change needed.
2. **Step timeline strip** — a horizontal bar strip rendered between the test detail header and the event table:
   - One proportional bar per event, coloured by outcome (green / red / yellow / blue for running).
   - Bar width is proportional to `ts_ms` delta relative to total run duration.
   - Hovering a bar shows a tooltip: `stage · duration · detail excerpt`.
   - Clicking a bar scrolls the event table to the corresponding row.
3. **Live progress** — during a run the timeline strip grows as events arrive; the rightmost bar pulses (CSS animation).
4. **Toolbar progress indicator** — a thin progress bar under the toolbar shows elapsed time as a fraction of the last-run duration (resets to zero for first run where no baseline exists).
5. **Minimum bar width** — events with `ts_ms` delta of 0 (simultaneous) receive a 4 px minimum width so they remain visible and clickable.

### Timestamp strategy: Option 1 (server-side relay)

`runner.rs` records `run_start = std::time::Instant::now()` at the moment it sends `run_started`. Every subsequent event broadcast from the subprocess relay includes `"ts_ms": run_start.elapsed().as_millis()` in its payload. This approach:

- Is accurate regardless of network latency variation between server and browser.
- Requires no client-side `Date.now()` bookkeeping.
- Is independently testable in Rust unit tests.
- Ships zero schema changes to `uto-reporter` — `ts_ms` is a UI-relay field only.

For **report replay**, the SPA derives timestamps from the `uto-report/v1` `timeline` fields already present on each event. No additional server work is required for replay mode.

---

## Iteration 5.7 — End-to-End Test Coverage

### Objectives

Add `uto-ui/tests/` integration tests that cover the full request / WebSocket lifecycle using `axum` route testing and `tokio-tungstenite` for real WebSocket client connections.

## Iteration 5.8 — Reportless Discovery and Selective Execution

### Objectives

1. **Reportless test discovery** — UI mode must list tests before any run/report by discovering source tests from `<project>/tests/*.rs`.
2. **Discovery API** — add `GET /api/tests` returning discovered `{ test_bin, test_name, target }` records.
3. **Run selected test** — add UI support for selecting one discovered test and triggering only that test.
4. **WebSocket payload selection** — `trigger_run` accepts optional payload `{ test_bin, test_name }`.
5. **CLI-backed selective run** — `uto run` accepts optional `--test-bin` and `--test-name` filters and executes only that case in CLI-owned mode.
6. **Legacy compatibility rule** — selective run flags are rejected for legacy runner projects with a clear migration error.

### Test suite

| Test | What it validates |
|---|---|
| `ws_client_receives_report_on_connect` | Server with preloaded report → connect WS client → first message is `{ type: "report", payload: … }` |
| `ws_client_receives_run_started_on_trigger` | Send `trigger_run` message → server broadcasts `run_started` within 500 ms |
| `ws_second_trigger_while_active_sends_run_ignored` | Trigger run → immediately trigger again → second response is `run_ignored` |
| `ws_stop_run_sends_run_finished_stopped` | Start run → send `stop_run` → `run_finished` payload has `status: "stopped"` |
| `ws_late_join_receives_preloaded_report` | Load report at startup → connect first client → connect second client later → both receive report push |
| `ws_broadcast_reaches_all_connected_clients` | Connect two clients → trigger an event → both clients receive it |
| `api_report_returns_updated_report_after_run` | Run completes → `GET /api/report` returns the freshly written artifact |
| `index_html_contains_required_structure` | `GET /` → HTML contains `id="btn-run"`, `id="events-list"`, `id="summary-bar"`, `id="test-list"` |

### E2E test infrastructure

- Tests live in `uto-ui/tests/integration.rs`.
- A shared `spawn_test_server(opts)` helper binds on port 0 (OS-assigned), returns the `SocketAddr` and a `broadcast::Sender` for injection.
- WebSocket client uses `tokio-tungstenite` (added as a `[dev-dependency]`).
- All tests are `#[tokio::test]` and do not require browser, ChromeDriver, or Appium.
- Tests that exercise subprocess spawning are gated with `#[cfg(not(ci_subprocess_disabled))]` so the standard `cargo test --workspace` baseline remains fast and environment-free.

---

## Phase 5bis Done Criteria

- [x] Event rows display per-step duration and step-type icon.
- [x] Sidebar test items show outcome strip and error summary for failed tests.
- [x] `fmtDetail()` applies stage-aware formatting for intent, assert, navigate, and error events.
- [x] Live active-step pulse animation is present during a run.
- [x] `runner.rs` attaches `ts_ms` (wall-clock offset from run start) to every broadcast event.
- [x] Horizontal step timeline strip renders in test detail view (live and replay).
- [x] Timeline strip bars are clickable and scroll the event table.
- [x] Toolbar progress bar is present during live runs.
- [x] `uto-ui/tests/integration.rs` covers all 8 test cases listed above.
- [x] UI mode shows discovered tests from source with no preloaded report.
- [x] UI mode can run a selected discovered test (single-case execution).
- [x] `cargo test --workspace` passes on macOS, Linux, and Windows CI with no new failures.
- [x] All Phase 1–5 layer boundaries remain intact (no changes to `uto-core`, `uto-reporter`, `uto-test`; `uto-cli` adds optional run filter flags only).

---

## Architecture Implications

### What changes

| Component | Change |
|---|---|
| `uto-ui/src/assets/index.html` | SPA: step-type icons, outcome strip, timeline strip, toolbar progress bar, smart detail formatting, active-step pulse, reportless discovery rendering, run-selected control |
| `uto-ui/src/server.rs` | Adds `GET /api/tests` source discovery endpoint and `trigger_run` payload parsing for selection |
| `uto-ui/src/runner.rs` | Adds `run_start: Instant` + `ts_ms` field to all broadcast events and optional selected-test forwarding |
| `uto-ui/tests/integration.rs` | New — integration test suite |
| `uto-ui/Cargo.toml` | Adds `tokio-tungstenite` and `futures-util` as `[dev-dependency]` |
| `uto-cli/src/parsing.rs` + `uto-cli/src/commands.rs` | Adds optional `--test-bin` / `--test-name` filters for CLI-owned selective runs |

### What does not change

- `uto-core`, `uto-reporter`, `uto-test`, `uto-runner` — untouched.
- `uto-cli` top-level commands are unchanged; `uto run` gains optional `--test-bin` / `--test-name` filtering flags.
- `uto-report/v1` and `uto-suite/v1` schemas — untouched; `ts_ms` is relay-only metadata.
- Phase 5 WebSocket message protocol remains additive: `ts_ms` fields and optional `trigger_run.payload` selection are backward-compatible.
- All existing `uto-ui` unit tests continue to pass unchanged.

---

## Validation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

The integration tests are included in `cargo test --workspace` and run without any host tool dependencies.

---

## References

- ADR 0014: UTO UI Mode — Interactive Test Debugging and Visualization
- ADR 0016: UTO Studio — Visual Test Authoring and Recording
- ADR 0009: Framework Product Direction — CLI and Reporting-First Experience
- `uto-ui/src/runner.rs` — subprocess bridge (ts_ms change lives here)
- `uto-ui/src/assets/index.html` — SPA (UI improvements live here)
