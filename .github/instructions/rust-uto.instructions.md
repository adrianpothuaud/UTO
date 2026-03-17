---
description: "Use when editing Rust code in uto-core or poc. Covers zero-config design, cross-platform process management, WebDriver session layering, and required validation commands."
name: "UTO Rust Guidance"
applyTo: "uto-core/src/**/*.rs, uto-core/tests/**/*.rs, poc/src/**/*.rs"
---
# UTO Rust Guidance

- Keep the layering intact: `env` discovers or provisions dependencies, `driver` manages child processes, and `session` speaks W3C WebDriver.
- New automation features should fit the existing zero-config model: discover first, provision only when necessary, and keep version pinning explicit.
- Process-management code must preserve clean shutdown semantics across Unix and Windows.
- Prefer extending the shared session abstractions over adding one-off flows in the POC binaries.
- For new public APIs, add Rustdoc and keep error messages actionable.

Validate Rust changes with:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`