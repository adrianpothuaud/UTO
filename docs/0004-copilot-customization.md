# 0004: Copilot Customization Baseline

## Status

Accepted

## Context

UTO already had a workspace `copilot-instructions.md`, but it duplicated project documentation and did not provide a layered customization model for:

- Rust implementation work inside `uto-core` and `poc`
- documentation and ADR maintenance
- reusable prompts that match the current POC binaries
- a project-specific custom agent for architecture-heavy work

The repository documentation in `GEMINI.md` and the accepted ADRs already defines the core architectural constraints, so Copilot customization should point back to those decisions rather than re-explaining the whole project in every file.

## Decision

Adopt a layered Copilot customization setup in `.github/`:

- keep `copilot-instructions.md` as a concise workspace-level baseline
- add `.github/instructions/rust-uto.instructions.md` for Rust code in `uto-core` and `poc`
- add `.github/instructions/docs.instructions.md` for `GEMINI.md`, ADRs, and related docs
- modernize the prompt files with frontmatter and current `uto-poc` entrypoints
- add a user-invocable `uto-architect` custom agent for architecture, driver, session, and documentation work

## Consequences

- Copilot guidance is more discoverable and better scoped to the task at hand.
- Prompt files now reflect the current workspace shape instead of obsolete `main.rs` workflows.
- Architecture-heavy changes can use a project-specific agent without replacing the default coding workflow for every task.