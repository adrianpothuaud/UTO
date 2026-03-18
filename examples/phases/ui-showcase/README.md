# Phase 5 — UI Showcase

This is an **interactive showcase project** for **UTO Phase 5 (UI Mode)**.

It demonstrates the full, real-time capabilities of `uto ui`:

- **Live test execution** with real-time event streaming to the browser
- **Interactive test tree** with pass/fail indicators
- **Event detail panels** showing step outcomes
- **Watch mode** for rapid iteration on test code
- **Report replay** of saved test artifacts
- **Cross-platform maturity** — tests run unchanged on web and mobile targets

## Quick Start

### 1. Run Tests & Launch UI (Live Mode)

```bash
# From this directory:
uto ui --open --watch

# Or from repository root:
uto ui --project examples/phases/ui-showcase --open --watch
```

This:
1. Starts the embedded UI server on `http://localhost:4000`
2. Auto-opens your browser
3. Watches for test file changes and re-runs automatically
4. Displays real-time events as they complete

### 2. Generate a Report & Replay It

```bash
# Generate a JSON report
cargo run --bin uto_project_runner -- --target web --json \
  --report-file .uto/reports/last-run.json

# Load and replay it in the UI
uto ui --report .uto/reports/last-run.json --open
```

### 3. One-Shot Test Run from CLI

```bash
# Regular test output
uto run --project . --target web

# Or with HTML report generation
uto run --project . --target web
uto report --html
```

## What You'll See in the UI

### Test Tree Panel (Left)
- **Hierarchical list** of all test files and test cases
- **Colors**:
  - ✅ Green = passed
  - ❌ Red = failed
  - ⏭️ Grey = skipped
  - 🔵 Blue = running
  - ◎ Light = pending
- **Search/Filter** at the top to find tests
- **Test count** badge showing total per file

### Toolbar
- **▶ Run** — Trigger a full suite run (disabled during active run)
- **⏹ Stop** — Stop an in-progress run
- **↻ Replay** — Replay the last loaded report
- **Status chip** — Shows "idle", "running", or "finished"
- **Platform badge** — Shows `web` or `mobile` target

### Event List (Center)
- **Real-time stream** of test steps as they execute
- **Columns**:
  - Stage (setup, step, assertion, cleanup)
  - Status (passed, failed, pending)
  - Detail (human-readable step outcome)
- **Click rows** to expand full JSON context (intent, driver response, etc.)
- **Auto-scroll** — keeps latest event in view

### Test Detail Panel (Bottom Right)
- **Test name & status** when a test is selected
- **Duration** in milliseconds
- **Platform** indicator (web/mobile)
- **Error details** if the test failed

### Summary Bar (Very Bottom)
- **Pass count** (green)
- **Fail count** (red)
- **Skip count** (grey)
- **Total duration**
- **Platform mode**

## Tests Included

### `tests/web_intent_showcase.rs`
Vision-first, selector-free test interactions on a public website:

- **Navigation & assertion** — visit a page and verify content visibility
- **Form interactions** — fill text fields by label, not CSS selectors
- **Multi-step workflows** — navigate, click, fill, assert in sequence
- **Error handling** — gracefully skip if browser unavailable

### `tests/web_visual_confidence.rs`
Demonstrates UTO's recognition confidence and robustness:

- **Element visibility** assertions using intent labels
- **Dynamic content** handling (wait-for patterns)
- **Resilience** to layout changes (vision-first means CSS refactors don't break tests)

## Features You Can Experiment With

### Watch Mode
Make a change to any test file in `tests/`:
```bash
# The entire suite re-runs automatically
# The UI updates in real-time
```

### Report Replay
Run tests, then reload your browser to see the same report twice:
```bash
# First browser window: run tests
uto ui --open

# Second browser window: visit http://localhost:4000
# Both show identical data
```

### Run Filtering
Try the search box in the test tree:
1. Type a test name partial
2. Only matching tests appear
3. Hit "Run" to run just those tests

### Platform Switch
Edit `uto.json` to change `default_target` from `web` to `mobile` (requires Appium):
```json
{
  "default_target": "mobile"
}
```

Then re-run to see the same tests execute on Android/iOS.

## Architecture Notes

### Why This Showcase?

1. **Real-world representative** — Uses actual public websites, not toy apps
2. **Interactive demo** — Designed to be watched live in a browser
3. **Learning resource** — See the UTO intent API in action
4. **Performance observation** — Watch real-time event latency in the UI
5. **Vision foundations** — Demonstrates selector-free test authoring

### No Mock Servers

This project intentionally targets **real, public websites** to show:
- UTO's cross-platform capability (works on any web app, any device)
- Vision-first robustness (survives design changes)
- Real-world timing (no artificial delays)

If you prefer isolated testing, adapt these tests to run against `localhost:3000` or your own test server.

## Measuring Maturity & Next Steps

See the [**MATURITY.md**](./MATURITY.md) document for:
- Phase 5 MVP completion status
- Feature gaps planned for Phase 6+
- Recommendations for immediate improvements
- Research directions (AI step suggestions, time-travel debugging, etc.)

## Troubleshooting

### Chrome not found
```
Error: Chrome executable not found
Solution: Install Google Chrome or ensure it's in PATH
```

### Port 4000 already in use
```bash
uto ui --port 5000 --open
```

### Tests timeout or fail
- Check internet connectivity (real website tests)
- Verify Chrome version matches ChromeDriver
- Review `cargo run` output for detailed error messages

### UI not updating
- Check browser console for JavaScript errors (F12)
- Refresh page (Ctrl+R / Cmd+R)
-Kill and restart the server

## Next Steps

After exploring the UI, check out:

1. **[Modify the tests](./tests/)** — Adapt test URLs/selectors to your own apps
2. **[Read ADR 0014](../../docs/0014-ui-mode.md)** — UI mode architecture & roadmap
3. **[Explore Phase 6 (Studio)](../../docs/0016-uto-studio-visual-test-authoring.md)** — Visual test recording (planned)
4. **[UTO README](../../README.md)** — Full project vision and competitive position

## Version Info

- **UTO Release**: Phase 5 MVP (5.1–5.4)
- **uto-ui Crate**: Fully integrated with CLI
- **Report Schema**: `uto-suite/v1`
- **Tested Platforms**: macOS, Linux, Windows
