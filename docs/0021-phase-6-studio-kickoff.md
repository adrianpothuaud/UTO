# ADR 0021: Phase 6 Kickoff — UTO Studio Delivery

Date: 2026-03-20

## Status

Active — Phase 6.1 delivered

## Context

Phase 5.5 completed the library ergonomics and CLI flexibility work (ADR 0020). Phase 6 was
planned in ADR 0016 as **UTO Studio** — a visual, interactive test authoring environment that
eliminates hand-written selectors and reduces test creation to recording human interactions.

This ADR records the Phase 6 kickoff decisions and the Iteration 6.1 delivery.

## Iteration 6.1: Studio Scaffold and Recorder Protocol

### What Was Delivered

**CLI: `--studio` flag**

The `uto ui --studio` flag is now parsed and forwarded to the server layer.

```sh
uto ui --project ./my-suite --studio --open
```

The help text is updated to describe Studio mode:

```
Studio mode (Phase 6):
  uto ui --studio   Launch UTO Studio -- visual test authoring and recording
```

**Server: Studio routes**

Four REST endpoints are added to the UTO UI server when studio mode is active:

| Method | Path                  | Description                                              |
|--------|-----------------------|----------------------------------------------------------|
| GET    | `/api/studio/status`  | Returns recording state, step count, and test name       |
| POST   | `/api/studio/start`   | Starts a new recording session (resets prior steps)      |
| POST   | `/api/studio/stop`    | Stops the session and returns generated Rust code         |
| POST   | `/api/studio/step`    | Appends a recorded interaction step to the active session |

**`/api/status` extension**

The existing `/api/status` endpoint now includes a `"studio"` boolean field
so clients can detect whether Studio mode is active:

```json
{ "project": "my-suite", "status": "ready", "studio": true }
```

**`uto-ui/src/studio.rs` scaffold**

The new `studio` module contains:

- [`StudioState`] — shared mutable recording context (recording flag, steps, test name, start time).
- [`RecordedStep`] — a single captured interaction with `kind`, `target`, `value`, and `ts_ms`.
- [`StepKind`] — enum of supported interaction types: `Navigate`, `Click`, `Fill`, `AssertVisible`,
  `AssertText`, `AssertGone`, `Custom`.
- [`generate_test_code`] — renders a `Vec<RecordedStep>` into a complete, runnable Rust test
  function using the `uto-test` intent API.

**`generate_test_code` output example**

```rust
#[tokio::test]
async fn login_flow() -> uto_core::error::UtoResult<()> {
    let s = uto_test::startNewSession("chrome").await?;
    s.goto("https://example.com/login").await?;
    s.fill_intent("Username", "alice").await?;
    s.fill_intent("Password", "secret").await?;
    s.click_intent("Sign in").await?;
    s.assert_visible("Dashboard").await?;
    s.close().await
}
```

No CSS selectors. No XPath. No fragile DOM lookups.

**`examples/phases/phase6-studio/` reference project**

A new committed reference project validates:

- Studio code generation correctness for all `StepKind` variants
- `RecordedStep` JSON round-trip (REST API contract)
- `uto-suite/v1` schema compatibility with the UI server
- `uto.json` project config validity

**Public API exports**

`uto-ui` now exports `generate_test_code`, `RecordedStep`, `StepKind`, and `StudioState`
as part of its public API.

### Architecture

The studio module follows the same pattern as the existing `runner.rs` and `watcher.rs`
modules — it adds functionality without modifying any existing module behaviour.

```
uto-ui/src/
├── assets/index.html   (SPA — unchanged in 6.1)
├── lib.rs              (+ studio module + public re-exports)
├── runner.rs           (unchanged)
├── server.rs           (+ studio_mode/studio fields in AppState, + 4 routes, + studio field in UiOptions)
├── studio.rs           (NEW — Phase 6.1 scaffold)
└── watcher.rs          (unchanged)
```

The `AppState` struct gains two new fields:

```rust
studio_mode: bool,                              // whether --studio was passed
studio: Arc<RwLock<StudioState>>,               // shared recording context
```

### Maintained Layer Boundaries

- `uto-core` — unchanged; no session layer modifications
- `uto-test` — unchanged; generated code targets existing public API
- `uto-reporter` — unchanged
- `uto-cli` — only adds `--studio` flag to `UiArgs` and passes it to `UiOptions`
- `uto-ui` — gains one new public module; no existing API surface changed

## Iteration 6.2: Vision Inspector Overlay (Planned)

The next iteration will add:

- WebSocket event type `studio.element_hover` — emitted when the vision recognition
  loop identifies an element under the cursor during an active recording session.
- Bounding box overlay in the SPA for recognized elements.
- Confidence score display per element.
- Intent suggestion (click / fill / assert) based on element role.

This requires an active `WebSession` to be running server-side during recording, managed
by a new `studio_session.rs` module that bridges the server and `uto-core::session`.

## Iteration 6.3: Code Generation and Test File Output (Planned)

- `POST /api/studio/save` — writes generated Rust code to the project `tests/` directory.
- File naming convention: `tests/{test_name}.rs` (new file) or appended to an existing file.
- Compilation validation: `cargo check --tests` is run against the generated output before saving.

## Iteration 6.4: Assertion Builder and Replay Validation (Planned)

- Pause recording, click an element, and choose an assertion type from a visual picker.
- "Replay" button runs the generated test using the existing `handle_trigger_run` path.
- Replay result is shown inline in the Studio panel.

## Done Criteria for Phase 6 MVP

1. ✅ **6.1 scaffold** — Studio state, REST API, code generation, `--studio` flag, reference project
2. ⬜ **6.2 inspector** — Vision overlay, bounding boxes, confidence scores
3. ⬜ **6.3 file output** — Save generated tests to `tests/` with compilation validation
4. ⬜ **6.4 assertion builder** — Visual assertion builder and replay validation

## References

- ADR 0016: UTO Studio full specification
- ADR 0014: Phase 5 UI mode architecture (foundation for Studio)
- ADR 0019: Phase 5bis improvements (timeline, step icons, selective execution)
- ADR 0020: Phase 5.5 library ergonomics (library mode, CWD inference)
