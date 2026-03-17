# ADR 0009: Framework Product Direction — CLI and Reporting-First Experience

Date: 2026-03-17

## Status

Accepted

## Context

UTO currently demonstrates core engine capabilities through `uto-poc` binaries and module-level APIs.

To be usable as an automation framework in the same category as Selenium, WebdriverIO, Cypress, and Playwright, UTO must provide:

1. a clear project-level CLI workflow (`init`, `run`, `report`)
2. strong user-facing observability from environment setup through assertions
3. consistent, structured reporting across web and mobile runs

Without these, technical capabilities remain difficult to operationalize for test authors and CI pipelines.

## Decision

UTO will adopt a **reporting-first framework UX** as a product objective for Phase 3 onward.

### 1. CLI Lifecycle Objective

UTO will converge on a first-class CLI surface with at least three workflow commands:

- `uto init` — scaffold a new automation project
- `uto run` — execute test suites with platform/profile options
- `uto report` — render machine-readable and human-readable results

During the transition period, existing `uto-poc` binaries remain valid implementation references and validation tools.

### 2. Reporting Scope Objective

Each run must produce a clear execution trace that can answer:

- what environment setup actions were attempted and with what outcomes
- what drivers were started/stopped and where
- what intents were resolved, with candidate ranking context
- what element/action/assertion steps passed or failed
- what driver-facing requests/responses were relevant to failures

### 3. Structured Output Objective

UTO will support structured report output (JSON as baseline) suitable for:

- CI artifact storage
- trend dashboards
- failure triage tooling

Human-readable logs remain supported, but structured output becomes a required path.

## Milestone Objectives

### Near-term (Phase 3 hardening)

1. POC binaries expose optional JSON output with end-to-end step events.
2. Intent resolution logs include candidate ranking summaries.
3. Mobile intent demo has explicit pass/fail objective criteria.

### Mid-term (CLI foundation)

1. Introduce a dedicated `uto-cli` crate or equivalent command entrypoint.
2. Implement initial `init`, `run`, and `report` command shells.
3. Move POC report schema toward reusable framework report schema.

### Longer-term (framework maturity)

1. Add plugin/adapters for CI reporting targets.
2. Add timeline and artifact linking (screenshots, page source, raw driver payload snippets).
3. Stabilize report schema with versioning guarantees.

## Consequences

**Positive:**

- UTO becomes practical for real test teams, not only engine contributors.
- Failure diagnosis is faster due to explicit, structured step-level visibility.
- Cross-platform parity improves because reporting expectations are shared.

**Negative:**

- Additional engineering scope on UX and telemetry, not only protocol features.
- Need to manage report schema compatibility over time.
- CLI ergonomics and documentation now become first-class maintenance areas.

## Validation Expectations

When implementing features under this ADR:

- keep reporting additions deterministic and CI-safe
- avoid coupling core success to optional host tools without graceful handling
- validate with `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`