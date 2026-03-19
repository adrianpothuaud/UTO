# Rust for Beginners: UTO Project Guide

This guide explains Rust concepts and patterns used throughout the UTO codebase. It's designed for developers new to Rust who want to understand and contribute to the project.

## Table of Contents

1. [Core Rust Concepts](#core-rust-concepts)
2. [Async Programming](#async-programming)
3. [Error Handling](#error-handling)
4. [Trait System](#trait-system)
5. [Project-Specific Patterns](#project-specific-patterns)
6. [Common Questions](#common-questions)

---

## Core Rust Concepts

### Ownership and Borrowing

Rust's ownership system ensures memory safety without garbage collection:

```rust
// `session` owns the ManagedSession
let session = ManagedSession::new(/* ... */);

// `&self` borrows the session immutably
session.goto("https://example.com").await?;

// `self` (no &) takes ownership, consuming the value
session.close().await?;
// session cannot be used after this point!
```

**Key rules:**
- Each value has exactly one owner
- When the owner goes out of scope, the value is dropped (cleaned up)
- You can borrow values with `&` (immutable) or `&mut` (mutable)
- References must always be valid

### The `?` operator

The `?` operator is shorthand for error propagation:

```rust
// Without ?
let title = match session.title().await {
    Ok(t) => t,
    Err(e) => return Err(e),
};

// With ?
let title = session.title().await?;
```

`?` returns early with the error if the operation fails, otherwise unwraps the `Ok` value.

### Type Aliases

Type aliases make complex types more readable:

```rust
// Instead of writing this everywhere:
pub type UtoResult<T> = Result<T, UtoError>;

// You can use the alias:
async fn title(&self) -> UtoResult<String> { /* ... */ }
```

### `pub` and Module Visibility

- `pub` makes items public (visible outside the module)
- Without `pub`, items are private to the module
- `pub(crate)` makes items visible within the current crate only

```rust
pub struct WebSession { /* ... */ }     // Public
struct InternalState { /* ... */ }      // Private
pub(crate) enum ElementHandle { /* */ } // Crate-visible
```

---

## Async Programming

### The `async`/`await` Pattern

Async functions return `Future`s that must be `.await`ed to run:

```rust
// An async function
async fn fetch_title(&self) -> UtoResult<String> {
    // .await pauses this function until the operation completes
    let title = self.driver.title().await?;
    Ok(title)
}

// Calling it
let title = fetch_title().await?;
```

**Why async?** It allows UTO to handle I/O operations (network requests, waiting for elements) efficiently without blocking threads.

### `#[async_trait]`

Rust doesn't natively support async functions in traits, so we use the `async_trait` crate:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait UtoSession: Send + Sync {
    async fn goto(&self, url: &str) -> UtoResult<()>;
    async fn click(&self, element: &UtoElement) -> UtoResult<()>;
}
```

This macro transforms the async trait methods to work properly at runtime.

### `Send` and `Sync` Bounds

These are **marker traits** for thread safety:

- `Send`: The type can be moved between threads
- `Sync`: The type can be shared between threads (via `&T`)

Most UTO types are `Send + Sync` to work with async runtimes like Tokio:

```rust
pub trait UtoSession: Send + Sync {
    // Ensures UtoSession implementations are thread-safe
}
```

### `Pin<Box<dyn Future>>`

This complex type appears in the Suite API:

```rust
type PinFut = Pin<Box<dyn Future<Output = UtoResult<()>> + Send + 'static>>;
```

Let's break it down:

- `Future<Output = UtoResult<()>>`: An async operation that returns `UtoResult<()>`
- `dyn Future`: A **trait object** (dynamic dispatch, allows different Future types)
- `Box<dyn Future>`: Heap-allocated trait object
- `Pin<Box<...>>`: Guarantees the Future won't move in memory (required by async runtime)
- `+ Send`: The Future can be sent between threads
- `+ 'static`: The Future doesn't borrow any data with a limited lifetime

**Why so complex?** This allows the Suite to store test functions of different types in a single `Vec`.

---

## Error Handling

### `Result<T, E>` Type

All fallible operations return `Result`:

```rust
pub enum Result<T, E> {
    Ok(T),   // Success with value T
    Err(E),  // Failure with error E
}
```

### The `thiserror` Crate

UTO uses `thiserror` to define custom error types:

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtoError {
    #[error("Browser not found: {0}")]
    BrowserNotFound(String),
    
    #[error("Session command failed: {0}")]
    SessionCommandFailed(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}
```

**Key features:**
- `#[error("...")]` defines the error message
- `#[from]` automatically converts other error types with `?`
- `{0}` interpolates the wrapped value into the message

### Pattern Matching Errors

```rust
match session.goto(url).await {
    Ok(()) => println!("Navigation succeeded"),
    Err(UtoError::SessionCommandFailed(msg)) => {
        eprintln!("Session error: {msg}");
    }
    Err(e) => return Err(e),
}
```

---

## Trait System

### What is a Trait?

A trait defines shared behavior (like interfaces in other languages):

```rust
// Define the trait
pub trait UtoSession {
    async fn goto(&self, url: &str) -> UtoResult<()>;
}

// Implement it for WebSession
impl UtoSession for WebSession {
    async fn goto(&self, url: &str) -> UtoResult<()> {
        // implementation
    }
}
```

### Trait Objects (`Box<dyn Trait>`)

Use `Box<dyn Trait>` when you need to store or return different types that implement the same trait:

```rust
// This function can return either WebSession or MobileSession
pub fn create_session(target: &str) -> Box<dyn UtoSession> {
    match target {
        "web" => Box::new(WebSession::new(/* ... */)),
        "mobile" => Box::new(MobileSession::new(/* ... */)),
        _ => panic!("Unknown target"),
    }
}
```

**Trade-off:** `Box<dyn Trait>` uses dynamic dispatch (slower) but is more flexible than generics.

### Trait Bounds

Trait bounds constrain generic types:

```rust
// F must be a function that takes ManagedSession and returns a Future
pub fn test<F, Fut>(self, name: &str, f: F) -> Self
where
    F: FnOnce(ManagedSession) -> Fut + Send + 'static,
    Fut: Future<Output = UtoResult<()>> + Send + 'static,
{
    // ...
}
```

**Reading trait bounds:**
- `F: FnOnce(...)`: F is a function that can be called once
- `+ Send`: F can be sent to another thread
- `+ 'static`: F doesn't borrow short-lived data

### Default Trait Methods

Traits can provide default implementations:

```rust
#[async_trait]
pub trait UtoSession: Send + Sync {
    // Required: implementors must define this
    async fn select(&self, label: &str) -> UtoResult<UtoElement>;
    async fn click(&self, element: &UtoElement) -> UtoResult<()>;
    
    // Default implementation: uses `select` and `click`
    async fn click_intent(&self, label: &str) -> UtoResult<()> {
        let element = self.select(label).await?;
        self.click(&element).await
    }
}
```

---

## Project-Specific Patterns

### `Drop` Trait for Cleanup

The `Drop` trait ensures cleanup even if code panics:

```rust
impl Drop for DriverProcess {
    fn drop(&mut self) {
        // Kill the driver process when DriverProcess is dropped
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
```

This is similar to destructors in C++ or `finally` blocks in Java.

### Process Groups with `command_group`

UTO uses process groups to ensure child processes (like ChromeDriver) are killed when the parent exits:

```rust
use command_group::{CommandGroup, GroupChild};

let child = std::process::Command::new(binary_path)
    .arg(&port_arg)
    .group_spawn()  // Creates a process group
    .map_err(|e| UtoError::DriverStartFailed(format!("{e}")))?;
```

Without process groups, orphaned driver processes could remain running after tests finish.

### Interior Mutability with `Arc<Mutex<T>>`

Sometimes you need to mutate data shared across async tasks:

```rust
use std::sync::{Arc, Mutex};

// Arc = Atomic Reference Counted (shared ownership)
// Mutex = Mutual exclusion (thread-safe interior mutability)
let events: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(Vec::new()));

// Clone the Arc to share ownership
let events_clone = events.clone();

// Lock and modify
events_clone.lock().unwrap().push(event);
```

**Key concepts:**
- `Arc<T>`: Multiple owners, reference counted
- `Mutex<T>`: Allows mutable access through immutable reference
- `.lock().unwrap()`: Acquires the lock, panics if poisoned

### `Option<T>` for Nullable Values

Rust doesn't have null. Use `Option<T>` instead:

```rust
pub enum Option<T> {
    Some(T),
    None,
}

// Match on Option
match self.inner {
    Some(session) => session.goto(url).await?,
    None => return Err(UtoError::SessionCommandFailed("session closed".into())),
}

// Or use combinators
self.inner.as_ref()
    .ok_or_else(|| UtoError::SessionCommandFailed("session closed".into()))?
```

---

## Common Questions

### Why is `self` sometimes `Self`, `&self`, `&mut self`, or `Box<Self>`?

- `&self`: Immutable borrow (most common)
- `&mut self`: Mutable borrow
- `self`: Takes ownership (consumes the value)
- `Box<Self>`: Takes ownership of a boxed value
- `Self`: Refers to the type itself (e.g., in return types)

```rust
impl WebSession {
    pub fn new() -> Self { /* ... */ }           // Returns WebSession
    pub async fn goto(&self, url: &str) { /* */ } // Borrows
    pub async fn close(self) { /* ... */ }        // Consumes
}
```

### What are lifetimes (`'a`)?

Lifetimes tell Rust how long references are valid:

```rust
// 'a is a lifetime parameter
fn first_word<'a>(s: &'a str) -> &'a str {
    // The returned reference lives as long as the input reference
    s.split_whitespace().next().unwrap_or("")
}
```

**Good news:** UTO rarely needs explicit lifetimes because most operations use owned types or `'static` bounds.

### What does `#[derive(...)]` do?

`#[derive]` automatically implements common traits:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct UtoElement {
    pub label: String,
    pub selector: String,
    // ...
}
```

- `Debug`: Enables `println!("{:?}", element)`
- `Clone`: Enables `element.clone()`
- `PartialEq`: Enables `element1 == element2`

### What's the difference between `String` and `&str`?

- `String`: Owned, heap-allocated, mutable, can grow
- `&str`: Borrowed, immutable reference to string data

```rust
let owned: String = "hello".to_string();
let borrowed: &str = "world";
let borrowed_from_owned: &str = &owned;
```

**Rule of thumb:** Use `&str` for function parameters, `String` for owned data.

### What are macros?

Macros generate code at compile time:

```rust
// Macro invocation ends with !
println!("Hello, {}!", name);
vec![1, 2, 3]
#[tokio::test]  // Procedural macro (attribute)
```

UTO uses several macros:
- `println!`, `format!`: String formatting
- `vec!`: Vector creation
- `#[tokio::test]`: Marks async test functions
- `#[uto_test]`: Custom macro for UTO test metadata

---

## Learning Resources

### Official Rust Resources

- [The Rust Book](https://doc.rust-lang.org/book/) — comprehensive introduction
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — learn by doing
- [Async Book](https://rust-lang.github.io/async-book/) — deep dive into async Rust

### Crates Used in UTO

- [`tokio`](https://tokio.rs/) — async runtime
- [`async-trait`](https://docs.rs/async-trait/) — async trait support
- [`thiserror`](https://docs.rs/thiserror/) — error type derivation
- [`reqwest`](https://docs.rs/reqwest/) — HTTP client
- [`serde`](https://serde.rs/) — JSON serialization
- [`thirtyfour`](https://docs.rs/thirtyfour/) — WebDriver client

### UTO-Specific Documentation

- [README.md](../README.md) — Project overview
- [docs/0001-zero-config-infrastructure.md](0001-zero-config-infrastructure.md) — Environment discovery
- [docs/0002-driver-communication-layer.md](0002-driver-communication-layer.md) — Session architecture
- [docs/0012-uto-test-api-usage-guide.md](0012-uto-test-api-usage-guide.md) — API patterns

---

## Next Steps

1. **Read [The Rust Book](https://doc.rust-lang.org/book/)** chapters 1-11 for core Rust concepts
2. **Explore [examples/phases/](../examples/phases/)** for runnable UTO code
3. **Run the tests**: `cargo test --workspace` to see Rust in action
4. **Ask questions** via Issues or Discussions when something is unclear

