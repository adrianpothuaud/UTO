# UTO Project Structure Guide

This document explains the organization of the UTO codebase, helping new contributors understand where to find code and where to add new features.

---

## Overview

UTO is organized as a **Cargo workspace** with multiple crates, each with a focused responsibility:

```
UTO/
├── uto-core/        # Core driver & session infrastructure
├── uto-test/        # Test authoring API for end users
├── uto-test-macros/ # Procedural macros for test metadata
├── uto-cli/         # CLI commands (init, run, report, ui)
├── uto-reporter/    # JSON/HTML report generation
├── uto-logger/      # Structured logging and progress output
├── uto-runner/      # Test runner infrastructure (deprecated)
├── uto-ui/          # Interactive UI mode server
├── uto-site/        # Static site generator for docs
├── poc/             # Proof-of-concept binaries for phases
├── examples/        # Reference projects and CLI-generated tests
└── docs/            # ADRs, guides, and design documents
```

---

## Crate Responsibilities

### 🔧 `uto-core` — Core Infrastructure

**Purpose:** Low-level driver management, WebDriver protocol, session abstractions, and environment discovery.

**Key modules:**

```
uto-core/src/
├── driver/          # Driver process lifecycle (ChromeDriver, Appium)
│   ├── mod.rs       # DriverProcess, start/stop, process groups
│   └── readiness.rs # Health checks and readiness polling
├── session/         # WebDriver communication layer
│   ├── mod.rs       # UtoSession trait (unified API)
│   ├── web.rs       # WebSession (Chrome/browser automation)
│   ├── mobile.rs    # MobileSession (Appium/mobile automation)
│   └── ...          # Mobile capabilities, accessibility helpers
├── env/             # Environment discovery and provisioning
│   ├── platform/    # Chrome/SDK version detection (OS-specific)
│   ├── provisioning/ # ChromeDriver download and caching
│   └── mobile_setup/ # Android SDK, Appium installation
├── vision/          # Intent resolution (OCR, accessibility trees)
│   ├── resolver.rs  # Vision-first element matching
│   └── latency.rs   # Performance tracking
├── error.rs         # Unified error types (UtoError)
└── lib.rs           # Public API exports
```

**When to edit:**
- Adding a new driver or platform
- Implementing WebDriver protocol features
- Improving environment discovery logic
- Extending the `UtoSession` trait

**Design principles:**
- Platform-agnostic abstractions
- Clean process lifecycle (no orphans)
- Explicit error handling with `UtoError`

---

### 🧪 `uto-test` — Test Authoring API

**Purpose:** High-level API for writing tests. Wraps `uto-core` with convenience methods and event recording.

**Key modules:**

```
uto-test/src/
├── lib.rs           # Re-exports and crate root
├── start.rs         # startNewSession() helper
├── managed_session.rs # ManagedSession (wraps UtoSession with events)
├── suite.rs         # Suite runner (WDIO-style test orchestration)
└── live_stream.rs   # Live event recording for UI mode
```

**Key types:**

- **`ManagedSession`**: Wraps `WebSession` or `MobileSession` with automatic event recording
- **`Suite`**: Declarative test runner that executes multiple tests in sequence
- **`startNewSession()`**: Convenience function for creating sessions with auto-discovery

**When to edit:**
- Adding new test authoring helpers
- Implementing new convenience methods
- Enhancing event recording for observability

**Usage example:**

```rust
use uto_test::{startNewSession, ManagedSession};

#[tokio::test]
async fn test_example() {
    let session = startNewSession("chrome").await.unwrap();
    session.goto("https://example.com").await.unwrap();
    let title = session.title().await.unwrap();
    assert!(!title.is_empty());
    session.close().await.unwrap();
}
```

---

### 🎨 `uto-test-macros` — Procedural Macros

**Purpose:** Provides the `#[uto_test]` attribute macro for annotating test functions with metadata.

**Key modules:**

```
uto-test-macros/src/
└── lib.rs           # Procedural macro implementation
```

**Features:**

- Extracts test metadata (target, tags, timeout)
- Used by `uto run` to discover and filter tests

**Example:**

```rust
use uto_test::uto_test;

#[uto_test(target = "web", tags = ["smoke", "critical"], timeout_ms = 30000)]
#[tokio::test]
async fn my_test() {
    // test body
}
```

**When to edit:**
- Adding new test metadata fields
- Changing discovery/filter behavior

---

### 💻 `uto-cli` — Command Line Interface

**Purpose:** Implements `uto init`, `uto run`, `uto report`, and `uto ui` commands.

**Key modules:**

```
uto-cli/src/
├── main.rs          # CLI entrypoint, argument parsing
├── commands/        # Command implementations
│   ├── init.rs      # Project scaffolding
│   ├── run.rs       # Test execution (cargo test wrapper)
│   ├── report.rs    # Report parsing and HTML generation
│   └── ui.rs        # Interactive UI mode launcher
├── templates.rs     # Code generation templates (Cargo.toml, tests)
└── project.rs       # Project path resolution, CWD inference
```

**When to edit:**
- Adding new CLI commands
- Changing project scaffolding templates
- Updating CLI help text or arguments

