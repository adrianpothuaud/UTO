# UTO CLI Examples

This folder validates the framework-oriented CLI workflow (`init`, `run`, `report`) against generated sample projects.

## Quick Validation

```sh
./examples/validate-cli.sh
```

This script will:

1. generate a sample project with `uto init`
2. execute it with `uto run`
3. summarize results with `uto report`

Generated reports are JSON artifacts with a versioned schema (`schema_version`), run identity (`run_id`), timeline metadata, and step events.

By default it validates the web target. Set `WITH_MOBILE=1` to also validate mobile.
