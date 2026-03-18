# ADR 0018 (DRAFT): Transparent Test Project Execution Without Generated Runner Binary

Date: 2026-03-18

## Status

Draft — Planning phase for test project generation improvement

## Context

### Current State

The UTO CLI currently scaffolds test projects with the following structure:

```
my-project/
  src/bin/uto_project_runner.rs    # Generated binary entry point
  tests/
    web_example.rs
    mobile_example.rs
  Cargo.toml
  uto.json
```

When `uto run --project my-project` is invoked, the CLI orchestrates:
1. `cargo run --bin uto_project_runner -- <options>`
2. The binary parses CLI options via `uto-runner::CliOptions::from_env()`
3. The binary builds a `uto-test::Suite` and executes tests
4. Test results aggregate into JSON and HTML reports

### Problem Statement

This design introduces unavoidable friction for users:

1. **Binary Scaffolding Burden:** Every generated project includes a boilerplate `src/bin/uto_project_runner.rs` that authors do not modify and cannot avoid.

2. **Compilation Step:** Users must wait for `cargo build` to complete before executing tests, even for unchanged code.

3. **Framework Coupling:** The runner binary couples project structure to framework internals (`uto-runner`, `uto-reporter`, `uto-logger`).

4. **Inconsistent UX vs. Industry Leaders:** Competitive frameworks (WebdriverIO, Playwright, Cypress) offer transparent execution:
   - Playwright: `npx playwright test` — no boilerplate runner binary
   - Cypress: `npx cypress open` or `cypress run` — no scaffolded runner
   - WebdriverIO: `wdio run` — no generated runner binary in test projects
   - Each framework owns the test discovery and execution details; projects focus on test authoring

5. **Maintenance Burden:** Changes to runner logic (e.g., new reporting format, option parsing) require regenerating or updating projects or maintaining backward compatibility across versions.

## Goals

1. **Eliminate the generated `uto_project_runner.rs` file** so test projects are cleaner and more focused on test authoring.

2. **Move test discovery and execution into the CLI** so `uto run` directly loads and executes test suites without intermediate binary scaffolding.

3. **Maintain the existing Suite API and reporting infrastructure** so authored tests and existing example projects remain functional.

4. **Preserve zero-config design** — test projects should work without any runner configuration.

5. **Align with industry UX patterns** — make UTO feel like a first-class test framework, not a WebDriver wrapper with boilerplate.

## Proposed Direction

### Phase 1: Library-Based Runner in uto-test

Move runner logic from generated binaries into a reusable library entry point in `uto-test`:

#### uto-test Enhancement
- Add a public `run_project_tests(target: &str, report_file: Option<&str>) -> Result<i32>`  function that:
  - Scans for test functions at runtime (via convention or metadata)
  - Loads and executes test suites
  - Emits events to the reporting stream
  - Returns exit code for CLI

- Add a `#[uto_test]` procedural macro (or attribute) to mark test functions
- Cache compiled test metadata so CLI can discover tests without recompilation

#### uto-cli Enhancement
- Modify `uto run` to:
  1. Detect whether the project is a UTO test project (check `uto.json`)
  2. **Option A (Simpler, nearer term):** Build the project with `cargo build --lib`, then invoke a lightweight standalone runner binary that loads compiled tests dynamically
  3. **Option B (More ambitious):** Use `cargo test` infrastructure directly and parse its events, avoiding custom binary entirely
  4. **Option C (Future):** Embed rustc directly to compile and run tests in-process with zero separate build step

- Deprecate generation of `src/bin/uto_project_runner.rs` in new projects
- Provide a migration path for existing projects (offer a cleanup script or deprecation warning)

#### Generated Project Template
```
my-project/
  src/lib.rs              # Empty or test-focused library root
  tests/
    web_example.rs        # Tests with #[uto_test] attributes (future)
    mobile_example.rs
  Cargo.toml              # No src/bin/ entry, only [lib]
  uto.json
```

### Phase 2: Unified Test Discovery

- Implement test function discovery using:
  - `cargo metadata` to find all test files
  - Symbol scanning via `cargo test --no-run` output, or
  - Procedural macro registry at build time
- Make discovery deterministic and cacheable for fast re-runs

### Phase 3: Consolidate Reporting

- Ensure `uto run` emits identical `uto-suite/v1` and HTML reports regardless of whether tests are run via:
  - Legacy per-project binaries (migration period)
  - CLI-owned runner (new projects)

## Implementation Roadmap

### Iteration 1: Prototyping (Current)
- [ ] Create a minimal runner library entry point in `uto-test` (no macros yet)
- [ ] Prototype `uto run` calling this library function instead of `cargo run --bin`
- [ ] Validate reporting output is identical
- [ ] Test with one example project (e.g., `examples/phases/phase4-framework`)

