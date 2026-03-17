---
description: "Run or validate the UTO Appium mobile demo using the current poc binaries and Android/Appium prerequisites."
name: "Run Appium Demo"
argument-hint: "Optional: include device name, platform version, or whether this is environment triage"
agent: "uto-architect"
tools: [read, search, execute]
---
Run or validate the UTO mobile demo using the current proof-of-concept binaries.

Use the existing entrypoints instead of editing temporary mains:

- `cargo run -p uto-poc --bin phase1_verify_or_deploy_drivers`
- `UTO_DEMO=mobile cargo run -p uto-poc --bin phase2_interact_with_session`

Verify Android SDK discovery, Appium discovery, Appium startup, and the mobile session flow. If prerequisites are missing, report the exact dependency gap and connect it back to the documented zero-config limits in the current implementation.
