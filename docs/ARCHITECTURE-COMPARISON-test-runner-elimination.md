# Runner Elimination: Architecture Comparison

## Current Architecture (Before)

```
┌─────────────────────────────────────────────────────────────────────┐
│ TEST PROJECT STRUCTURE                                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  my-project/                                                         │
│  ├── src/                                                            │
│  │   └── bin/                                                        │
│  │       └── uto_project_runner.rs  ◄── GENERATED BOILERPLATE       │
│  ├── tests/                                                          │
│  │   ├── web_example.rs                                             │
│  │   └── mobile_example.rs                                          │
│  ├── Cargo.toml                                                      │
│  └── uto.json                                                        │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ EXECUTION PIPELINE (Current)                                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  $ uto run --project .                                              │
│      │                                                              │
│      ├─ load uto.json & validate                                   │
│      │                                                              │
│      ├─ spawn: cargo run --bin uto_project_runner -- <options>    │
│      │           │                                                 │
│      │           ├─ cargo compiles src/bin/uto_project_runner.rs │
│      │           │                                                 │
│      │           └─ binary executes:                              │
│      │               ├─ CliOptions::from_env()                    │
│      │               ├─ Suite::new().test().test()                │
│      │               └─ .run().await                              │
│      │                                                              │
│      └─ collect report → .uto/reports/last-run.json               │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────┐
│ CRATE RESPONSIBILITIES (Current)                         │
├──────────────────────────────────────────────────────────┤
│                                                            │
│ uto-cli               | Orchestrates cargo invocation    │
│ uto-runner            | CliOptions parsing for generated │
│                       |   binary                          │
│ uto-test              | Suite API, session helpers       │
│ uto-reporter          | Report schema & rendering        │
│ [Generated Binary]    | Test discovery & execution       │
│                       |   (per project)                   │
│                                                            │
└──────────────────────────────────────────────────────────┘

PAIN POINTS:
❌ Every project has identical boilerplate runner.rs
❌ Compilation step even for unchanged tests
❌ Framework coupled to project structure (src/bin/)
❌ Test discovery logic duplicated in each binary
❌ Harder to evolve runner logic (requires regeneration)
❌ UX differs from Playwright, Cypress, WebdriverIO
```

---

## Proposed Architecture (After — Option A)

```
┌─────────────────────────────────────────────────────────────────────┐
│ TEST PROJECT STRUCTURE                                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  my-project/                                                         │
│  ├── src/                                                            │
│  │   └── [no bin/ directory needed]                                 │
│  ├── tests/                                                          │
│  │   ├── web_example.rs                                             │
│  │   └── mobile_example.rs                                          │
│  ├── Cargo.toml                                                      │
│  └── uto.json                                                        │
│                                                                       │
│  ✅ CLEANER: No boilerplate runner binary                           │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ EXECUTION PIPELINE (Proposed — Option A)                            │
├─────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  $ uto run --project .                                              │
│      │                                                              │
│      ├─ load uto.json & validate                                   │
│      │                                                              │
│      ├─ compile project: cargo build --lib                         │
│      │                                                              │
│      ├─ call uto_test::run_project_tests("web", report_path)      │
│      │   [Rust library function, no subprocess]                    │
│      │      │                                                      │
│      │      ├─ discover tests in tests/ directory                 │
│      │      │   (via cargo metadata / convention)                 │
│      │      │                                                      │
│      │      ├─ execute test functions:                            │
│      │      │   ├─ Suite::new().test(fn1).test(fn2)               │
│      │      │   └─ .run().await                                   │
│      │      │                                                      │
│      │      └─ emit uto-suite/v1 events                           │
│      │                                                              │
│      └─ write report → .uto/reports/last-run.json                 │
│                                                                       │
└─────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ CRATE RESPONSIBILITIES (Proposed)                            │
├──────────────────────────────────────────────────────────────┤
│                                                                │
│ uto-cli               | Load project, orchestrate execution  │
│ uto-test              | Suite API, session helpers,          │
│                       |   test discovery & execution logic   │
│ uto-reporter          | Report schema & rendering            │
│ uto-runner (optional) | Deprecated; kept for                 │
│                       |   backward compatibility             │
│                                                                │
│ NO Generated Binary   | ✅ Removed from scaffold             │
│                                                                │
└──────────────────────────────────────────────────────────────┘

BENEFITS:
✅ Cleaner test projects (no boilerplate)
✅ No per-project compilation overhead (only once via cargo build --lib)
✅ Test discovery centralized in uto-test
✅ Easier to evolve runner logic
✅ Familiar UX (matches Playwright, WebdriverIO)
✅ Smaller Cargo.toml (no src/bin/ entry)
```

