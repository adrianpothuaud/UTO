# phase3-intent

Committed example project for UTO Phase 3 (intent resolution).

This project is a stable reference, similar to the POC binaries in `poc/src/bin`, and demonstrates a minimal but complete implementation of the intent APIs:

- `select(label)`
- `fill_intent(label, value)`
- `click_intent(label)`
- JSON reporting with `uto-report/v1`

## Why This Example Exists

- Keep a runnable, versioned implementation trace for the phase.
- Provide a concrete review anchor for architecture and CI expectations.
- Avoid relying only on temporary scripts or PR snippets.

## Run

From the repository root:

```sh
# Web run + JSON report
cargo run -p uto-cli -- run \
  --project examples/phases/phase3-intent \
  --target web \
  --report-json examples/phases/phase3-intent/.uto/reports/last-run.json

# Report summary
cargo run -p uto-cli -- report --project examples/phases/phase3-intent

# Report summary + native HTML artifact
cargo run -p uto-cli -- report --project examples/phases/phase3-intent --html

# Optional mobile run
cargo run -p uto-cli -- run \
  --project examples/phases/phase3-intent \
  --target mobile \
  --report-json examples/phases/phase3-intent/.uto/reports/last-run.mobile.json
```

## Tests

```sh
cd examples/phases/phase3-intent
cargo test
```

Web/mobile tests skip gracefully when required host tooling is not available.

## Test Authoring Style

This reference now demonstrates the simplified UTO test style where setup and session lifecycle details are provided by the shared `uto-test` crate:

```rust
let web = uto_test::startNewSession("chrome").await?;
let mobile = uto_test::startNewSessionWithArg("android", 16).await?;
```

Rust-style equivalents are also supported:

```rust
let web = uto_test::start_new_session("chrome").await?;
let mobile = uto_test::start_new_session_with_hint("android", 16).await?;
```

The helper still logs environment discovery, driver startup, and session creation steps so setup remains observable even though the test code stays concise.

## Related Onboarding Docs

- `docs/0013-getting-started-and-troubleshooting.md` (Phase 4.4 onboarding baseline)
- `docs/0012-uto-test-api-usage-guide.md` (helper API surface)
