# UTO UI Showcase — Quick Start Guide

This guide gets you up and running with the UTO UI showcase in 5 minutes.

## What You'll Experience

- **Real-time test execution** in a browser UI
- **Live event streaming** as tests run
- **Watch mode** for rapid iteration
- **Report replay** of saved test artifacts
- **Dark/light theme toggle**
- **Interactive test tree** with filtering

## Prerequisites

- ✅ UTO installed and in PATH (`curl -sSf https://raw.githubusercontent.com/adrianpothuaud/UTO/main/install.sh | sh`)
- ✅ Google Chrome installed (for web tests)
- ✅ Rust toolchain (for `cargo` commands)

## 🚀 Option 1: Quick Demo (2 minutes)

```bash
# From the repository root
cd /Users/adrian/UTO

# Launch the interactive UI with watch mode
uto ui --project examples/phases/ui-showcase --open --watch
```

**What happens:**
1. Browser opens automatically to `http://localhost:4000`
2. Test tree appears in the left sidebar
3. Click **▶ Run** button to start tests
4. Watch events stream in real-time to the center panel
5. Tests complete and show results (green ✅ or red ❌)
6. Edit any file in `examples/phases/ui-showcase/tests/` — suite re-runs automatically

## 🎯 Option 2: Generate Report & Replay (3 minutes)

```bash
# Generate a JSON report
cd examples/phases/ui-showcase
cargo run --bin uto_project_runner -- --target web --json \
  --report-file .uto/reports/last-run.json

# Replay it in the UI
uto ui --report .uto/reports/last-run.json --open
```

**What you'll see:**
- Identical test tree as Option 1
- Same pass/fail indicators
- All events are from the saved artifact (not re-running)
- Useful for sharing test results or offline debugging

## 📚 Option 3: Dive Deeper (5+ minutes)

### Modify a Test

```bash
# Edit a test file
vim examples/phases/ui-showcase/tests/web_intent_showcase.rs

# E.g., change example.com to a different website
# Save — the test suite auto-reruns in watch mode
```

### Run Tests from CLI (No UI)

```bash
# Standard test output
uto run --project examples/phases/ui-showcase --target web

# With HTML report
uto report --html
# Opens .uto/reports/latest.html in your browser
```

### Explore the Project Structure

```bash
cd examples/phases/ui-showcase

# View the test code
cat tests/web_intent_showcase.rs

# View project configuration
cat uto.json

# View maturity assessment
cat MATURITY.md
```

## 🎮 UI Controls Explained

### Sidebar
- **Test list** — Click items to select them
- **Search box** — Type to filter tests
- **Count badge** — Total number of tests

### Toolbar
| Button | Action | When Available |
|--------|--------|---|
| ▶ **Run** | Start test suite | When idle |
| ⏹ **Stop** | Stop in-progress suite | During run |
| ↻ **Replay** | Re-run last loaded report | Always |
| Status chip | Current state (idle/running/finished) | Always |
| Platform badge | `web` or `mobile` | Always |

### Event List (Center)
- **Real-time updates** — new events appear every 100-500ms
- **Click row** → expand to see full JSON context
- **Auto-scroll** → latest event stays visible

### Summary Bar (Bottom)
- ✅ Pass count (green)
- ❌ Fail count (red)
- ⏭️ Skip count (grey)
- ⏱️ Total duration
- 🏷️ Platform (web/mobile)

## 🔧 Troubleshooting

### "Chrome not found"
```bash
# Install Google Chrome from https://chromedev.tools/chrome
# Verify it's in PATH:
which google-chrome  # Linux
which /Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome  # macOS
```

### "Port 4000 already in use"
```bash
# Use a different port
uto ui --project examples/phases/ui-showcase --port 5000 --open
```

### "Tests timeout"
- Check internet (tests hit real websites like example.com)
- Verify Chrome is running and responsive
- Review full output: `cargo run --bin uto_project_runner -- --target web`

### "UI not updating"
- Refresh browser (Ctrl+R / Cmd+R)
- Check browser console for JavaScript errors (F12)
- Kill and restart the server (Ctrl+C, then re-run command)

## 📖 Learn More

- **[Full Documentation](./README.md)** — Complete feature list and advanced usage
- **[Maturity Assessment](./MATURITY.md)** — Phase 5 status, Phase 6 roadmap, research directions
- **[ADR 0014](../../docs/0014-ui-mode.md)** — UI Mode architecture and design
- **[ADR 0016](../../docs/0016-uto-studio-visual-test-authoring.md)** — Phase 6 Visual Test Recording (Studio)
- **[API Guide](../../docs/0012-uto-test-api-usage-guide.md)** — Intent-driven test authoring

## ✨ Next Steps

1. **Modify tests** — Change URLs or assertions to see different outputs
2. **Try cross-platform** — Edit `uto.json` to target `mobile` (if Appium is available)
3. **Read MATURITY.md** — Understand Phase 5 completeness and Phase 6 plans
4. **Explore Phase 6 vision** — See how UTO's visual recording will differentiate from Cypress/Playwright

---

**Enjoy exploring UTO Phase 5!**

For questions or issues, see the main [**README**](./README.md) or visit the [**UTO GitHub repository**](https://github.com/adrianpothuaud/UTO).
