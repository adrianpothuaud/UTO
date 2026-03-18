# UTO CLI Examples

This folder validates the framework-oriented CLI workflow (`init`, `run`, `report`) against generated sample projects.

## Phase Reference Habit

In addition to generated smoke projects, UTO keeps one **committed reference project per development phase** under `examples/phases/`.

These phase examples are durable references, similar to the `poc/src/bin` binaries:

1. they are versioned with the architecture changes for that phase
2. they remain runnable as implementation references
3. they are review anchors for expected behavior and reporting shape

Current phase references:

- `examples/phases/phase3-intent`: committed Phase 3 intent-resolution project
- `examples/phases/phase4-framework`: committed Phase 4 framework + reporting project

## Quick Validation

```sh
./examples/validate-cli.sh
```

This script will:

1. generate a sample project with `uto init`
2. execute it with `uto run`
3. summarize results with `uto report`
4. generate a native HTML artifact with `uto report --html`

Generated reports are JSON artifacts with a versioned schema (`schema_version`), run identity (`run_id`), timeline metadata, and step events.

For local debugging and CI artifact readability, the same report can be rendered as a single-file HTML output derived from the JSON source of truth.

By default it validates the web target. Set `WITH_MOBILE=1` to also validate mobile.
