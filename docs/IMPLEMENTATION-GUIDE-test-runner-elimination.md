# Test Runner Elimination: Detailed Implementation Guide

## File Index & Change Map

### Core Files to Modify

#### 1. `uto-test/src/lib.rs` — Add public runner entry point

**Current State:**
```rust
pub mod managed_session;
pub mod start;
pub mod suite;
pub use suite::Suite;
pub use managed_session::ManagedSession;
pub use start::{startNewSession, startNewSessionWithArg};
```

**Changes Required:**
- Add new module: `pub mod dispatch;`
- Export new runner function: `pub use dispatch::run_project_tests;`
- Keep all existing exports (backward compatible)

---

#### 2. Create `uto-test/src/dispatch.rs` — New test execution orchestrator

**New File Purpose:**
- Discover test functions in compiled project
- Execute tests via Suite API
- Return exit code

**Pseudocode Structure:**
```rust
pub async fn run_project_tests(
    target_platform: &str,
    options: CliOptions,
) -> Result<i32, Box<dyn std::error::Error>> {
    // 1. Determine RunMode from target_platform
    // 2. Return exit code
}

// Internal helper
async fn discover_test_functions() -> Result<Vec<TestFn>> {
    // Use cargo metadata to find test files
    // OR: Use convention (all .rs files in tests/)
}

// Internal helper  
async fn load_compiled_tests() -> Result<TestRegistry> {
    // If compilation needed, return error with guidance
}
```

**Key Considerations:**
- MVP includes `#[uto_test]` support, with convention fallback for existing suites
- For now, assume tests are in `tests/` and follow naming convention where macro is absent
- Return structured `Result` for error handling in CLI

---

#### 3. `uto-cli/src/commands.rs` — Modify run command

**Current Implementation (lines ~50-89):**
```rust
pub mod run {
    pub fn run(args: &[String]) -> Result<(), String> {
        let parsed = crate::parsing::parse_run_args(args)?;
        let config = crate::config::load_project_config(&parsed.project)?;
        crate::config::validate_project_runner(&parsed.project)?;
        
        // ... construct options ...
        
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&parsed.project)
            .arg("run")
            .arg("--bin")
            .arg("uto_project_runner")
            // ...
        
        let status = cmd.status()?;
        // ...
    }
}
```

**Changes Required:**
1. Remove `crate::config::validate_project_runner()` call (this validates binary existence)
2. Change `cargo run --bin uto_project_runner` to:
   ```rust
   // Step 1: Compile library
   let build_cmd = Command::new("cargo")
       .current_dir(&parsed.project)
       .arg("build")
       .arg("--lib")
       .status()?;
   
   if !build_cmd.success() {
       return Err("Project compilation failed".to_string());
   }
   
   // Step 2: Call library function
   let exit_code = uto_test::run_project_tests(&effective_target, &options)
       .await?;
   
   if exit_code != 0 {
       return Err("Test suite failed".to_string());
   }
   ```

3. Handle async execution (may need `#[tokio::main]` in main.rs or refactor command structure)

---

#### 4. `uto-cli/src/templates.rs` — Stop generating runner binary

**Current Implementation (lines ~48-115):**
```rust
pub fn project_runner_rs() -> String {
    r##"use uto_runner::{CliOptions, RunMode};
    // ... full template ...
    "##.to_string()
}
```

**Change Required:**
- Update caller in `init::run()` to NOT generate runner file
- In `init::run()` around line 30-50, remove/comment out:
  ```rust
  fs::write(
      parsed.project_dir.join("src/bin/uto_project_runner.rs"),
      templates::project_runner_rs(),
  )?;
  ```

**Alternative (Backward Compat):**
- Keep template function but don't call it in new projects
- Or return empty string for MVP

---

#### 5. `uto-cli/src/templates.rs` — Update cargo_toml template

**Current State (lines ~116-135):**
```rust
pub fn cargo_toml(project_name: &str, uto_root: &Path) -> String {
    format!(
        "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[workspace]\n\n[dependencies]\n...",
        project_name,
        // ...
    )
}
```

