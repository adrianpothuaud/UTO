# Rust Patterns Quick Reference

A quick-reference guide for common Rust patterns in the UTO codebase.

---

## Error Handling

### Propagating Errors with `?`

```rust
// Old way
match do_something() {
    Ok(val) => val,
    Err(e) => return Err(e),
}

// With ?
do_something()?
```

### Defining Custom Errors

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),  // Auto-converts with ?
}

pub type MyResult<T> = Result<T, MyError>;
```

### Matching Specific Errors

```rust
match result {
    Ok(value) => println!("Got: {}", value),
    Err(UtoError::BrowserNotFound(browser)) => {
        eprintln!("Install {browser} first");
    }
    Err(e) => return Err(e),
}
```

---

## Async Programming

### Basic Async Function

```rust
async fn fetch_data() -> Result<String, Error> {
    let response = http_client.get(url).await?;
    let text = response.text().await?;
    Ok(text)
}

// Call it
let data = fetch_data().await?;
```

### Async Trait

```rust
use async_trait::async_trait;

#[async_trait]
pub trait Repository: Send + Sync {
    async fn save(&self, item: &Item) -> Result<()>;
    async fn load(&self, id: &str) -> Result<Item>;
}

#[async_trait]
impl Repository for FileRepo {
    async fn save(&self, item: &Item) -> Result<()> {
        // Implementation
    }
    
    async fn load(&self, id: &str) -> Result<Item> {
        // Implementation
    }
}
```

### Spawning Async Tasks

```rust
use tokio::task;

// Run task in background
let handle = tokio::spawn(async move {
    do_work().await
});

// Wait for completion
let result = handle.await?;
```

---

## Ownership Patterns

### Taking Ownership

```rust
// Consumes self, can't use after
impl Session {
    pub async fn close(self) -> Result<()> {
        // cleanup
        Ok(())
    }
}

let session = Session::new();
session.close().await?;
// session is gone, can't use it anymore
```

### Borrowing

```rust
// Immutable borrow
fn read_title(&self) -> &str { &self.title }

// Mutable borrow
fn set_title(&mut self, title: String) { self.title = title; }

// Multiple immutable borrows OK
let a = obj.read_title();
let b = obj.read_title();  // OK

// But can't mix mutable and immutable
let a = &mut obj;
let b = &obj;  // ERROR: can't borrow as immutable
```

### Clone When Needed

```rust
// Clone for owned copies
let original = vec![1, 2, 3];
let copy = original.clone();

// Use Arc for shared ownership (no copy)
use std::sync::Arc;
let shared = Arc::new(data);
let reference = Arc::clone(&shared);
```

---

## Collections

### Vec (Dynamic Array)

```rust
// Create
let mut v = Vec::new();
let v = vec![1, 2, 3];

// Add/Remove
v.push(4);
v.pop();

// Iterate
for item in &v {
    println!("{}", item);
}

// Map/Filter
let doubled: Vec<_> = v.iter().map(|x| x * 2).collect();
let evens: Vec<_> = v.iter().filter(|x| x % 2 == 0).collect();
```

### HashMap

```rust
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("key", "value");

// Get
if let Some(val) = map.get("key") {
    println!("Found: {}", val);
}

// Iterate
for (k, v) in &map {
    println!("{}: {}", k, v);
}
```

---

## Option and Result

### Option<T>

```rust
// Creating
let some = Some(42);
let none: Option<i32> = None;

// Pattern matching
match some {
    Some(val) => println!("Got: {}", val),
    None => println!("Nothing"),
}

// Combinators
let val = some.unwrap_or(0);
let val = some.unwrap_or_else(|| compute_default());
let doubled = some.map(|x| x * 2);
```

### Converting Between Option and Result

```rust
// Option -> Result
let result = option.ok_or("missing value")?;
let result = option.ok_or_else(|| format!("missing: {}", key))?;

// Result -> Option
let option = result.ok();
```

---

## Trait Patterns

### Basic Trait

```rust
pub trait Renderable {
    fn render(&self) -> String;
}

impl Renderable for MyType {
    fn render(&self) -> String {
        format!("MyType({})", self.value)
    }
}
```

### Trait with Default Methods

```rust
pub trait Logger {
    fn log(&self, msg: &str);
    
    fn debug(&self, msg: &str) {
        self.log(&format!("DEBUG: {}", msg));
    }
    
