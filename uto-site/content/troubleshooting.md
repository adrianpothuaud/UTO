+++
title = "Troubleshooting"
description = "Resolve common UTO setup, run, and report issues across web and mobile environments."
template = "page"
slug = "troubleshooting"
+++

# Troubleshooting

## `uto init` cannot resolve workspace crates

Use `--uto-root` with the repository root path (the directory that contains workspace `Cargo.toml`).

## Web run fails to create browser session

- ensure Chrome is installed
- validate discovery/provisioning with:

```sh
cargo run -p uto-poc --bin phase1_verify_or_deploy_drivers
```

## Mobile run skips or fails

- verify `adb devices` shows an online device or emulator
- ensure Appium is installed and reachable from PATH
- ensure UiAutomator2 driver is installed in Appium

UTO mobile tests are designed to skip gracefully when Appium/device tooling is unavailable.

## Report schema validation failure

Ensure report JSON uses `schema_version: "uto-report/v1"` and regenerate reports via `uto run`.

## HTML report not generated

Run:

```sh
cargo run -p uto-cli -- report --project <project-dir> --html
```

Optionally choose output path with `--html-output`.