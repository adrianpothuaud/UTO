# UTO Medium Series: Complete Article Content

This document contains ready-to-publish drafts for:
- Part 1 (vision, architecture direction, and private beta call)
- Part 2 (UI mode and live debugging focus)

## Publication-Ready Variants

### Part 1 Headline Options

1. Why We Built a Unified Web + Mobile Test Engine in Rust
2. UI Automation Is Fragmented. We Built a Single Lifecycle Instead.
3. We Stopped Fighting Selectors and Rebuilt Automation Around Intent

### Part 1 Subtitle Options

- Better Programming style: A reporting-first Rust framework for teams done with brittle selectors and disconnected tooling
- Founder story style: What broke in modern test stacks, and the architecture we chose to fix it
- Engineering leadership style: One cross-platform workflow for web and mobile validation at scale

### Part 2 Headline Options

1. UTO Part 2: Live Debugging Without the Guesswork
2. Debugging UI Tests Should Not Feel Like Forensics
3. From Red Pipeline to Root Cause: Inside UTO UI Mode

### Part 2 Subtitle Options

- Better Programming style: How live run streaming and report replay shorten debugging cycles
- Founder-operator style: Why we treated test observability as product, not plugin glue
- QA leadership style: A practical interface for faster triage across test teams

---

## Part 1

# Why We Built a Unified Web + Mobile Test Engine in Rust

Subtitle: A reporting-first test automation approach for teams who are done stitching runners, selectors, and dashboards together.

A checkout flow fails only on Android.
The dashboard says red, the logs are noisy, and no one can quickly explain why.

Most UI test automation teams are not blocked by a lack of tools.
They are blocked by too many disconnected tools.

You might use one framework for web, another setup for mobile, one format for logs, another for reports, and a pile of fragile selectors tying everything together.

When a design system changes, tests break.
When a mobile flow shifts, parity breaks.
When a run fails in CI, debugging becomes archaeology.

That was the starting point behind UTO.

UTO (Unified Testing Object) is a cross-platform test automation framework designed around one execution model, one workflow, and one reporting surface for both web and mobile.

We are currently in private beta, and we are actively looking for serious testers from the Medium community who want to help shape it.

### The Problem We Decided to Solve

In most teams, the testing stack evolves by accumulation:
- one tool for browser automation,
- one path for mobile,
- one report plugin here,
- one custom script there.

Each choice is rational in the moment.
The result is often operational fragmentation.

Three patterns keep repeating:
1. Selector brittleness creates maintenance drag.
2. Web and mobile automation become separate philosophies.
3. Failure diagnostics are too shallow to be actionable quickly.

We did not want to add another layer on top of that stack.
We wanted to simplify the stack itself.

### What UTO Is Today

UTO is built in Rust and designed for macOS, Linux, and Windows from day one.

At a practical level, the current framework already supports a full lifecycle:
- uto init to scaffold projects
- uto run to execute suites
- uto report to produce structured diagnostics and HTML output
- uto ui to run, watch, and debug from a local browser interface

The central design choice is reporting-first execution visibility.
From environment setup to action steps and assertion outcomes, runs are intended to be inspectable rather than opaque.

That matters because modern teams need test output that both humans and CI systems can consume.

### Why Rust, and Why a Unified Model

Rust gives us strong control over process lifecycle and cross-platform behavior, which is essential for infrastructure-heavy automation.

We also needed explicit architecture boundaries so the product remains predictable as it grows:
- environment discovery and provisioning
- driver lifecycle
- session communication
- end-user test helpers
- CLI orchestration
- reporting and UI surfaces

Those boundaries are not just implementation details.
They are what let us keep one coherent workflow across web and mobile instead of branching into tool-specific behavior.

### The Zero-Config Principle

Our approach is simple:
- discover what exists on the host,
- provision what is missing,
- keep lifecycle cleanup explicit.

The goal is not to hide reality.
The goal is to reduce the amount of setup engineering users repeat in every project.

When onboarding is smoother, teams can spend more time validating product behavior and less time hand-maintaining automation plumbing.

### About Selector-Free Direction

Many teams ask this immediately: How do you avoid selector fragility without making tests vague?

Our direction is intent-oriented authoring with clear execution traces.
In practical terms, that means we optimize around user-visible behavior and resilient target resolution, while still exposing enough diagnostics to understand what happened during a run.

We share roadmap direction publicly and keep implementation-level mechanics private for now.
That is intentional.

If you are in the beta, we can go deeper with you in context, including tradeoffs and boundaries.

### What Exists Now vs What Comes Next

