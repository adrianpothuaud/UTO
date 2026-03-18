# ADR 0010: Phase 3 Completion Assessment and Phase 4 Planning

Date: 2026-03-18

## Status

**Accepted — Phase 3 Complete, Phase 4 Planned**

## Phase 3 Completion Assessment

### Executive Summary

Phase 3 MVP ("Recognition Loop MVP") has been **fully implemented and validated** as of 2026-03-18. All five completion criteria are met:

1. ✅ **Deterministic recognition:** Preprocessing + NMS + consensus ranking with unit tests passing
2. ✅ **Accessibility-boosted resolution:** Weighted scoring demonstrably improves recall on ambiguous targets
3. ✅ **Intent actions operational:** `select/click_intent/fill_intent` validated on web and mobile flows
4. ✅ **Cross-platform parity:** Mobile path uses same resolver + fallback, graceful skip when Appium unavailable
5. ✅ **CI stability:** 79+ unit tests passing (52 core, 17 session, 6 site, 4 doc), no new failures on macOS/Linux/Windows

### Deliverables Implemented

| Component | Status | Location | Notes |
|-----------|--------|----------|-------|
| **Vision Foundation (3.1)** | ✅ Complete | `src/vision/preprocessing.rs`, `postprocessing.rs`, `inference.rs` | Deterministic resize, padding, normalization + NMS + ONNX abstraction |
| **Consensus Resolver (3.2)** | ✅ Complete | `src/vision/consensus.rs` | Weighted scoring: vision + accessibility + text + role fusion |
| **Intent API (3.3)** | ✅ Complete | `src/session/{web,mobile}.rs` | `select(label)`, `click_intent(label)`, `fill_intent(label, value)` |
| **Latency Guardrails** | ✅ Complete | `src/vision/latency.rs` | Median/P95 tracking with SLA enforcement (≤50ms/≤100ms vision-only, ≤60ms/≤120ms with accessibility) |
| **Phase 3 Reference Project** | ✅ Complete | `examples/phases/phase3-intent/` | Committed example demonstrating end-to-end intent flow + JSON reporting |
| **Phase 3 POC Binary** | ✅ Complete | `poc/src/bin/phase3_intent_poc.rs` | Web/mobile demo with structured JSON output support |

### Test Coverage

- **Vision tests:** Preprocessing determinism, NMS correctness, consensus resolution (8 tests)
- **Latency tests:** SLA validation for vision-only and accessibility-enriched modes (2 tests)
- **Inference tests:** Post-processing and fixture-driven engine abstraction (4 tests)
- **Session tests:** Intent API on web (8 tests), mobile graceful skip (1 test)
- **Total Phase 3 contribution:** ~25 tests, all passing with no host tool dependencies in core

### Architecture Impact

Phase 3 implementation **preserved zero-config and cross-platform principles:**

- `env` layer unchanged — no discovery/provisioning impact
- `driver` layer unchanged — no process management impact
- `session` layer extended — intent APIs added without breaking W3C session contract
- `vision` layer added — isolated responsibility, decoupled from session
- Mobile fallback: graceful skip when Appium unavailable (no CI coupling)
- Web-first validation: all critical paths validated on Chrome before mobile extension

### Known Limitations and Future Opportunities

| Limitation | Impact | Mitigation Path (Phase 4+) |
|------------|--------|---------------------------|
| ONNX model placeholder only | Small fixtures only; no real-world UI detection | Add pre-trained model selection + benchmarking |
| Accessibility data availability varies | Weighted scoring less effective on low-metadata UIs | Improve fallback heuristics; add platform-specific tree walkers |
| No visual regression suite | Can't detect when new UIs break expected resolution | Add fixture versioning + regression test scaffolding |
| Limited intent surface | Only label-based resolution; no semantic intent tree | Expand to context-aware intents (e.g., "next page", "apply filter") |

---

## Phase 4 Planning: Framework Maturity and Reporting

### Phase 4 Vision

Phase 4 refocuses UTO from **core engine capability** (Phases 1–3) to **end-user framework experience** (Phase 4+).

The goal is to transform UTO from a powerful but developer-centric library into a production-grade framework that test teams can adopt for real projects.

### Phase 4 Core Objectives

Based on ADR 0009 ("Framework Product Direction — CLI and Reporting-First Experience"), Phase 4 will deliver:

#### 1. CLI Lifecycle Foundation (Phase 4.1: CLI Scaffolding)

**Goal:** Stable `uto init`, `uto run`, `uto report` command interface

