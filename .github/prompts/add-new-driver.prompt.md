---
description: "Add support for a new browser or mobile driver in UTO. Use for SafariDriver, GeckoDriver, or other WebDriver-compatible additions that touch env, driver, session, tests, and docs."
name: "Add New Driver"
argument-hint: "Describe the driver to add and any platform constraints"
agent: "uto-architect"
tools: [read, search, edit, execute, todo]
---
Add support for a new WebDriver-compatible driver for UTO.

Requirements:

1. Implement discovery or provisioning in `uto-core/src/env` following the existing discover-or-deploy approach.
2. Extend driver lifecycle management in `uto-core/src/driver` without breaking clean-hook semantics.
3. Update the relevant session capabilities or session creation path in `uto-core/src/session`.
4. Add or update tests that cover the new behavior and skip gracefully when the host dependency is unavailable.
5. Update `GEMINI.md` and the relevant ADR in `docs/` if the architecture or workflow changes.

Use the current ChromeDriver and Appium implementations as the baseline pattern.
