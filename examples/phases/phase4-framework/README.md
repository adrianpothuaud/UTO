# phase4-framework

Committed reference project for UTO Phase 4 framework maturity.

This project demonstrates the current framework state:

- runnerless CLI-owned execution via `uto run`
- `uto-test::uto_test` annotations for target-aware test discovery
- `uto-reporter` structured `uto-suite/v1` JSON output plus native HTML suite reports
- multi-file scenario organization under `src/web/` and `src/mobile/`
- authored integration tests grouped by capability under `tests/`
- web + mobile execution paths with graceful mobile fallback behavior in authored tests

## Run

From repository root:

```sh
# Web run + JSON report
cargo run -p uto-cli -- run \
  --project examples/phases/phase4-framework \
  --target web \
  --report-json examples/phases/phase4-framework/.uto/reports/last-run.json

# Report summary + HTML artifact
cargo run -p uto-cli -- report \
  --project examples/phases/phase4-framework \
  --html

# Optional mobile run
cargo run -p uto-cli -- run \
  --project examples/phases/phase4-framework \
  --target mobile \
  --report-json examples/phases/phase4-framework/.uto/reports/last-run.mobile.json
```

## Tests

```sh
cd examples/phases/phase4-framework
cargo test
```

Current authored test layout:

- `tests/web_example.rs`: basic web smoke and content assertions
- `tests/web_intent_example.rs`: intent-driven web flows
- `tests/mobile_example.rs`: mobile helper flows with graceful skip behavior

Mobile tests skip gracefully when required host tooling is unavailable.
