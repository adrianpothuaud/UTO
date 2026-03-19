# uto-reporter

**JSON and HTML report generation for the UTO automation framework.**

`uto-reporter` defines the report schema and provides serialization/deserialization for test execution reports. It supports both single-run and multi-test suite reports.

## Features

- **Structured JSON schema** — Well-defined report format for CI/CD integration
- **HTML generation** — Human-friendly HTML reports with detailed test results
- **Schema versioning** — Support for `uto-report/v1` (single run) and `uto-suite/v1` (suite)
- **Event tracking** — Detailed execution traces with timestamps and outcomes

## Report Schemas

### Single Run (`uto-report/v1`)

Contains results for a single test execution:

```json
{
  "schema_version": "uto-report/v1",
  "test_id": "test_login",
  "status": "passed",
  "duration_ms": 3542,
  "events": [...]
}
```

### Suite Run (`uto-suite/v1`)

Contains aggregated results for multiple tests:

```json
{
  "schema_version": "uto-suite/v1",
  "suite_id": "suite-1234567890",
  "mode": "web",
  "status": "passed",
  "summary": {
    "total": 10,
    "passed": 9,
    "failed": 1,
    "skipped": 0
  },
  "runs": [...]
}
```

## Usage

`uto-reporter` is used internally by `uto-cli` and `uto-test`, but can be used directly:

```rust
use uto_reporter::{UtoReportV1, ReportEvent};

let report = UtoReportV1 {
    schema_version: uto_reporter::UTO_REPORT_SCHEMA_V1.to_string(),
    test_id: "my_test".to_string(),
    status: "passed".to_string(),
    duration_ms: 1500,
    events: vec![
        ReportEvent {
            stage: "test.start".to_string(),
            status: "ok".to_string(),
            detail: serde_json::json!({}),
        },
    ],
};

// Serialize to JSON
let json = serde_json::to_string_pretty(&report).unwrap();
```

## Report Consumption

Reports can be consumed by:

- **CI/CD systems** — Parse JSON for pass/fail decisions
- **uto report** — Convert JSON to HTML
- **uto ui** — Live display in interactive mode
- **Custom tooling** — Build dashboards, metrics, etc.

## Related Crates

- **uto-cli** — Generates reports during `uto run`
- **uto-test** — Emits report events during test execution
- **uto-ui** — Displays reports in interactive UI mode

## License

MIT or Apache-2.0 (dual-licensed)
