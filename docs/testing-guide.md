# Testing Guide for UTO Contributors

This guide explains how to write, run, and understand tests in the UTO codebase.

---

## Test Organization

UTO uses Rust's built-in testing framework with several distinct test types:

### 1. Unit Tests

**Location:** Inside source files with `#[cfg(test)] mod tests`

**Purpose:** Test individual functions and implementation details

**Example:**

```rust
// In src/my_module.rs

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
        assert_eq!(add(-1, 1), 0);
    }
}
```

**Run:**

```bash
cargo test --package uto-core --lib
```

---

### 2. Integration Tests

**Location:** `tests/` directories within each crate

**Purpose:** Test public APIs from a user's perspective

**Example:**

```rust
// In uto-core/tests/session_integration.rs

use uto_core::session::{WebSession, UtoSession};

#[tokio::test]
async fn web_session_can_navigate() {
    let driver = start_chromedriver().await.unwrap();
    let session = WebSession::new(&driver.url).await.unwrap();
    
    session.goto("https://example.com").await.unwrap();
    let title = session.title().await.unwrap();
    
    assert!(!title.is_empty());
}
```

**Run:**

```bash
cargo test --test session_integration
cargo test --package uto-core --tests
```

---

### 3. Example Projects as Tests

**Location:** `examples/phases/*/tests/`

**Purpose:** Validate end-to-end workflows and API ergonomics

**Run:**

```bash
cd examples/phases/phase4-framework
cargo test
```

---

## Test Types in UTO

### Pure Unit Tests

**Characteristics:**
- No I/O, no network, no external dependencies
- Fast (milliseconds)
- Always reliable
- Test data transformation and logic

**Example:**

```rust
#[test]
fn mobile_capabilities_serializes_correctly() {
    let caps = MobileCapabilities::android("device-123");
    let json = caps.to_json_pub();
    assert_eq!(json["platformName"], "Android");
}
```

### Integration Tests with Skipping

**Characteristics:**
- Require external dependencies (Chrome, ChromeDriver, etc.)
- Skip gracefully when dependencies aren't available
- Use `Option` return types for conditional execution

**Example:**

```rust
#[tokio::test]
async fn web_session_navigation_or_skip() {
    // Returns None if ChromeDriver not found
    let Some(driver) = start_system_chromedriver().await else {
        println!("Skipping: ChromeDriver not available");
        return;
    };
    
    let session = WebSession::new(&driver.url).await.unwrap();
    session.goto("https://example.com").await.unwrap();
    // test continues...
}
```

### Network Tests (Ignored by Default)

**Characteristics:**
- Require internet connectivity
- Marked with `#[ignore]`
- Run explicitly with `--ignored` flag

**Example:**

```rust
#[tokio::test]
#[ignore]
async fn external_api_integration() {
    let response = reqwest::get("https://api.example.com/data")
        .await
        .unwrap();
    assert!(response.status().is_success());
}
```

**Run:**

```bash
cargo test -- --ignored
```

---

## Writing Good Tests

### 1. Test One Thing

❌ **Bad:** Test multiple unrelated behaviors

```rust
#[test]
fn everything_test() {
    test_addition();
    test_subtraction();
    test_multiplication();
    // Hard to debug when it fails!
}
```

✅ **Good:** Separate tests for separate behaviors

```rust
#[test]
fn test_addition() {
    assert_eq!(add(2, 2), 4);
}

#[test]
fn test_subtraction() {
    assert_eq!(subtract(5, 3), 2);
}
```

### 2. Use Descriptive Names

❌ **Bad:** Vague names

```rust
#[test]
fn test1() { /* ... */ }

#[test]
fn it_works() { /* ... */ }
```

✅ **Good:** Names describe what's being tested

```rust
#[test]
fn web_session_navigates_to_valid_url() { /* ... */ }

#[test]
fn driver_process_stops_cleanly() { /* ... */ }

#[test]
fn mobile_capabilities_includes_device_name() { /* ... */ }
```

