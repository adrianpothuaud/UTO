# UTO UI Showcase — Documentation Index

This directory contains comprehensive documentation on UTO Phase 5 (UI Mode) maturity assessment and strategic planning for Phase 6.

## Quick Navigation

**Start here depending on your goal:**

| Goal | Read | Time |
|------|------|------|
| **Get UTO UI running in 5 min** | [QUICKSTART.md](./QUICKSTART.md) | 5 min |
| **Understand what you can do with the UI** | [README.md](./README.md) | 15 min |
| **Assess Phase 5 completeness** | [MATURITY.md](./MATURITY.md) | 20 min |
| **Understand Phase 6 strategy** | [SUMMARY.md](./SUMMARY.md) | 25 min |
| **Learn Phase 5 architecture** | [ADR 0014](../../docs/0014-ui-mode.md) | 20 min |
| **Understand Phase 6 (Studio) vision** | [ADR 0016](../../docs/0016-uto-studio-visual-test-authoring.md) | 15 min |
| **View full competitive analysis** | [ADR 0017](../../docs/0017-competitive-vision-and-exit-strategy.md) | 20 min |

---

## Documents in This Project

### 🚀 Getting Started

1. **[QUICKSTART.md](./QUICKSTART.md)**  
   5-minute guide to launch the UI and run your first tests. Perfect for first-time users.
   
   Contents:
   - Prerequisites check
   - 3 quick demo options
   - 5 UI control explanations
   - Troubleshooting (common errors + fixes)

### 📖 Usage & Features

2. **[README.md](./README.md)**  
   Comprehensive documentation of the showcase project and all UI mode features.
   
   Contents:
   - Quick start (3 variations)
   - What you'll see in the UI (detailed breakdown)
   - Test descriptions
   - Feature experiments (watch mode, report replay, filtering, platform switch)
   - Architecture notes
   - Troubleshooting guide
   - Next steps for exploration

### 🔍 Maturity Assessment

3. **[MATURITY.md](./MATURITY.md)**  
   Deep dive into Phase 5 completion status, quality assessment, and Phase 6 roadmap.
   
   Contents:
   - Executive summary (7.5/10 maturity score)
   - 14 implemented features vs. 8 known gaps
   - Quality assessment scorecard
   - Phase 6.0-6.3 roadmap (timing + effort estimates)
   - Competitive analysis (UTO vs. Cypress vs. Playwright)
   - Success metrics
   - Risks & mitigation strategies
   - Appendix with reference links

### 📊 Strategic Summary

4. **[SUMMARY.md](./SUMMARY.md)**  
   Executive-level overview tying everything together: business value, action items, and go-to-market strategy.
   
   Contents:
   - Overview of what we built (project + assessment)
   - Phase 5 feature status table
   - Phase 6 roadmap (Q3 2026 onwards)
   - Why Phase 6 matters (competitive positioning)
   - 30-day action items (high/medium/low priority)
   - Maturity lessons learned
   - Success metrics
   - Strategic questions (monetization, timing, risk mitigation)

---

## Architecture & Design Documents (Elsewhere)

These are the foundational ADRs for UTO that inform the showcase project:

- **[ADR 0014 — UI Mode](../../docs/0014-ui-mode.md)**  
  Official design for Phase 5 (UI Mode). Describes server architecture, feature scope, event protocol.
  
  Read this to understand:
  - Why we built `uto-ui` as an Axum + WebSocket server
  - What MVP features are "done" vs. post-MVP
  - Technical decisions (offline-first, embedded SPA, no CDN)

- **[ADR 0016 — UTO Studio](../../docs/0016-uto-studio-visual-test-authoring.md)**  
  Proposal for Phase 6 visual test recording. Explains how Studio will differ from Cypress Studio and Playwright Codegen.
  
  Read this to understand:
  - Why visual test recording is critical for adoption
  - How vision-first recording differs from selector-brittle approaches
  - Planned capabilities (cross-platform, AI suggestions, Rust code generation)

- **[ADR 0017 — Competitive Vision](../../docs/0017-competitive-vision-and-exit-strategy.md)**  
  Full competitive analysis and long-term business vision for UTO.
  
  Read this to understand:
  - How UTO compares to Cypress/Playwright/TestCafe on every dimension
  - Market opportunity and exit strategies
  - Why UTO's vision-first approach is the right bet

- **[ADR 0001 — Zero-Config Infrastructure](../../docs/0001-zero-config-infrastructure.md)**  
  Explains UTO's core infrastructure philosophy (auto-discover, auto-provision drivers).

- **[ADR 0003 — Vision Foundation](../../docs/0003-vision-foundation.md)**  
  Details on element recognition and the "recognition loop" that makes UTO selector-free.

---

## Project Files

### Configuration
- **`uto.json`** — Project configuration (schema version, target, paths)
- **`Cargo.toml`** — Project dependencies

