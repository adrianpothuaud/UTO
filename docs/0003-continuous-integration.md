# 0003: GitHub Actions Continuous Integration Baseline

## Status

Accepted

## Context

UTO is explicitly cross-platform and its current implementation spans two Rust
workspace crates: `uto-core` and `poc`.

The project needs automated checks that catch regressions in three areas:

- formatting drift
- lint regressions
- cross-platform build and test failures

At the same time, the current browser-backed integration tests are designed to
skip when `chromedriver` is unavailable, so CI should not depend on custom
driver provisioning just to validate the baseline repository.

## Decision

Add a GitHub Actions workflow that runs on every push and pull request with:

- a Linux quality job for `cargo fmt --all --check`
- a Linux quality job for `cargo clippy --workspace --all-targets -- -D warnings`
- a cross-platform test matrix for `cargo test --workspace` on Ubuntu, macOS,
  and Windows

Use the stable Rust toolchain and cache Cargo artifacts to keep feedback fast.

## Consequences

- Formatting and lint regressions fail early in pull requests.
- The repository is continuously validated on the three major desktop OSes.
- Browser integration coverage remains opportunistic in CI until a future
  workflow explicitly provisions Chrome and ChromeDriver.