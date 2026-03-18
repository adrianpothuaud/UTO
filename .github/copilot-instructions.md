# UTO Project Guidelines

## Core Philosophy

UTO is a Rust automation framework built around zero-config infrastructure and a unified web/mobile execution model.

Framework product direction:

- Build toward a first-class CLI lifecycle (`init`, `run`, `report`) for end users.
- Treat reporting and observability as core features, not optional add-ons.
- Ensure test execution visibility is clear from environment setup to intent resolution, element/action steps, assertions, and relevant driver communication outcomes.

Current delivery status:

- Phase 3 is complete.
- Phase 4.1 is in progress in `uto-cli` with strict command/config/report validation and growing CLI test coverage.
- Generated and reference test projects now consume `uto-test` and favor a simple helper-driven session API (`startNewSession`) so setup/session code remains transparent but concise.

- Prefer solutions that preserve the "discover or deploy" workflow described in `docs/0001-zero-config-infrastructure.md`.
- Never leave orphaned child processes behind. Driver lifecycle work must preserve the clean-hook model via process groups on Unix and Job Objects on Windows.
- Design for macOS, Linux, and Windows from the start. Do not hardcode single-platform assumptions into environment discovery, provisioning, or driver management.

## Architecture

- `docs/XXXX-*.md` contains ADRs that describe architectural decisions and project direction.
- `GEMINI.md`, `.github/agents/*`, `.github/instructions/*`, and `README.md` should all be kept in sync with the ADRs and project evolution.
- `uto-site/` contains the static site content for the project and must be kept in sync with project evolution, especially for CLI behavior changes.
- `uto-core/src/env` handles host discovery and provisioning.
- `uto-core/src/driver` owns WebDriver-compatible process startup, readiness checks, and shutdown.
- `uto-core/src/session` owns the W3C WebDriver communication layer and the shared `UtoSession` abstraction.
- `uto-test/src` owns end-user helper APIs for concise session lifecycle in authored tests.
- `uto-cli/src` owns framework-style command lifecycle workflows (`init`, `run`, `report`).
- `poc/src/bin` contains the executable proof-of-concept flows for Phase 1, Phase 2, and Phase 3.
- `examples/` contains CLI-generated validation flows and committed per-phase reference projects (`examples/phases/*`) that should remain runnable.

Read `GEMINI.md`, `docs/0001-zero-config-infrastructure.md`, and `docs/0002-driver-communication-layer.md` before making architectural changes.

## Build And Test

- Build the workspace with `cargo build --workspace`.
- Run the main validation set with `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- Use the `uto-poc` binaries for manual demos instead of inventing ad hoc entrypoints.
- Use the phase-related example project(s) under `examples/phases/` for iterative development and validation of new features

## Conventions

- Keep public Rust APIs documented with Rustdoc comments.
- Fix problems at the root instead of layering special cases around them.
- Favor clean, testable design: small functions, focused files, and explicit separation of concerns.
- Keep `GEMINI.md` and the relevant ADR in `docs/` in sync when architecture, workflow, or project direction changes.
- Preserve the current crate split and keep the `env`, `driver`, and `session` responsibilities clearly separated.
- Keep responsibilities separated across crates: `uto-core` (infrastructure/protocol), `uto-test` (end-user test helpers), and `uto-cli` (orchestration).
- Keep Readme.md and static site content in sync with project evolution
- Keep CLI behavior changes in sync across `README.md`, `uto-site/content/`, `GEMINI.md`, and ADRs.
- Prefer structured report output (JSON baseline) for new workflow surfaces so CI and diagnostics tooling can consume results reliably.
- For each new development phase, add or update one committed reference project under `examples/phases/` in addition to POC binaries.
