# UTO Part 2: Live Debugging Without the Guesswork

Subtitle: A closer look at UTO UI mode and why run visibility is now a first-class testing feature.

Alternative subtitles by publication style:
- Better Programming style: How UTO UI turns failing test runs into live, inspectable execution timelines
- Founder-operator style: We were wasting hours on flaky debugging loops, so we built UTO UI mode
- QA leadership style: A practical path from red test results to faster root-cause analysis

A pipeline fails at 2:14 AM.
By 9:30 AM, three people are still asking the same question: what actually happened during the run?

Most testing tools still treat debugging as a side activity.
You run a suite, get a red result, then start stitching together logs, screenshots, and assumptions.

The failure is visible.
The reason is often not.

This is exactly why we built UTO UI mode.

UTO UI mode is a local browser interface for running, watching, and debugging suites in real time, designed to stay aligned with the same lifecycle as the CLI: init, run, report, and inspect.

Private beta is currently open for teams who want to shape this workflow early.

## The Practical Debugging Problem

In many pipelines, debugging friction comes from three gaps:

1. Context arrives too late.
You only get useful clues after the run is already over.

2. Signals are fragmented.
Terminal output, report artifacts, and test structure are spread across tools.

3. Reproduction loops are slow.
You edit a test, rerun, and repeat with little confidence about what changed.

UTO UI mode targets these gaps directly by turning execution into a live, inspectable stream instead of a postmortem artifact.

## What UTO UI Mode Does Today

UTO UI mode is available through a dedicated command:

```bash
uto ui --project ./my-tests
```

Core capabilities currently available:
- Real-time event stream while tests execute
- Run and stop controls from the browser UI
- Watch mode to re-run suites when test files change
- Report replay from existing artifacts
- Shared run visibility for multiple local viewers
- Status and report APIs used by the UI for stable state updates

This makes the debugging loop more direct:
- trigger a run,
- observe events as they happen,
- inspect the resulting report in the same surface,
- iterate quickly.

## Why This Matters for Teams

A stable test runner is important, but operational clarity is what reduces time-to-fix.

UI mode improves that clarity in practical ways:
- Engineers can see run progression without waiting for final logs.
- QA leads get clearer traceability from test intent to outcome.
- Teams can replay prior report artifacts without reproducing every run from scratch.

The result is fewer "cannot reproduce" moments and faster defect triage.

## Implementation Direction (What We Share Publicly)

At a high level, UI mode combines:
- an embedded local HTTP/WebSocket server,
- a browser-based interface for event and report visibility,
- subprocess orchestration for CLI-triggered runs,
- file watching for rapid re-execution during active development.

That public model is enough for teams evaluating architecture fit.

We intentionally keep deeper implementation details private during beta, including specific event buffering tradeoffs and internal optimization mechanics.

## Live Debugging Workflow Example

A common flow in private beta looks like this:

```bash
uto init ./my-tests --template web
uto run --project ./my-tests --target web
uto report --project ./my-tests --html
uto ui --project ./my-tests --watch
```

From there, a contributor edits tests, the watcher re-triggers runs, and the team sees run events and outcomes in one place.

It is not just about "running tests in a GUI."
It is about reducing the distance between authoring, execution, and diagnosis.

## What Comes Next

UI mode is complete at MVP scope and is now the bridge toward UTO Studio.

Next major direction:
- visual, selector-free test authoring across web and mobile,
- deeper debugging surfaces built on the same reporting-first foundation.

We are focused on keeping this progression coherent, so each step improves developer workflow without introducing a second system to maintain.

## Who Should Join This Beta Wave

This follow-up is especially relevant if your team:
- spends significant time investigating flaky or opaque UI test failures,
- wants a shared live-debugging surface instead of ad hoc terminal workflows,
- is willing to provide direct product feedback while the UX is still being shaped.

## Apply for UTO Private Beta

We are onboarding a limited number of QA and automation teams on a rolling basis.

Apply here:
https://github.com/adrianpothuaud/UTO/discussions/13

Please include:
1. Team profile and current automation workflow
2. Biggest debugging bottleneck in your current stack
3. Why live debugging visibility matters for your team

If your team values run transparency and faster diagnosis loops, this is the right phase to get involved.