    fn error(&self, msg: &str) {
        self.log(&format!("ERROR: {}", msg));
    }
}
```

### Trait Objects

```rust
// Store different types that implement same trait
let loggers: Vec<Box<dyn Logger>> = vec![
    Box::new(ConsoleLogger),
    Box::new(FileLogger::new("log.txt")),
];

for logger in loggers {
    logger.log("Hello");
}
```

### Trait Bounds

```rust
// Single bound
fn process<T: Display>(item: T) {
    println!("{}", item);
}

// Multiple bounds
fn process<T: Display + Clone>(item: T) {
    println!("{}", item.clone());
}

// Where clause (more readable)
fn process<T>(item: T)
where
    T: Display + Clone + Send,
{
    println!("{}", item);
}
```

---

## String Types

### String vs &str

```rust
// String: owned, heap-allocated, growable
let mut s = String::from("hello");
s.push_str(" world");

// &str: borrowed, immutable, fixed size
let slice: &str = "hello world";
let substring: &str = &s[0..5];

// Convert between them
let owned: String = slice.to_string();
let borrowed: &str = &owned;
```

### Common Operations

```rust
// Concatenation
let s = format!("{} {}", "hello", "world");
let s = ["hello", " ", "world"].concat();

// Comparison
if s == "hello world" { /* ... */ }

// Contains
if s.contains("hello") { /* ... */ }

// Split
for word in s.split_whitespace() {
    println!("{}", word);
}
```

---

## Lifetime Annotations

### Basic Lifetimes

```rust
// 'a says "the output lives as long as the input"
fn first_word<'a>(s: &'a str) -> &'a str {
    s.split_whitespace().next().unwrap_or("")
}

// Multiple lifetimes
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

### Lifetime Elision (Automatic)

```rust
// Compiler infers lifetimes in simple cases
fn first(&self) -> &str { &self.data }
// Equivalent to:
fn first<'a>(&'a self) -> &'a str { &self.data }
```

### 'static Lifetime

```rust
// Lives for entire program
let s: &'static str = "hardcoded string";

// Trait bound: no borrowed data (or all borrowed data is 'static)
fn spawn_task<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    std::thread::spawn(f);
}
```

---

## Concurrency Patterns

### Arc for Shared Ownership

```rust
use std::sync::Arc;

let data = Arc::new(vec![1, 2, 3]);

// Clone Arc (cheap, just increments ref count)
let data_clone = Arc::clone(&data);

// Use from multiple threads
tokio::spawn(async move {
    println!("{:?}", data_clone);
});
```

### Mutex for Interior Mutability

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));

let counter_clone = Arc::clone(&counter);
tokio::spawn(async move {
    let mut num = counter_clone.lock().unwrap();
    *num += 1;
});

// RwLock for multiple readers, single writer
use std::sync::RwLock;
let data = Arc::new(RwLock::new(vec![1, 2, 3]));

// Multiple readers OK
let read_guard = data.read().unwrap();
println!("{:?}", *read_guard);

// Exclusive writer
let mut write_guard = data.write().unwrap();
write_guard.push(4);
```

---

## Macros

### Common Macros

```rust
// Print with newline
println!("Hello, {}!", name);

// Print to stderr
eprintln!("Error: {}", error);

// Format string
let s = format!("x = {}, y = {}", x, y);

// Create vector
let v = vec![1, 2, 3];

// Debug print
dbg!(my_value);

// Panic (crash program)
panic!("Something went wrong: {}", reason);

// Unreachable code marker
unreachable!("This should never execute");

// Not yet implemented
todo!("Implement this later");
```

### Derive Macros

```rust
// Auto-implement common traits
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

let p = Point { x: 10, y: 20 };
println!("{:?}", p);           // Debug
let p2 = p.clone();            // Clone
assert_eq!(p, p2);             // PartialEq
let def = Point::default();    // Default (x: 0, y: 0)
```

---

## Pattern Matching

### Match Expression

```rust
match value {
    0 => println!("zero"),
    1 | 2 => println!("one or two"),
    3..=9 => println!("three through nine"),
    n if n < 0 => println!("negative: {}", n),
    _ => println!("something else"),
}
```

### If Let (Single Pattern)

```rust
// Instead of match with one pattern
if let Some(value) = optional {
    println!("Got: {}", value);
}

// With else
if let Some(value) = optional {
    println!("Some: {}", value);
} else {
    println!("None");
}
```

### While Let (Loop Until Pattern Fails)

```rust
let mut stack = vec![1, 2, 3];

while let Some(top) = stack.pop() {
    println!("{}", top);
}
```

### Destructuring

```rust
// Tuples
let (x, y, z) = (1, 2, 3);