### 3. Skip Gracefully

❌ **Bad:** Panic when dependencies are missing

```rust
#[tokio::test]
async fn test_web() {
    let driver = start_chromedriver().await.unwrap(); // Panics!
    // ...
}
```

✅ **Good:** Return early with clear message

```rust
#[tokio::test]
async fn test_web() {
    let Some(driver) = start_chromedriver().await else {
        println!("Skipping: ChromeDriver not found");
        return;
    };
    // ...
}
```

### 4. Test Error Cases

✅ **Good:** Verify error handling

```rust
#[test]
fn parse_invalid_json_returns_error() {
    let result = parse_json("invalid{json");
    assert!(result.is_err());
}

#[test]
fn missing_file_returns_not_found_error() {
    let result = read_file("/nonexistent/path");
    match result {
        Err(UtoError::IoError(_)) => { /* expected */ }
        _ => panic!("Expected IoError"),
    }
}
```

### 5. Use Fixtures and Helpers

✅ **Good:** Extract common setup into helpers

```rust
// In tests/support/mod.rs
pub fn sample_report() -> Report {
    Report::new(true, None, "web")
}

pub async fn start_test_session() -> Option<ManagedSession> {
    start_new_session("chrome").await.ok()
}

// In tests/my_test.rs
#[test]
fn report_has_correct_schema() {
    let report = sample_report();
    assert_eq!(report.schema_version(), "uto-report/v1");
}
```

---

## Test Assertions

### Common Assertions

```rust
// Equality
assert_eq!(actual, expected);
assert_ne!(actual, unexpected);

// Boolean conditions
assert!(condition);
assert!(!condition);

// Result/Option checking
assert!(result.is_ok());
assert!(result.is_err());
assert!(option.is_some());
assert!(option.is_none());

// String contains
assert!(text.contains("substring"));
assert!(text.starts_with("prefix"));
assert!(text.ends_with("suffix"));

// Collection membership
assert!(vec.contains(&item));
assert!(vec.is_empty());
assert_eq!(vec.len(), 3);
```

### Custom Assertions with Messages

```rust
assert!(
    result.is_ok(),
    "Expected Ok, got: {:?}",
    result
);

assert_eq!(
    session.target(),
    "web",
    "Session target should be 'web', got '{}'",
    session.target()
);
```

---

## Async Testing

### Basic Async Test

```rust
#[tokio::test]
async fn async_operation() {
    let result = fetch_data().await;
    assert!(result.is_ok());
}
```

### Testing Sequential Operations

```rust
#[tokio::test]
async fn multi_step_workflow() {
    let session = start_session().await.unwrap();
    
    session.goto("https://example.com").await.unwrap();
    let title = session.title().await.unwrap();
    assert!(!title.is_empty());
    
    session.close().await.unwrap();
}
```

### Testing Error Propagation

```rust
#[tokio::test]
async fn operation_fails_gracefully() -> Result<(), Box<dyn std::error::Error>> {
    let session = start_session().await?;
    session.goto("https://example.com").await?;
    session.close().await?;
    Ok(())
}
```

---

## Mocking and Test Doubles

### Using Test Fixtures

```rust
// Create a test-specific implementation
struct MockSession {
    responses: Vec<String>,
}

impl MockSession {
    fn new() -> Self {
        Self {
            responses: vec!["Title".into(), "Element text".into()],
        }
    }
}

#[async_trait::async_trait]
impl UtoSession for MockSession {
    async fn goto(&self, _url: &str) -> UtoResult<()> {
        Ok(())
    }
    
    async fn title(&self) -> UtoResult<String> {
        Ok(self.responses[0].clone())
    }
    
    // ... other methods
}

#[tokio::test]
async fn test_with_mock() {
    let session = MockSession::new();
    let title = session.title().await.unwrap();
    assert_eq!(title, "Title");
}
```

---

## Running Tests

### Run All Tests

```bash
cargo test --workspace
```