**CLI workflow:**

```bash
uto init ./my-tests --template web
uto run --project ./my-tests --target web
uto report --project ./my-tests --html
uto ui --project ./my-tests
```

---

### 📊 `uto-reporter` — Report Generation

**Purpose:** Structured report schemas (JSON) and HTML rendering.

**Key modules:**

```
uto-reporter/src/
├── lib.rs           # Re-exports
├── report.rs        # Report and ReportEvent types (uto-report/v1)
├── suite.rs         # SuiteReport type (uto-suite/v1)
└── render.rs        # HTML report generation (inline CSS/JS)
```

**Schemas:**

- **`uto-report/v1`**: Single test execution report
- **`uto-suite/v1`**: Multi-test suite execution report

**When to edit:**
- Changing report format or schema version
- Updating HTML report styling
- Adding new event types

---

### 📝 `uto-logger` — Structured Logging

**Purpose:** Centralized logging setup and progress output.

**Key modules:**

```
uto-logger/src/
├── lib.rs           # Logger initialization
└── progress.rs      # Progress indicators (spinners, bars)
```

**When to edit:**
- Changing log format or verbosity
- Adding new progress indicators

---

### 🎬 `uto-ui` — Interactive UI Mode

**Purpose:** Local web server for running, watching, and debugging tests.

**Key modules:**

```
uto-ui/src/
├── main.rs          # Server entrypoint
├── server.rs        # Axum web server, routes
├── watcher.rs       # File system watching (hot reload)
└── discovery.rs     # Test discovery without report files
```

**Features:**

- Real-time test execution streaming
- Watch mode (auto-rerun on file changes)
- Selective test execution
- Report replay with step-by-step inspection

**When to edit:**
- Adding new UI features or endpoints
- Improving test discovery logic
- Enhancing WebSocket streaming

**Access:**

```bash
uto ui --project ./my-tests
# Opens http://localhost:8080 in browser
```

---

### 🌐 `uto-site` — Static Site Generator

**Purpose:** Generates the UTO documentation website from Markdown.

**Key modules:**

```
uto-site/src/
├── main.rs          # Site build CLI
└── ...              # Tera templates, frontmatter parsing
```

**Content:**

```
uto-site/
├── content/         # Markdown pages
├── templates/       # HTML templates
└── static/          # CSS, images
```

**When to edit:**
- Updating documentation content
- Changing site styling

---

### 🚀 `poc/` — Proof of Concept Binaries

**Purpose:** Executable demonstrations of core functionality, used for iterative development.

**Structure:**

```
poc/src/bin/
├── phase1-chrome-click.rs        # Phase 1: Zero-config Chrome automation
├── phase2-mobile-click.rs        # Phase 2: Mobile (Android) support
├── phase3-intent-resolution.rs   # Phase 3: Vision-first recognition
└── ...
```

**When to edit:**
- Prototyping new features
- Validating architectural decisions
- Creating minimal reproducible examples

**Run:**

```bash
cargo run --bin uto-poc-phase1
cargo run --bin uto-poc-phase2
```

---

### 📚 `examples/` — Reference Projects

**Purpose:** Committed reference projects that demonstrate UTO usage patterns.

**Structure:**

```
examples/
├── phases/
│   ├── phase3-intent/    # Phase 3 reference (intent resolution)
│   ├── phase4-framework/ # Phase 4 reference (CLI + reporting)
│   └── ...
└── generated/
    └── web-smoke/        # CLI-generated validation project
```

**When to edit:**
- Adding new example projects for documentation
- Updating examples to match API changes

**Run:**

```bash
cd examples/phases/phase4-framework
cargo test
```

---

### 📖 `docs/` — Design Documents

**Purpose:** Architecture Decision Records (ADRs), guides, and planning documents.

**Key documents:**

```
docs/
├── 0001-zero-config-infrastructure.md  # Environment discovery
├── 0002-driver-communication-layer.md  # Session architecture
├── 0012-uto-test-api-usage-guide.md    # API patterns
├── 0014-ui-mode.md                     # Interactive UI design
├── rust-beginner-guide.md              # This guide's companion
└── rust-patterns-cheatsheet.md         # Quick reference
```

**When to edit:**
- Making architectural changes
- Adding new features
- Documenting design decisions

---

## Common Tasks

### Adding a New WebDriver Command