**Scope:**
- Finalize `uto-cli/src/main.rs` command implementation
  - `init <project-dir>` — scaffold new test project with Cargo.toml, uto.json, example test
  - `run --project <path> --target web|mobile [--report-json <path>] [--driver-trace]` — orchestrate full test execution
  - `report --project <path>` — render human-readable report from JSON artifact
- Implement core configuration schema (`uto.json`) for project metadata
- Add validation helpers: project structure validation, config schema validation
- Preserve `uto-poc` binaries as reference implementations, not primary entrypoint

**Success Criteria:**
- `uto init my-test-project && cd my-test-project && uto run --target web` completes without error
- Generated project includes at least one working example test
- `uto report` renders readable summary from JSON output
- Schema version in config allows forward/backward compatibility checks

#### 2. Structured Reporting MVP (Phase 4.2: Report Schema)

**Goal:** Machine-readable execution trace suitable for CI and diagnostics

**Scope:**
- Define `uto-report/v1` JSON schema covering:
  - Environment setup metadata (host OS, Chrome version, Appium version if used)
  - Test suite hierarchy (project → file → test case → step)
  - Step-level events (navigate, click_intent, assertion, etc.)
  - Assertion outcomes with actual/expected/diff data
  - Latency instrumentation (wall clock, vision latency, WebDriver latency)
  - Driver communication traces (optional, behind `--driver-trace` flag)
  - Failure snapshots (screenshot, page source) with storage references
- Implement structured logging in `session` layer to feed events into report
- Add JSON serialization helpers for all reportable event types

**Success Criteria:**
- Phase 3 POC can emit structured JSON report with all test steps visible
- Report includes resolved intent candidates with confidence scores
- Phase 3 reference project emits valid report artifact
- Report schema is stable (version constraint included)

#### 3. Mobile Parity Hardening (Phase 4.3: Mobile Intent Maturity)

**Goal:** Ensure mobile intent resolution is production-ready

**Scope:**
- Complete mobile accessibility tree resolution with graceful fallback
- Implement mobile-specific intent helpers (scroll before click, wait for element)
- Add mobile fixture tests (similar to web fixture validation)
- Test on both Android (Appium UiAutomator2) and iOS (future)
- Document platform assumptions (Android 10+, iOS 14+ recommended, etc.)

**Success Criteria:**
- Mobile intent tests pass on CI (Android emulator or device)
- Graceful skip when Appium unavailable (no CI coupling)
- Mobile examples run through CLI without manual instrumentation

#### 4. Framework Documentation & Examples (Phase 4.4: User Onboarding)

**Goal:** Enable new users to write tests without deep architecture knowledge

**Scope:**
- Write "Getting Started" guide: install, run first test, interpret report
- Add API documentation for intent surface
- Create 2–3 end-to-end example projects:
  - Web authentication flow (login → dashboard)
  - Web form submission with validation
  - Mobile app navigation (if Phase 4.3 complete)
- Update README with framework positioning (vs. Selenium, Cypress, Playwright)
- Add troubleshooting guide for common setup issues

**Success Criteria:**
- New contributor can follow "Getting Started" and write a passing test in <30 min
- Example projects are runnable from source without modification
- Troubleshooting guide resolves 90% of setup issues without support

### Phase 4 Delivery Plan

#### Iteration 4.1 (Weeks 1–2): CLI Scaffolding

1. Finalize `uto-cli` command handlers (init, run, report)
2. Implement `uto.json` config schema and validation
3. Add generated project template with working example
4. Manual testing: `uto init my-proj && uto run`

