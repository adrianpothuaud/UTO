# Test Project Runner Elimination — Planning Summary

**Status:** ✅ **Complete Planning & Documentation**  
**Date:** March 18, 2026  
**Initiated By:** User Request to eliminate `uto_project_runner.rs` boilerplate from generated test projects

---

## Overview

This planning initiative documents how to eliminate the need for generated `uto_project_runner.rs` binary files in UTO test projects, making the framework UX match modern test frameworks like Playwright, Cypress, and WebdriverIO.

### Current Problem
- Every generated test project includes a boilerplate `src/bin/uto_project_runner.rs` file
- This file couples project structure to framework internals
- Requires per-project compilation and binary scaffolding
- UX differs from industry-standard frameworks where CLI owns test execution

### Proposed Solution
- Move test execution logic into `uto-test` library
- Have CLI call `uto_test::run_project_tests()` directly (no subprocess)
- Stop generating runner binary in new projects
- Support backward compatibility during transition

### Expected Outcome
- Cleaner generated test projects (no runner boilerplate)
- Better UX alignment with Playwright/Cypress/WebdriverIO
- Simplified maintenance (runner logic in one place, not per-project)
- Faster `uto run` invocation (no per-project binary compilation)

---

## Deliverables Created

### 1. **ADR 0018: Transparent Test Execution Without Runner Binary** 
📄 [docs/DRAFT-0018-transparent-test-execution-without-runner-binary.md](DRAFT-0018-transparent-test-execution-without-runner-binary.md)

**Purpose:** Formal architectural decision record proposing the improvement

**Content:**
- Full problem statement and goals
- Three implementation options analyzed (A, B, C)
- Recommended approach (Option A: library-based runner in uto-test)
- Four-phase implementation roadmap
- Consequences and tradeoffs
- Open design questions and design choices needed
- Appendices with current template examples and discovery comparison

**Key Sections:**
- **Recommended Direction:** Option A (simplest, nearest-term)
- **Phases:** Prototyping → Deprecation → Macro layer → Performance
- **Timeline:** Phase 4.5 for MVP, Phase 5+ for enhancements

---

### 2. **Architecture Comparison Document**
📄 [docs/ARCHITECTURE-COMPARISON-test-runner-elimination.md](ARCHITECTURE-COMPARISON-test-runner-elimination.md)

**Purpose:** Visual comparisons of current vs. proposed architecture

**Content:**
- Side-by-side ASCII diagrams of current execution pipeline vs. proposed
- Crate responsibility matrix (before & after)
- Industry pattern comparison (Playwright, Cypress, WebdriverIO)
- Migration timeline (5 phases from now through Phase 5+)
- MVP implementation checklist
- Success criteria (6 checkpoints)

**Useful For:**
- Understanding flow changes at a glance
- Showing stakeholders the improvement
- Tracking implementation progress
- Validating success criteria

---

### 3. **Detailed Implementation Guide**
📄 [docs/IMPLEMENTATION-GUIDE-test-runner-elimination.md](IMPLEMENTATION-GUIDE-test-runner-elimination.md)

**Purpose:** Concrete, file-by-file implementation roadmap for developers

**Content:**
- Complete file index with specific line ranges to modify
- Change summaries for each affected file (7 core files)
- Pseudocode for new `dispatch.rs` module
- Example Cargo.toml and template changes
- Test & validation procedures
- Dependency interaction changes (no new deps needed)
- Error handling & migration strategy
- Performance considerations
- Post-MVP enhancement ideas
- Code review checklist

**Critical Sections:**
- **Files to Modify:** uto-test/lib.rs, uto-test/dispatch.rs (NEW), uto-cli/commands.rs, templates.rs
- **Files to Delete:** Example project runner.rs files
- **Files to Update:** Cargo.tomls, GEMINI.md, README.md, uto-site/content
- **Validation:** How to test phase4-framework and ui-showcase still work

---

### 4. **Session Notes — Planning Context**
📄 [/memories/session/runner-elimination-planning.md](/memories/session/runner-elimination-planning.md)

**Purpose:** Session memory tracking analysis and decisions