1. **Define in trait** (if it's platform-agnostic):
   - Edit `uto-core/src/session/mod.rs` → `UtoSession` trait
   
2. **Implement for WebSession**:
   - Edit `uto-core/src/session/web.rs`
   
3. **Implement for MobileSession**:
   - Edit `uto-core/src/session/mobile.rs`
   
4. **Add test helper** (optional):
   - Edit `uto-test/src/managed_session.rs` for convenience wrappers

### Adding a New CLI Command

1. **Create command module**:
   - Add `uto-cli/src/commands/mycommand.rs`
   
2. **Define CLI args**:
   - Edit `uto-cli/src/main.rs` → `Commands` enum
   
3. **Wire up handler**:
   - Edit `uto-cli/src/main.rs` → `match` on command

### Adding a New Error Type

1. **Define error variant**:
   - Edit `uto-core/src/error.rs` → `UtoError` enum
   - Add `#[error("...")]` attribute for the message
   
2. **Use it**:
   ```rust
   return Err(UtoError::MyNewError("details".into()));
   ```

### Adding Platform Support

1. **Environment discovery**:
   - Edit `uto-core/src/env/platform/` (OS-specific modules)
   
2. **Driver management** (if new driver needed):
   - Edit `uto-core/src/driver/mod.rs`
   
3. **Session type** (if needed):
   - Add new session type in `uto-core/src/session/`

---

## Testing Strategy

### Unit Tests

- **Location**: `#[cfg(test)] mod tests` within each module
- **Run**: `cargo test --package uto-core`
- **Coverage**: Core logic, error paths, edge cases

### Integration Tests

- **Location**: `uto-*/tests/` directories
- **Run**: `cargo test --workspace`
- **Coverage**: Cross-module workflows, CLI commands

### Example Projects

- **Location**: `examples/phases/*/`
- **Run**: `cd examples/phases/phaseX && cargo test`
- **Coverage**: End-to-end validation, API ergonomics

---

## Build Commands

```bash
# Full workspace build
cargo build --workspace

# Release build
cargo build --workspace --release

# Run all tests
cargo test --workspace

# Format code
cargo fmt --all

# Lint code
cargo clippy --workspace --all-targets -- -D warnings

# Check without building (fast)
cargo check --workspace

# Build docs
cargo doc --workspace --open

# Install locally
./install-local.sh  # macOS/Linux
.\install-local.ps1  # Windows
```

---

## Dependency Management

### Key Dependencies

- **`tokio`**: Async runtime
- **`async-trait`**: Async trait support
- **`thirtyfour`**: WebDriver client (Chrome, Appium)
- **`reqwest`**: HTTP client (provisioning)
- **`serde`**: JSON serialization
- **`thiserror`**: Error type derivation
- **`clap`**: CLI argument parsing
- **`axum`**: Web framework (UI mode)
- **`command_group`**: Process group management

### Adding a Dependency

```bash
# Add to specific crate
cargo add --package uto-core <crate-name>

# With features
cargo add --package uto-core tokio --features full
```

---

## Code Style Guidelines

### Formatting

- **Use `cargo fmt`** before committing
- **Line length**: 100 characters (Rustfmt default)
- **Imports**: Group by `std`, `external`, `crate`, then sort alphabetically

### Documentation

- **Public APIs**: Must have doc comments (`///`)
- **Modules**: Should have module-level docs (`//!`)
- **Examples**: Include usage examples in doc comments

### Error Handling

- **Avoid `.unwrap()`** in library code (use `?` or explicit `match`)
- **Contextual errors**: Use `.map_err()` to add context
- **Early returns**: Prefer `?` for clean error propagation

### Naming Conventions

- **Types**: `PascalCase`
- **Functions/variables**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Lifetimes**: `'a`, `'b`, etc. (short, descriptive if needed)

---

## Git Workflow

### Branch Strategy

- **`main`**: Stable, production-ready code
- **Feature branches**: `feature/my-feature`
- **Fix branches**: `fix/issue-123`

### Commit Messages

```
<type>: <short summary> (50 chars max)

<optional body with details>

<optional footer with issue refs>
```

**Types**: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

**Example:**

```
feat: Add mobile screenshot support to MobileSession

Implements the screenshot() method for MobileSession using
Appium's native screenshot endpoint.

Closes #42
```

---

## Resources

### Internal Documentation

- [Rust Beginner Guide](rust-beginner-guide.md) — Comprehensive Rust tutorial for UTO
- [Rust Patterns Cheatsheet](rust-patterns-cheatsheet.md) — Quick reference
- [README.md](../README.md) — Project overview
- [GEMINI.md](../GEMINI.md) — AI assistant context (architecture, conventions)

### External Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [WebDriver Spec](https://w3c.github.io/webdriver/)

---

## Getting Help

1. **Check the docs**: Start with `docs/` folder
2. **Read the code**: Well-documented modules explain complex patterns
3. **Run examples**: `examples/phases/` shows working usage
4. **Ask questions**: Open a GitHub Discussion or Issue

---

## Next Steps for New Contributors

1. **Set up environment**:
   ```bash
   git clone https://github.com/adrianpothuaud/UTO.git
   cd UTO
   cargo build --workspace
   cargo test --workspace
   ```

2. **Explore POC binaries**:
   ```bash
   cargo run --bin uto-poc-phase1
   cargo run --bin uto-poc-phase3
   ```

3. **Read key ADRs**:
   - `docs/0001-zero-config-infrastructure.md`
   - `docs/0002-driver-communication-layer.md`
   - `docs/0012-uto-test-api-usage-guide.md`

4. **Pick a good first issue**:
   - Look for `good-first-issue` label on GitHub
   - Start with documentation improvements
   - Add tests for uncovered code paths

5. **Make your first contribution**:
   - Fix a typo, improve a doc comment, add a test
   - Open a PR with a clear description
   - Respond to review feedback

Happy coding! 🦀