**Change Required:**
- Remove `src/bin/uto_project_runner.rs` from any binary section
- Ensure `[lib]` section exists (should be implicit, but verify)
- Keep all dependencies (`uto-test`, `uto-runner` for backward compat, `tokio`, etc.)

---

#### 6. `uto-cli/src/config.rs` — Update validation

**Current Implementation:**
```rust
pub fn validate_project_runner(project_dir: &Path) -> Result<(), String> {
    // Checks for src/bin/uto_project_runner.rs existence
}
```

**Change Required:**
- Either: Remove this function and its caller
- Or: Make it a no-op for backward compatibility
- Update to validate `uto.json` and tests/ directory instead

---

#### 7. Remove generated runner files from example projects

**Files to Delete:**
- `examples/phases/phase4-framework/src/bin/uto_project_runner.rs`
- `examples/phases/ui-showcase/src/bin/uto_project_runner.rs`

**Files to Update:**
- `examples/phases/phase4-framework/Cargo.toml` — remove `bin` section
- `examples/phases/ui-showcase/Cargo.toml` — remove `bin` section

---

### Documentation Files to Update

#### 1. `GEMINI.md` — Sync architecture section

**Sections to Update:**
- "Framework-facing workflow components" section
  - Current mentions runner binary generation
  - Change to describe CLI-owned runner in uto-test
  
- "Architecture" section
  - Update uto-cli description: "orchestrates test discovery and execution"
  
- "Conventions" section
  - Add guidance: "Generated test projects contain only tests/; no runner binary needed"

**Example Edit:**
```markdown
## Current Workflow Components

- `uto-test/src` owns end-user test helpers AND test discovery/execution for CLI
- `uto-cli/src` owns project scaffolding and command orchestration (no subprocess for runners)
```

---

#### 2. `README.md` — Update CLI usage examples

**Sections to Update:**
- CLI Quick Start section (if exists)
- Generated project structure illustration
- `uto run` command examples

**Example:**
```markdown
### Running Tests

# Run all tests (web target is default)
uto run --project ./my-project

# Run mobile tests
uto run --project ./my-project --target mobile

# No runner binary needed — the CLI handles test discovery and execution
```

---

#### 3. `uto-site/content/` — Update CLI documentation

**Files to Check & Update:**
- CLI guide / Getting Started page
- "How to Run Tests" section
- Example project walkthroughs

---

#### 4. `docs/0011-uto-test-crate-and-clean-soc-guidelines.md` — Expand scope

**Changes:**
- Add section: "Test Execution Responsibility (Phase 4.5+)"
- Document that uto-test now owns test discovery and execution
- Update SoC table to reflect new responsibility

---

### Test & Validation

#### 1. Example Project Testing

```bash
# Phase 4 framework
cd examples/phases/phase4-framework
cargo test                          # Should still work
uto run --project . --target web    # Should execute tests
uto run --project . --target mobile # Should execute (or skip gracefully)

# UI Showcase  
cd examples/phases/ui-showcase
cargo test                          # Should still work
uto run --project . --target web    # Should work
```

#### 2. New Project Generation Testing

```bash
uto init ./test-project --uto-root /path/to/uto
cd test-project

# Verify no src/bin/ directory
ls -la src/      # Should be empty or not exist

# Verify tests/  exists
ls -la tests/    # Should have examples

# Verify it runs
uto run --project .
```

#### 3. Report Output Validation

```bash
# Run in old (before) environment
uto run --project examples/phases/phase4-framework \
  --report-json /tmp/before.json

# Run in new (after) environment
uto run --project examples/phases/phase4-framework \
  --report-json /tmp/after.json

# Compare JSONs (should be identical structure)
jq '.' /tmp/before.json > /tmp/before.pretty
jq '.' /tmp/after.json > /tmp/after.pretty
diff /tmp/before.pretty /tmp/after.pretty
```

---

## Dependency & Crate Interaction Changes