// Structs
struct Point { x: i32, y: i32 }
let Point { x, y } = point;

// Enums
match result {
    Ok(value) => println!("Success: {}", value),
    Err(error) => println!("Error: {}", error),
}
```

---

## Module System

### Defining Modules

```rust
// In src/lib.rs or src/main.rs
pub mod session {
    pub mod web;
    pub mod mobile;
    
    pub use web::WebSession;
    pub use mobile::MobileSession;
}

// Re-export for convenience
pub use session::{WebSession, MobileSession};
```

### Importing

```rust
// Use specific items
use std::collections::HashMap;

// Use multiple items
use std::io::{Read, Write, Error};

// Use all public items (avoid in libraries)
use std::io::*;

// Rename imports
use std::io::Result as IoResult;

// Use from crate root
use crate::session::WebSession;

// Use from parent module
use super::utils::helper;
```

---

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
    
    #[test]
    fn test_error() {
        let result = fallible_function();
        assert!(result.is_err());
    }
    
    #[test]
    #[should_panic(expected = "divide by zero")]
    fn test_panic() {
        divide(10, 0);
    }
}
```

### Async Tests

```rust
#[tokio::test]
async fn async_test() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Doc Tests

```rust
/// Adds two numbers together.
///
/// # Example
///
/// ```
/// assert_eq!(add(2, 2), 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

---

## Common Idioms

### Builder Pattern

```rust
pub struct Request {
    url: String,
    method: String,
    headers: HashMap<String, String>,
}

impl Request {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            method: "GET".to_string(),
            headers: HashMap::new(),
        }
    }
    
    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = method.into();
        self
    }
    
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    
    pub async fn send(self) -> Result<Response> {
        // send request
    }
}

// Usage
let response = Request::new("https://example.com")
    .method("POST")
    .header("Content-Type", "application/json")
    .send()
    .await?;
```

### NewType Pattern

```rust
// Wrap primitive type for type safety
pub struct SessionId(String);
pub struct Port(u16);

impl Port {
    pub fn new(value: u16) -> Result<Self> {
        if value < 1024 {
            return Err("reserved port");
        }
        Ok(Port(value))
    }
    
    pub fn value(&self) -> u16 {
        self.0
    }
}
```

### Type State Pattern

```rust
// Encode state in the type system
struct Locked;
struct Unlocked;

struct Door<State> {
    state: PhantomData<State>,
}

impl Door<Locked> {
    fn unlock(self) -> Door<Unlocked> {
        println!("Unlocking door");
        Door { state: PhantomData }
    }
}

impl Door<Unlocked> {
    fn open(self) {
        println!("Opening door");
    }
}

// Compiler prevents opening locked door
let door = Door::<Locked>::default();
// door.open();  // ERROR: no method `open` on `Door<Locked>`
let door = door.unlock();
door.open();  // OK
```

---

## Quick Command Reference

```bash
# Build project
cargo build
cargo build --release

# Run tests
cargo test
cargo test --package uto-core
cargo test test_name

# Run specific binary
cargo run --bin uto-poc-phase1

# Check without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Generate documentation
cargo doc --open

# Update dependencies
cargo update

# Add dependency
cargo add tokio --features full
```

---

## Common Compiler Errors

### "Cannot move out of borrowed content"

```rust
// ERROR: Can't move out of &self
fn take_data(&self) -> Data {
    self.data  // ERROR
}

// FIX: Clone or return reference
fn take_data(&self) -> Data {
    self.data.clone()  // OK
}

fn get_data(&self) -> &Data {
    &self.data  // OK
}
```

### "Cannot borrow as mutable more than once"

```rust
// ERROR: Multiple mutable borrows
let a = &mut data;
let b = &mut data;  // ERROR

// FIX: Limit scope of borrowsa
{
    let a = &mut data;
    // use a
}
let b = &mut data;  // OK now
```

### "Does not live long enough"

```rust
// ERROR: Reference outlives data
fn broken() -> &str {
    let s = String::from("hello");
    &s  // ERROR: s is dropped here
}

// FIX: Return owned data
fn fixed() -> String {
    String::from("hello")
}
```

### "The trait bound is not satisfied"

```rust
// ERROR: T doesn't implement Display
fn print<T>(val: T) {
    println!("{}", val);  // ERROR
}

// FIX: Add trait bound
fn print<T: Display>(val: T) {
    println!("{}", val);  // OK
}
```

