---
description: "Run or validate the UTO Chrome web demo using the current poc binaries and zero-config driver flow."
name: "Run Chrome Demo"
argument-hint: "Optional: include whether this is just a smoke test or a debugging session"
agent: "uto-architect"
tools: [read, search, execute]
---
Run or validate the UTO web demo using the current proof-of-concept binaries.

Use the existing entrypoints instead of editing temporary mains:

- `cargo run -p uto-poc --bin phase1_verify_or_deploy_drivers`
- `cargo run -p uto-poc --bin phase2_interact_with_session`

Check whether Chrome discovery, ChromeDriver provisioning, driver startup, and the web session flow behave as expected. If something fails, explain the failure in terms of the current `env`, `driver`, or `session` design.