### Iteration 2: Deprecation & Migration
- [ ] Update `uto init` to **not** generate `src/bin/uto_project_runner.rs` for new projects
- [ ] Add deprecation warning if old projects with runner binary are detected
- [ ] Provide a `uto migrate` helper or documentation for existing projects
- [ ] Keep template for backward compatibility in `LEGACY_MODE`

### Iteration 3: Procedural Macro (Optional, Post-MVP)
- [ ] Design lightweight `#[uto_test]` macro for explicit test marking
- [ ] Avoid heavyweight test framework dependency; keep it minimal
- [ ] Update templates to use macro for clarity

### Iteration 4: Performance & Caching
- [ ] Implement test discovery caching to avoid repeated metadata scans
- [ ] Measure and optimize `uto run` latency vs. `cargo run --bin`
- [ ] Profile cold vs. warm runs

## Consequences

### Positive
- **Cleaner Projects:** Test projects focus on test code, not framework boilerplate.
- **Faster Iteration:** No intermediate binary per project; CLI handles execution.
- **Industry Alignment:** UX matches Playwright, WebdriverIO, Cypress patterns.
- **Simpler Maintenance:** Evolution of runner logic is centralized in one place.
- **Lower Barrier to Entry:** Generated projects look immediately familiar to test authors.

### Tradeoffs
- **Complex Discovery:** Finding and loading tests dynamically is more complex than a generated binary.
- **Compilation Overhead:** Tests must still be compiled before execution (unavoidable in Rust); we can only optimize the orchestration.
- **Migration Burden:** Existing projects with old-style runners must be updated or supported during transition.
- **Procedural Macros:** If we add them, they add build-time complexity (but remain optional for MVP).

## Open Design Questions

1. **Test Discovery Mechanism:**
   - Use `cargo test --no-run` output parsing?
   - Use `cargo metadata` + convention (all `tests/*.rs` files)?
   - Use procedural macro registry (requires macro in each project)?
   
2. **Runner Location:**
   - Keep as library function in `uto-test`?
   - Move to new dedicated `uto-test-runner` crate?
   - Embed directly in `uto-cli`?

3. **Backward Compatibility Duration:**
   - Full support for old projects: 1 version, 2 versions, indefinitely?
   - Provide migration helper for automatic cleanup?

4. **Performance vs. Simplicity:**
   - Is caching/discovery performance critical for the MVP?
   - Or can we accept a slightly slower `uto run` in exchange for simpler implementation?

5. **Test Authoring Style:**
   - Continue supporting the current `Suite::new().test().test().run()` approach?
   - Add `#[uto_test]` macro for consistency with industry patterns?
   - Support both?

## Related ADRs & Decisions

- **ADR 0009:** Framework-level CLI lifecycle and reporting-first UX
- **ADR 0011:** `uto-test` crate for end-user helper APIs
- **ADR 0015:** Installation script and onboarding

## Next Steps

1. **Review & Feedback:** Gather input on proposed approach (library vs. CLI-owned, test discovery strategy).
2. **Prototype:** Build minimal Iteration 1 proof-of-concept with one example project.
3. **Measure:** Compare execution latency, report output fidelity, and user experience.
4. **Refine:** Incorporate findings into final design.
5. **Document:** Update `GEMINI.md`, `README.md`, and project templates with new workflow.

---

## Appendix: Current Generated Runner Example

For reference, the current `src/bin/uto_project_runner.rs` template:

```rust
use uto_runner::{CliOptions, RunMode};
use uto_test::Suite;

#[tokio::main]
async fn main() {
    let _ = uto_logger::init("generated-runner");
    let options = CliOptions::from_env();
    let mode = options.mode;

    let suite = Suite::new(options);
    let exit_code = match mode {
        RunMode::Web => suite
            .test("web: page title is non-empty", web_title_test)
            .test("web: inline form assertion", web_form_test)
            .run()
            .await,
        RunMode::Mobile => suite
            .test("mobile: settings launches", mobile_launch_settings_test)
            .test("mobile: find search intent", mobile_search_intent_test)
            .run()
            .await,
    };

    std::process::exit(exit_code);
}

// ... test function definitions ...
```

This template is generated for every new project and couples test execution to framework internals.

---

## Appendix: Test Discovery Comparison Matrix

| Approach | Discovery Speed | Implementation Complexity | Flexibility | Notes |
|----------|---|---|---|---|
| **Convention-based (tests/ dir)** | Fast | Low | Medium | Simple; requires `cargo test --no-run` parsing |
| **Cargo metadata** | Medium | Medium | Medium | More robust but slower than convention |
| **Procedural macro registry** | Very Fast | High | High | Requires macro in each project; compile-time |
| **Symbol scanning** | Medium | High | High | Complex; language-specific; fragile |