---

## Comparative Industry Pattern

```
┌──────────────────────────────────────────────────────────────────────────┐
│ How Other Frameworks Handle Test Execution                              │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│ PLAYWRIGHT (npm)                                                          │
│   $ npx playwright test                                                   │
│   → Framework owns test discovery & execution                            │
│   → No runner binary in project                                          │
│   → Tests are plain .js/.ts files in tests/ dir                          │
│                                                                            │
│ CYPRESS                                                                   │
│   $ cypress run                                                           │
│   → Framework owns runner                                                │
│   → No runner binary/file in project                                     │
│   → Tests are spec files following convention                            │
│                                                                            │
│ WEBDRIVERIO                                                               │
│   $ wdio run wdio.conf.ts                                                │
│   → Framework owns execution; config file, not binary                    │
│   → No runner binary in project                                          │
│   → Simple, focused test files                                           │
│                                                                            │
│ UTO (CURRENT)                                                             │
│   $ uto run --project .                                                  │
│   → Spawns compiled binary from generated src/bin/uto_project_runner.rs │
│   → ❌ Boilerplate runner file in each project                          │
│   ❌ Compilation step per project                                       │
│                                                                            │
│ UTO (PROPOSED)                                                            │
│   $ uto run --project .                                                  │
│   → Framework directly loads & executes tests via uto-test library      │
│   → ✅ No runner binary in project                                       │
│   → ✅ Matches industry patterns                                         │
│                                                                            │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Migration Timeline

```
┌─────────────────────────────────────────────────────────────┐
│ ROLLOUT PHASES                                              │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│ NOW (Phase 4.4)                                             │
│  └─ Document improvement (ADR 0018) [PROPOSAL PHASE]        │
│                                                              │
│ PHASE 4.5 (Early Iteration 1)                              │
│  ├─ Add runner library entry point to uto-test             │
│  ├─ Prototype test discovery & execution                   │
│  ├─ Modify uto-cli/src/commands/run.rs                     │
│  └─ Validate with phase4-framework example                 │
│                                                              │
│ PHASE 4.6 (Iteration 2)                                    │
│  ├─ Stop generating runner.rs in new projects              │
│  ├─ Add deprecation warning for old projects               │
│  ├─ Provide migration documentation                        │
│  └─ Support both old & new styles                          │
│                                                              │
│ PHASE 5.0+ (Beyond MVP)                                    │
│  ├─ Optional #[uto_test] procedural macro                  │
│  ├─ Test discovery caching & optimization                 │
│  ├─ Full cleanup of old runner.rs files                    │
│  └─ UX alignment with UI mode (Phase 5)                    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Checklist (MVP — Iteration 1)

### uto-test Module Changes
- [ ] Add new module: `src/dispatch.rs` — runtime test context & dispatch logic
- [ ] Add `pub async fn run_project_tests(target: &str, options: CliOptions) -> Result<i32>`
- [ ] Implement test discovery via `cargo metadata` + convention (tests/ directory)
- [ ] Load & execute test functions, emit events to reporter

### uto-cli Changes
- [ ] Modify `commands::run::run()` to:
  1. Invoke `cargo build --lib --project <path>`
  2. Call `uto_test::run_project_tests(target,  options)` instead of spawning process
- [ ] Update project validation to not require runner binary

### Template Changes
- [ ] Remove `templates::project_runner_rs()` call from init (or set to empty string)
- [ ] Update `cargo_toml()` template to remove `bin` section

### Example Project Updates
- [ ] Remove `examples/phases/phase4-framework/src/bin/uto_project_runner.rs`
- [ ] Remove `examples/phases/ui-showcase/src/bin/uto_project_runner.rs`
- [ ] Update Cargo.toml files to not reference bin

### Testing
- [ ] `cargo test --workspace` passes
- [ ] `uto run --project examples/phases/phase4-framework` works
- [ ] `uto run --project examples/phases/ui-showcase` works
- [ ] Report output (JSON/HTML) identical to previous implementation
- [ ] All assertions in examples pass

---

## Success Criteria

1. ✅ **Cleaner Projects:** Generated projects have no `src/bin/` directory
2. ✅ **Transparent Execution:** `uto run` works without visible compilation steps to user
3. ✅ **Identical Reports:** JSON and HTML output unchanged from current implementation
4. ✅ **Suite API Preserved:** Existing test code continues to work
5. ✅ **Example Projects Work:** All `examples/phases/*` projects execute successfully
6. ✅ **Backward Compatible:** Old projects with runner.rs still work (migration phase)