### Run Tests for Specific Crate

```bash
cargo test --package uto-core
cargo test --package uto-test
```

### Run Single Test

```bash
cargo test test_name
cargo test --test integration_test -- specific_test
```

### Run Tests with Output

```bash
# Show println! output
cargo test -- --nocapture

# Show output only for failed tests
cargo test -- --show-output
```

### Run Ignored Tests

```bash
cargo test -- --ignored
```

### Run Tests in Parallel/Sequential

```bash
# Parallel (default)
cargo test

# Sequential (when tests interfere with each other)
cargo test -- --test-threads=1
```

---

## Test Coverage

### Measuring Coverage (requires tarpaulin)

```bash
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage

# Open in browser
open coverage/index.html
```

### What to Test

✅ **High priority:**
- Public API functions
- Error handling paths
- Edge cases and boundary conditions
- Critical workflows (session creation, element finding, etc.)

⚠️ **Medium priority:**
- Internal helper functions
- Configuration parsing
- Output formatting

❌ **Low priority (or skip):**
- Generated code (macros)
- Simple getters/setters
- Obvious delegation methods

---

## Debugging Failing Tests

### 1. Read the Error Message

Rust test output shows:
- Which test failed
- The assertion that failed
- Expected vs actual values

```
---- web_session_navigates_to_url stdout ----
thread 'web_session_navigates_to_url' panicked at 'assertion failed: `(left == right)`
  left: `"Example Domain"`,
 right: `"Different Title"`'
```

### 2. Add Debug Output

```rust
#[tokio::test]
async fn debug_test() {
    let result = operation().await;
    eprintln!("Result: {:?}", result);  // Debug output
    assert!(result.is_ok());
}
```

Run with:

```bash
cargo test debug_test -- --nocapture
```

### 3. Use dbg! Macro

```rust
#[test]
fn debug_values() {
    let x = compute_value();
    dbg!(&x);  // Prints: [src/test.rs:123] &x = 42
    assert_eq!(x, 42);
}
```

### 4. Run Single Test with Logs

```bash
RUST_LOG=debug cargo test specific_test -- --nocapture
```

### 5. Use Rust Analyzer "Debug Test"

In VS Code with Rust Analyzer:
- Click "Debug" lens above test function
- Set breakpoints
- Step through code

---

## CI/CD Testing

UTO tests run in GitHub Actions CI. See `.github/workflows/ci.yml`.

**Key considerations:**

- Tests must pass on Linux, macOS, and Windows
- Network-dependent tests are skipped by default
- ChromeDriver/Appium may not be available in CI

**Writing CI-friendly tests:**

```rust
#[tokio::test]
async fn ci_compatible_test() {
    // Check if test environment is suitable
    let Some(driver) = start_system_chromedriver().await else {
        if std::env::var("CI").is_ok() {
            // In CI, skip instead of failing
            println!("Skipping in CI: ChromeDriver not available");
            return;
        } else {
            // Locally, fail to alert developer
            panic!("ChromeDriver required for this test");
        }
    };
    
    // Test continues...
}
```

---

## Best Practices Summary

1. ✅ Write tests for new features before implementation (TDD)
2. ✅ Test public APIs comprehensively
3. ✅ Make tests independent (no shared state)
4. ✅ Use descriptive test names
5. ✅ Skip gracefully when dependencies are unavailable
6. ✅ Test error cases, not just happy paths
7. ✅ Keep tests fast (use mocks for slow operations)
8. ✅ Document complex test setups
9. ✅ Run tests frequently during development
10. ✅ Don't commit failing tests (except as `#[ignore]` with TODO)

---

## Resources

- [Rust Book: Testing Chapter](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio: Testing](https://tokio.rs/tokio/topics/testing)
- [Rust By Example: Testing](https://doc.rust-lang.org/rust-by-example/testing.html)
- [Project Structure Guide](project-structure-guide.md)
- [Rust Patterns Cheatsheet](rust-patterns-cheatsheet.md)

