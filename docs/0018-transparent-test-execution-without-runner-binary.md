# ADR 0018: Transparent Test Project Execution Without Generated Runner Binary

Date: 2026-03-18

## Status

Accepted - Phase 4.5 implementation target

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

This design introduces avoidable friction for users:

1. Binary scaffolding burden: every generated project includes a boilerplate `src/bin/uto_project_runner.rs`.
2. Framework coupling: project layout is coupled to framework internals.
3. Maintenance burden: runner changes require template churn and migration pressure.
4. UX mismatch with Playwright/Cypress/WebdriverIO, where runner mechanics are owned by the framework CLI.

## Decision

UTO will implement transparent test execution in Phase 4.5 with Option A.

### 1. Approach Selection

Option A is accepted as the implementation path:

- Move runner orchestration into framework-owned logic.
- Remove generated `src/bin/uto_project_runner.rs` from newly initialized projects.
- Keep the project authoring surface focused on tests, not runner binaries.

### 2. MVP Scope Includes `#[uto_test]`

`#[uto_test]` is included in the MVP scope, not deferred.

- Generated examples should use `#[uto_test]` in Phase 4.5 delivery.
- Existing `Suite` patterns remain supported for compatibility and incremental migration.

### 3. Example Migration Timing

Migrate both committed phase projects immediately in the Phase 4.5 implementation stream:

- `examples/phases/phase4-framework`
- `examples/phases/ui-showcase`

These examples should be updated in the same delivery window as CLI/template changes so reference projects remain aligned.

### 4. Backward Compatibility Window

Legacy runner-binary projects are supported for two minor releases after Phase 4.5 lands.

- Compatibility window: `N = 2` minor versions.
- During this window, `uto run` must continue to execute legacy projects with a clear deprecation warning.
- After the window, legacy runner mode is removed and projects must be migrated.

## Implementation Plan (Phase 4.5)

1. Add framework-owned test execution path and keep report parity (`uto-suite/v1` + HTML).
2. Update `uto init` templates to stop generating `src/bin/uto_project_runner.rs`.
3. Introduce `#[uto_test]` in generated examples and framework docs.
4. Migrate `phase4-framework` and `ui-showcase` examples immediately.
5. Add deprecation messaging and compatibility guardrails for legacy projects.

## Consequences

Positive:

- Cleaner generated projects with less boilerplate.
- UX aligns with mainstream framework expectations.
- Centralized evolution of runner behavior.
- Better onboarding and lower cognitive overhead.

Tradeoffs:

- Transitional complexity while supporting both project styles.
- Macro support adds build/tooling surface that must be documented and tested.

## Compatibility and Removal Policy

1. Phase 4.5 + next two minor releases: legacy runner mode supported with warnings.
2. At end of second minor release window: legacy runner mode removed.
3. Migration guidance must be available before removal (CLI messaging and docs).

## Related ADRs

- ADR 0009: Framework CLI and reporting-first direction
- ADR 0011: `uto-test` ownership and SoC guidelines
- ADR 0013: Getting started and troubleshooting experience
- ADR 0015: Installation and onboarding