### Source Code
- **`src/bin/uto_project_runner.rs`** — Test suite runner (invoked by `uto run`)
- **`src/lib.rs`** — Stub library (tests are in `tests/` directory)

### Test Code
- **`tests/web_intent_showcase.rs`** — Interactive web tests (vision-first, selector-free)
- **`tests/ui_features_test.rs`** — Integration tests validating schema compatibility

### Reports
- **`.uto/reports/`** — Generated JSON/HTML reports (created by test runs)

---

## Reading Recommendations

### For First-Time UTO Users
1. Start with [QUICKSTART.md](./QUICKSTART.md) — get the UI running
2. Explore the UI while reading [README.md](./README.md) — understand features
3. (Optional) Read [ADR 0014](../../docs/0014-ui-mode.md) if you want technical context

### For Product Managers / Stakeholders
1. Skim [QUICKSTART.md](./QUICKSTART.md) — see it in action (demo)
2. Read [SUMMARY.md](./SUMMARY.md) — strategic overview and Phase 6 roadmap
3. Read [ADR 0017](../../docs/0017-competitive-vision-and-exit-strategy.md) — business positioning

### For Developers / Architects
1. Read [MATURITY.md](./MATURITY.md) — understand what's done vs. what's next
2. Read [ADR 0014](../../docs/0014-ui-mode.md) — understand `uto-ui` architecture
3. Read [ADR 0016](../../docs/0016-uto-studio-visual-test-authoring.md) — understand Phase 6 design
4. Explore `src/` and `tests/` — see test authoring patterns

### For Phase 6 Planning
1. Read [SUMMARY.md](./SUMMARY.md) sections on Phase 6 roadmap
2. Read [ADR 0016](../../docs/0016-uto-studio-visual-test-authoring.md) — Studio design
3. Read [MATURITY.md](./MATURITY.md) section on research directions
4. (Schedule) Deep dive on Phase 6 architecture & effort estimation

---

## How This Project Was Designed

### Goals
1. **Showcase Phase 5 maturity** — prove the UI mode is production-ready
2. **Enable hands-on exploration** — let teams run tests interactively
3. **Document strategic thinking** — articulate Phase 6 vision and next steps
4. **Inform release planning** — provide recommendations for the next phases

### Structure
- **README.md** — Vanilla project documentation (how to use it)
- **QUICKSTART.md** — Onboarding guide (fastest time-to-value)
- **MATURITY.md** — Technical assessment (what's done, what's next, quality)
- **SUMMARY.md** — Strategic overview (business context, roadmap, actions)
- **Examples & Tests** — Real, runnable code that demonstrate UTO capabilities

### Audience
- **End users**: QUICKSTART.md → README.md
- **Team leads**: SUMMARY.md + MATURITY.md
- **Architects**: MATURITY.md + ADRs
- **Researchers**: SUMMARY.md (research directions) + ADRs

---

## Quick Reference

### Commands

#### Launch Interactive UI
```bash
# From the showcase project directory:
uto ui --open --watch

# From repository root:
uto ui --project examples/phases/ui-showcase --open --watch
```

#### Generate & Replay Report
```bash
cargo run --bin uto_project_runner -- --target web --json --report-file .uto/reports/last-run.json
uto ui --report .uto/reports/last-run.json --open
```

#### Run Tests from CLI
```bash
uto run --project examples/phases/ui-showcase --target web
uto report --html
```

#### Validate Project
```bash
cargo test --test ui_features_test
```

### Key Files
- **Project config:** `uto.json`
- **Test code:** `tests/*.rs`
- **Runner:** `src/bin/uto_project_runner.rs`
- **Reports:** `.uto/reports/`

### Maturity Snapshot
- **Phase 5 Status:** MVP complete (7.5/10 maturity)
- **Phase 6 (Studio):** Planned, 8-10 weeks effort
- **Critical Gap:** Screenshot timeline (planned for 6.1)
- **Quick Win:** Add keyboard shortcuts (1 day)

---

## Questions or Issues?

- **How do I run this?** → [QUICKSTART.md](./QUICKSTART.md)
- **What features are in Phase 5?** → [README.md](./README.md) or [MATURITY.md](./MATURITY.md)
- **What's coming in Phase 6?** → [SUMMARY.md](./SUMMARY.md) or [ADR 0016](../../docs/0016-uto-studio-visual-test-authoring.md)
- **How does UTO compete?** → [ADR 0017](../../docs/0017-competitive-vision-and-exit-strategy.md)
- **How do I write tests?** → [README.md](./README.md) or [API Guide](../../docs/0012-uto-test-api-usage-guide.md)

---

**Last updated:** March 18, 2026  
**Project:** UTO UI Showcase (Phase 5 Reference)  
**Location:** `examples/phases/ui-showcase/`
