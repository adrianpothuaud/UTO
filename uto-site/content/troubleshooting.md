+++
title = "Troubleshooting"
description = "Resolve common UTO setup, run, and report issues across web and mobile environments."
template = "page"
slug = "troubleshooting"
+++

# Troubleshooting

## Web run fails to create browser session

- Ensure Chrome is installed and up to date.
- Run `uto run --project ./my-tests --target web` with verbose output to see provisioning logs.

## Mobile run skips or fails

- Verify `adb devices` shows an online device or emulator.
- Ensure Appium is installed and reachable from PATH.
- Ensure UiAutomator2 driver is installed in Appium.

UTO mobile tests skip gracefully when Appium or device tooling is unavailable.

## Report schema validation failure

Ensure the report JSON uses `schema_version: "uto-report/v1"` and regenerate reports via `uto run`.

## HTML report not generated

Run:

```sh
uto report --project <project-dir> --html
```

Optionally choose an output path with `--html-output`.

## Still stuck?

[Request early access](../early-access/) to get direct support from the UTO team.