**Content:**
- Request summary and context
- Current implementation analysis
- Proposed architecture options analyzed
- Recommended MVP approach breakdown
- List of all affected files
- Key design decisions
- Deliverables checklist
- Open questions for user/team
- References to current code locations

---

## Key Decisions ✅

### 1. Implementation Approach: Option A (Recommended)
| Aspect | Decision |
|--------|----------|
| **Test Discovery** | Convention-based (tests/ directory) + `cargo metadata` fallback |
| **Runner Location** | Library function in `uto-test`: `run_project_tests(target, options)` |
| **CLI Integration** | `uto run` calls library function directly (no subprocess for runner) |
| **Compilation** | Keep `cargo build --lib`, but eliminate per-project binary compilation |
| **Macros** | Optional post-MVP enhancement; not required for MVP |
| **Backward Compat** | Support old projects for 1-2 versions; fall back gracefully |

### 2. Implementation Timeline
```
Phase 4.5  (Iteration 1) → Prototyping with one example project
Phase 4.6  (Iteration 2) → Deprecation, migration tooling  
Phase 5+   (Iterations 3-4) → Macros, optimization, full cleanup
```

### 3.Scope & Files Affected
- **New code:** 1 file (`uto-test/src/dispatch.rs` ~100-150 lines)
- **Modified:** 6 existing files (CLI, templates, config)
- **Deleted:** 2 example runner.rs files (cleanup)
- **Updated:** Documentation & site content (alignment only)
- **No new dependencies** required

### 4. Success Criteria
1. ✅ Generated projects have no `src/bin/` directory
2. ✅ `uto run` works without visible compilation per project
3. ✅ JSON/HTML report output unchanged
4. ✅ Suite API fully backward compatible
5. ✅ Example projects execute successfully
6. ✅ Old projects still work (deprecation path)

---

## Implementation Phases

### Phase 1: MVP Prototyping (Iteration 1 — Target: 1-2 weeks)
**Goal:** Prove the concept works with one example project

- Add `uto_test::run_project_tests()` library function with test discovery
- Modify `uto run` command to use library function instead of spawning subprocess
- Validate phase4-framework example runs identically with old and new approaches
- Document any issues discovered

**Success:** `uto run --project examples/phases/phase4-framework` works with clean project structure

---

### Phase 2: Deprecation & Migration (Iteration 2 — Target: 1-2 weeks)
**Goal:** Transition to new approach for new projects while maintaining compatibility

- Update `uto init` to NOT generate runner.rs for new projects
- Add deprecation warning for old projects detected at run time
- Provide `uto migrate` command to auto-clean old projects
- Verify both old and new style projects work

**Success:** New projects are clean, old projects work with deprecation notice

---

### Phase 3: Procedural Macros (Iteration 3 — Post-MVP, optional)
**Goal:** Improve test discoverability with explicit markers

- Design lightweight `#[uto_test]` attribute macro
- Update templates to use macro (optional, not required)
- Improve documentation with explicit test marking examples

**Success:** Tests can be marked with macro for clarity; discovery more robust

---

### Phase 4: Optimization & Polish (Iteration 4 — Post-MVP)
**Goal:** Performance and user experience refinements

- Implement test discovery caching
- Benchmark latency vs. old approach
- Refine error messages
- Full cleanup of backward compatibility code (post v1.0)

**Success:** `uto run` performance ≥ old binary approach; UX smooth

---

## Open Questions for Team Review

1. **Timeline Priority:**
   - Is this a Phase 4.5 priority, or should it wait until Phase 5?
   - Other features competing for 4.4/4.5 bandwidth?

2. **Implementation Scope:**
   - Is Option A (library runner) sufficient, or explore Option B (cargo test integration)?
   - Accept slightly slower discovery for simpler implementation?

3. **Backward Compatibility:**
   - Support old projects indefinitely, or phase out after N versions?
   - Mandatory migration or gentle deprecation?

4. **Procedural Macros:**
   - Required for MVP or truly optional?
   - Worth the build-time complexity for improved discoverability?

5. **Example Projects:**
   - Migrate phase4-framework and ui-showcase immediately or in separate PR?
   - Keep one as reference of migration for documentation?

---

## How to Use This Planning Documentation