### Current Call Chain
```
uto-cli::{commands::run}
  └─ spawn cargo process
     └─ cargo run --bin uto_project_runner
        └─ (generated binary)
           └─ uto_runner::CliOptions::from_env()
           └─ uto_test::Suite::new()
           └─ uto_reporter (via Suite)
```

### Proposed Call Chain
```
uto-cli::{commands::run}
  ├─ spawn cargo build --lib
  └─ uto_test::run_project_tests(target, options)
     ├─ discover tests in tests/
     └─ execute via uto_test::Suite::new()
        └─ uto_reporter (unchanged)
```

### Crate Dependencies (No Changes)
```
uto-cli
  └─ uto-test (new function)
  └─ uto-core (unchanged)
  └─ uto-logger (unchanged)
  
uto-test (enhanced)
  └─ uto-core (unchanged)
  └─ uto-reporter (unchanged)
  └─ uto-logger (unchanged)
  └─ tokio (unchanged)
```

No new crate dependencies required.

---

## Error Handling & Migration strategy

### Error Cases to Handle

1. **Project not a UTO project** (no `uto.json`)
   - Provide clear error message
   
2. **No tests found in tests/ directory**
   - Warn but don't fail (allow empty projects)
   - Message: "No tests found in tests/ directory"

3. **Compilation fails**
   - Bubble up cargo build error
   - Message: "Failed to compile project. Check above output for details."

4. **Test discovery fails**
   - Provide list of what was looked for
   - Message: "Failed to discover tests. Ensure tests are in tests/ directory with conventional names."

### Backward Compatibility

**For projects with existing runner binary:**
1. `uto run` detects old-style project (checks for src/bin/uto_project_runner.rs)
2. Falls back to old behavior: `cargo run --bin uto_project_runner`
3. Log deprecation warning: "⚠️ This project uses legacy runner binary. Run `uto migrate` to update."
4. Keep this fallback for exactly two minor releases after Phase 4.5, then remove it.

**Implementation in uto-cli:**
```rust
if parsed.project.join("src/bin/uto_project_runner.rs").exists() {
    log::warn!("Legacy runner binary detected. Consider running: uto migrate");
    // ... fall back to old cargo run behavior ...
} else {
    // ... new direct execution via uto_test::run_project_tests ...
}
```

---

## Performance Considerations

### Build Caching
- `cargo build --lib` uses Rust's incremental compilation
- Subsequent runs should be fast (unchanged code = no recompilation)
- Avoid `--release` for test runs unless explicitly requested

### Discovery Overhead
- Scanning tests/ directory is one-time per invocation (~ < 10ms impact)
- Use `cargo metadata` for robust discovery (cached by cargo)
- Consider memoization for repeated `uto run` calls in watch mode

### Comparison to Current
| Aspect | Current | Proposed |
|--------|---------|----------|
| Per-project binary compile | Yes (every run) | Only if source changed |
| Binary existence check | ✅ (fast) | N/A |
| Test discovery | Implicit (hard-coded per binary) | Explicit, maybe slightly slower |
| Report generation | Same | Same |
| **Total time** | ~2-5 sec | ~1-3 sec (depending on changes) |

---

## Post-MVP Enhancements

### Phase 2: Deprecation Tooling
- `uto migrate` command to automatically remove old runner files
- `uto doctor` command to check project health

### Phase 3: Procedural Macros (Already in MVP, polish phase)
- Harden `#[uto_test]` diagnostics and developer ergonomics
- Expand examples and docs around explicit test marking
- Keep convention-based compatibility for existing suites during transition

### Phase 4: Watch Mode Integration
- `uto run --watch` for auto-rerun on file changes
- Reuse file watcher from UI mode (Phase 5)

---

## Code Review Checklist

- [ ] No public API changes to Suite or Session
- [ ] All existing tests pass
- [ ] Example projects compile and execute
- [ ] Report output (JSON/HTML) structure unchanged
- [ ] Backward compatibility maintained (old projects still work)
- [ ] Error messages are clear and actionable
- [ ] Documentation updated
- [ ] No new external dependencies
- [ ] Clippy & fmt pass: `cargo clippy --workspace --all-targets && cargo fmt --all --check`