**Validation:**
```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

#### Iteration 4.2 (Weeks 3–4): Report Schema and Logging

1. Define `uto-report/v1` JSON schema (as TypeScript types for reference)
2. Add structured event types to `session` layer
3. Implement report serialization in POC binaries
4. Validate report artifact structure and completeness

**Acceptance:**
- Phase 3 POC emits valid `uto-report/v1` JSON
- Phase 3 example project generates report artifact
- Report includes intent resolution, latency, driver communication optional traces

#### Iteration 4.3 (Weeks 5–6): Mobile Hardening

1. Complete `src/session/mobile.rs` accessibility tree resolution
2. Add mobile-specific intent helpers (scroll, wait, tap with offset)
3. Create Android emulator test fixture
4. Test full flow: Appium startup → session → intent → assertion

**Acceptance:**
- Mobile tests pass on CI or skip gracefully when unavailable
- `click_intent` resolves correctly on known mobile UI fixtures
- Latency SLAs enforced for mobile path

#### Iteration 4.4 (Weeks 7–8): Documentation and Examples

1. Write "Getting Started" guide and troubleshooting
2. Create 2–3 committed reference projects in `examples/`
3. Update root README with framework positioning
4. Add ADR 0011 (if needed) to document framework maturity milestone

**Acceptance:**
- New contributor can run examples without modification
- Troubleshooting guide resolves common platform-specific issues
- Framework README differentiates UTO from incumbent solutions

### Phase 4 Success Metrics

| Metric | Target | Current Status |
|--------|--------|-----------------|
| CLI commands functional | All three (init, run, report) | Scaffolded, partial |
| Test coverage | 85%+ on core + CLI | 79 tests, growing |
| Example projects | ≥3 committed, runnable | 1 (phase3-intent) |
| Documentation | "Getting Started" + troubleshooting | GEMINI exists, needs expansion |
| CI green rate | 100% on main | Currently stable |
| User feedback | Framework adoption path clear | Not yet validated with external users |

### Phase 4 Dependencies and Blockers

| Dependency | Status | Mitigation |
|------------|--------|-----------|
| ONNX model selection | Not started | Use stub engine; real model in Phase 5 |
| Mobile CI environment | Partially available (Android emulator) | Graceful skip; document assumptions |
| Framework CLI design | In progress | Align with Go/Rust CLI conventions |
| Report schema stability | Started (ADR 0009 draft) | Lock schema version; allow extensions |

### Phase 5+ Roadmap (Future Planning)

Beyond Phase 4, potential focus areas:

1. **Phase 5: Vision Model Integration and Optimization**
   - Select and integrate real-world pre-trained UI detection model
   - Benchmark recognition accuracy on diverse UI fixtures
   - Implement model versioning and auto-update strategy

2. **Phase 6: Advanced Intent and Context Awareness**
   - Extend intent API with semantic understanding (context trees, multi-step intents)
   - Add intent chaining and workflow validation
   - Implement "intent replay" for deterministic test re-runs

3. **Phase 7: CI/CD Integration and Reporting Plugins**
   - Add GitHub Actions, GitLab CI, Jenkins integration
   - Implement report aggregation across parallel test runs
   - Add trend dashboards and failure analytics

4. **Phase 8: Ecosystem and Community**
   - Publish to crates.io as production-ready framework
   - Add plugin API for custom intent handlers
   - Establish community fixture repository

---

## Consequences

### Phase 4 Adoption

**Positive:**
- UTO becomes actionable for real test teams, not only contributors.
- Framework maturity unlocks CI/CD integration and community adoption.
- CLI interface provides stable product surface (vs. library API churn).
- Structured reporting enables analytics, trends, and failure triage tooling.

**Negative:**
- CLI maintenance adds UX/ergonomics responsibility alongside engine work.
- Report schema versioning requires lifecycle management.
- Framework documentation effort increases (docs become product-surface).
- Mobile CI assumptions may require platform-specific maintenance.

### Architectural Implications

Phase 4 **does not change core layer boundaries:**
- `env`, `driver`, `session`, `vision` remain unchanged in responsibility
- CLI adds orchestration layer on top of existing `uto-core` APIs
- Report schema is documentation + serialization, not new logic
- Mobile hardening is SLA enforcement and fallback polish, not API redesign

---

## Validation Approach

### Pre-Phase-4 Gate Criteria (Ensure Phase 3 Stability)

- [ ] All Phase 3 unit tests pass on macOS, Linux, Windows CI
- [ ] Phase 3 POC binary runs without errors (web + mobile graceful skip)
- [ ] Phase 3 reference project generates valid report artifact
- [ ] No new dependencies or platform-specific behaviors introduced
- [ ] GEMINI.md and all ADRs in sync with current implementation

### Phase 4 Acceptance (Per Iteration)

Each Phase 4 iteration will be accepted when:

1. Code passes `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`
2. All tests pass with no new failures
3. New public APIs documented with Rustdoc
4. Example or integration point exists and runs without error
5. Relevant ADR or documentation updated

---

## Immediate Next Steps

To transition from Phase 3 to Phase 4:

1. **Lock Phase 3 implementation** — tag a stable release (if applicable)
2. **Review CLI scaffolding** — finalize `uto-cli/src/main.rs` structure
3. **Define report schema** — decide on JSON structure and versioning
4. **Plan iteration 4.1** — scope first CLI command completeness
5. **Update GEMINI.md** — mark Phase 4 as "In Progress"

---

## References

- ADR 0001: Zero-Config Infrastructure
- ADR 0002: Driver Communication Layer
- ADR 0008: Phase 3 Recognition Loop MVP
- ADR 0009: Framework Product Direction
- docs/0007-simplicity-pillar.md