### For Project Managers / Decision Makers
→ Read **ADR 0018** (DRAFT-0018-*.md) - 15 min overview of problem, solution, and decision points

### For Developers Starting Implementation
→ Follow **IMPLEMENTATION-GUIDE** (IMPLEMENTATION-GUIDE-*.md) - step-by-step file changes and code patterns

### For Code Reviewers
→ Check **Architecture Comparison** (ARCHITECTURE-COMPARISON-*.md) for visual validation of changes + **Code Review Checklist** at end of Impl Guide

### For Status Tracking
→ Reference **Session Notes** (/memories/session/runner-elimination-planning.md) - context and decisions logged

### For Stakeholders / Documentation
→ Use **Architecture Comparison** diagrams for presentations; shows current pain + proposed benefit clearly

---

## Synchronization Tasks (When Implementation Begins)

Keep in sync as this improvement is built:

| File | Section | Action |
|------|---------|--------|
| GEMINI.md | "Framework product direction" | Update to reflect CLI-owned runner |
| GEMINI.md | "uto-test/src owns..." | Expand scope: test discovery & execution |
| README.md | CLI workflow examples | Remove runner.rs from generated project examples |
| ADR 0009 (CLI/Reporting) | Related ADRs | Link to ADR 0018 |
| ADR 0011 (uto-test) | Enhancement notes | Document expanded uto-test scope |
| .uto-site/content/ | "Getting Started" | Update generated project structure |

---

## Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| **Regression in reports** | Compare JSON/HTML output before/after with automated diff |
| **Old projects break** | Implement backward compatibility fallback (check for runner.rs) |
| **Test discovery misses tests** | Convention + metadata scanning; validate with comprehensive test suite |
| **Slower execution** | Profile and cache aggressively; benchmark vs. old approach |
| **Migration too complex** | Provide automated `uto migrate` command; clear docs |

---

## References & Links

**Core Planning Documents (Created):**
- [ADR 0018 (Draft)](DRAFT-0018-transparent-test-execution-without-runner-binary.md)
- [Architecture Comparison](ARCHITECTURE-COMPARISON-test-runner-elimination.md)
- [Implementation Guide](IMPLEMENTATION-GUIDE-test-runner-elimination.md)
- [Session Notes](/memories/session/runner-elimination-planning.md)

**Existing Related Documentation:**
- [GEMINI.md](GEMINI.md) - Project overview & architecture
- [ADR 0009: Framework CLI and Reporting-First](docs/0009-framework-cli-and-reporting-first.md)
- [ADR 0011: uto-test Crate Design](docs/0011-uto-test-crate-and-clean-soc-guidelines.md)
- [ADR 0014: UI Mode](docs/0014-ui-mode.md)

**Code Locations:**
- Template generation: [uto-cli/src/templates.rs](uto-cli/src/templates.rs)
- Run command: [uto-cli/src/commands.rs](uto-cli/src/commands.rs) (lines ~50-89)
- Suite API: [uto-test/src/suite.rs](uto-test/src/suite.rs)
- Example projects: [examples/phases/phase4-framework](examples/phases/phase4-framework), [ui-showcase](examples/phases/ui-showcase)

---

## Summary

This planning initiative provides **three complementary documents** plus **session notes** that fully document the improvement to eliminate generated runner binaries from UTO test projects:

✅ **ADR (Decision):** What we're doing and why  
✅ **Architecture (Visual):** How the system changes before/after  
✅ **Implementation (Tactical):** Exactly what code to change, line by line  
✅ **Notes (Context):** Planning decisions and team questions  

The improvement is **low-risk** (no new dependencies, backward compatible), **high-value** (better UX, simpler projects, matches industry patterns), and **actionable** (clear MVP and phased roadmap).

**Next Steps:**
1. **Review** ADR 0018 for architectural alignment
2. **Clarify** answers to open questions with team
3. **Assign** implementation to developer (use Implementation Guide as spec)
4. **Validate** with phase4-framework example project first (lowest risk)
5. **Iterate** through phases at whatever pace fits roadmap

---

**Planning Document Version:** 1.0  
**Last Updated:** March 18, 2026  
**Status:** Ready for team review and decision
