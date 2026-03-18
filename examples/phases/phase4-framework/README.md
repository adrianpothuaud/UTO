# phase4-framework

Committed reference project for UTO Phase 4 framework maturity.

This project demonstrates the current framework state:

- `uto-runner` CLI option parsing (`--target`, `--json`, `--report-file`)
- `uto-reporter` structured `uto-report/v1` JSON output
- `uto-reporter` deterministic HTML report output from JSON source-of-truth
- `uto-logger` modern structured logs plus loader/spinner support
- web + mobile execution paths with graceful mobile fallback behavior

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

Mobile tests skip gracefully when required host tooling is unavailable.