Completed foundation:
- zero-config infrastructure baseline
- unified web and mobile session model
- CLI lifecycle: init, run, report, ui
- structured JSON and HTML reporting
- interactive UI mode for live run, watch, and debug

Next major step:
- UTO Studio (Phase 6): visual, selector-free test authoring for web and mobile

The point is not more features.
The point is reducing the distance between author intent, execution behavior, and diagnosis.

### Who We Want in Private Beta

We are currently onboarding a limited group of:
- QA and SDET teams trying to reduce flaky maintenance cycles
- engineering leaders evaluating long-horizon test platform decisions
- advanced practitioners who want direct influence on workflow and roadmap

If you enjoy giving hard, practical product feedback, you are exactly who we want.

### What Beta Participants Can Expect

- Early access to private beta drops
- Direct collaboration channel with the UTO team
- Priority onboarding support
- Real opportunity to influence roadmap decisions before public release

We are optimizing for signal, not vanity signups.

### Apply for Beta

If this resonates, apply through GitHub Discussions:
https://github.com/adrianpothuaud/UTO/discussions/13

When you apply, include:
1. Your team profile and current automation setup
2. Your biggest quality engineering pain point
3. Why you want to join the private beta

We review applications on a rolling basis.

### Final Note

The test automation market has many capable tools.
What is still missing is a coherent, cross-platform, reporting-first model that teams can grow with.

That is the problem we are focused on solving with UTO.

If you want to help shape it early, we would be glad to have you in beta.

---

## Part 2

# UTO Part 2: Live Debugging Without the Guesswork

Subtitle: A closer look at UTO UI mode and why run visibility is now a first-class testing feature.

A pipeline fails at 2:14 AM.
By 9:30 AM, three people are still asking the same question: what actually happened during the run?

Most testing tools still treat debugging as a side activity.
You run a suite, get a red result, then start stitching together logs, screenshots, and assumptions.

The failure is visible.
The reason is often not.

This is exactly why we built UTO UI mode.

UTO UI mode is a local browser interface for running, watching, and debugging suites in real time, designed to stay aligned with the same lifecycle as the CLI: init, run, report, and inspect.

Private beta is currently open for teams who want to shape this workflow early.

### The Practical Debugging Problem

In many pipelines, debugging friction comes from three gaps:
1. Context arrives too late.
2. Signals are fragmented.
3. Reproduction loops are slow.

UTO UI mode targets these gaps directly by turning execution into a live, inspectable stream instead of a postmortem artifact.

### What UTO UI Mode Does Today

UTO UI mode is available through:

```bash
uto ui --project ./my-tests
```

Core capabilities currently available:
- real-time event stream while tests execute
- run and stop controls from the browser UI
- watch mode to re-run suites when test files change
- report replay from existing artifacts
- shared run visibility for multiple local viewers
- status and report APIs used by the UI for stable state updates

This makes the debugging loop more direct: run, observe, inspect, and iterate.

### Why This Matters for Teams

A stable test runner is important, but operational clarity is what reduces time-to-fix.

UI mode improves that clarity in practical ways:
- engineers can see run progression without waiting for final logs
- QA leads get traceability from test intent to outcome
- teams can replay prior report artifacts without reproducing every run from scratch

The result is fewer cannot-reproduce moments and faster defect triage.

### Implementation Direction We Share Publicly

At a high level, UI mode combines:
- an embedded local HTTP and WebSocket server
- a browser interface for event and report visibility
- subprocess orchestration for CLI-triggered runs
- file watching for rapid re-execution during active development

We intentionally keep deeper implementation details private during beta, including specific event buffering tradeoffs and internal optimization mechanics.

### Live Debugging Workflow Example

```bash
uto init ./my-tests --template web
uto run --project ./my-tests --target web
uto report --project ./my-tests --html
uto ui --project ./my-tests --watch
```

From there, a contributor edits tests, watch mode re-triggers runs, and the team sees run events and outcomes in one place.

It is not just about running tests in a GUI.
It is about reducing the distance between authoring, execution, and diagnosis.

### What Comes Next

UI mode is complete at MVP scope and is now the bridge toward UTO Studio.

Next major direction:
- visual, selector-free test authoring across web and mobile
- deeper debugging surfaces built on the same reporting-first foundation

We are focused on keeping this progression coherent, so each step improves developer workflow without introducing a second system to maintain.

### Apply for UTO Private Beta

We are onboarding a limited number of QA and automation teams on a rolling basis.

Apply here:
https://github.com/adrianpothuaud/UTO/discussions/13

Please include:
1. Team profile and current automation workflow
2. Biggest debugging bottleneck in your current stack
3. Why live debugging visibility matters for your team

If your team values run transparency and faster diagnosis loops, this is the right phase to get involved.
