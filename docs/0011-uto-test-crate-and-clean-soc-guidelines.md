# ADR 0011: Shared `uto-test` Crate and Clean SoC Design Guidelines

Date: 2026-03-18

## Status

Accepted

## Context

Phase 4.1 introduced a simplified end-user test style with helper functions such as:

- `startNewSession("chrome")`
- `startNewSessionWithArg("android", 16)`

The first implementation embedded helper code directly in generated/reference test projects. That improved ergonomics, but it duplicated logic across projects and made helper evolution harder to test and maintain.

At the same time, framework code growth highlighted a need to formalize design hygiene expectations:

- smaller, focused files and functions
- explicit separation of concerns (SoC)
- easy unit/integration testing at each layer

## Decision

1. Introduce a new workspace crate, `uto-test`, as the canonical end-user test helper surface.
   - `uto-test` owns session bootstrap/lifecycle helpers for authored tests.
   - Generated and committed reference projects should import helpers from `uto-test` directly.

2. Keep crate responsibilities explicit:
   - `uto-core`: environment discovery/provisioning, driver lifecycle, W3C session protocol, vision and intent internals
   - `uto-test`: concise test authoring API over `uto-core` primitives
   - `uto-cli`: project scaffolding and run/report orchestration

3. Adopt clean/testable implementation guidance for ongoing work:
   - prefer small functions with one clear responsibility
   - prefer focused modules/files instead of monolithic utility files
   - keep business logic easy to unit test without host tool dependencies
   - keep lifecycle/setup actions observable via logs even when APIs are simplified

4. Keep JavaScript-style helper aliases for onboarding familiarity while exposing idiomatic Rust names:
   - aliases: `startNewSession`, `startNewSessionWithArg`
   - idiomatic: `start_new_session`, `start_new_session_with_hint`

## Consequences

Positive:

- Test projects become simpler to read and write.
- Helper behavior is centralized, versioned, and testable in one crate.
- Framework evolution no longer requires regenerating helper internals in each example project.
- SoC boundaries become clearer for contributors and AI agents.

Tradeoffs:

- Adds one crate to workspace maintenance.
- Requires synchronized dependency updates in template/reference projects.

## Implementation Notes (Current)

- New crate `uto-test` added to workspace.
- `uto-test` modularized into:
  - `managed_session` module: cross-platform managed session wrapper
  - `start` module: target normalization and bootstrap flows
- `uto-cli` templates now depend on `uto-test` directly.
- Phase reference project `examples/phases/phase3-intent` now uses `uto-test` in tests.
- Local per-project helper duplication was removed from the phase example.

## Improvement Audit and Follow-up Backlog

The following opportunities should be addressed incrementally to reinforce this ADR:

1. Extract generated runner source in `uto-cli/src/main.rs` into template files or builder modules to reduce main-file size.
2. Split `uto-cli/src/main.rs` command parsing/execution into submodules (`init`, `run`, `report`, `config`) for smaller test surfaces.
3. Add unit tests in `uto-test` for lifecycle/error-handling branches that do not require host tools.
4. Add targeted integration tests validating generated projects compile with `uto-test` APIs.
5. Add API-level docs page showing minimal test authoring patterns with `uto-test`.

Backlog status update (2026-03-18):

- Item 1 completed by extracting generated runner source into `uto-cli/src/templates.rs`.
- Item 2 completed by splitting CLI execution surfaces into `uto-cli/src/{commands,config,parsing,io}.rs` with a thin `main.rs` entrypoint.
- Item 3 completed via async closed-session error-path tests in `uto-test/src/managed_session.rs`.
- Item 4 completed via real generated-project compatibility checks in `uto-cli/tests/generated_project_compat.rs`.
- Item 5 completed via `docs/0012-uto-test-api-usage-guide.md`.

## Validation

Changes implementing this ADR must continue to pass:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

And generated workflow validation:

```bash
./examples/validate-cli.sh
```
