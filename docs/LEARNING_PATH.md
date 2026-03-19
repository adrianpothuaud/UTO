# Learning Rust with UTO

Welcome! This guide provides a structured learning path for Rust beginners who want to contribute to the UTO project.

---

## 🎯 Learning Path

### Stage 1: Rust Fundamentals (1-2 weeks)

**Goal:** Understand core Rust concepts

**Resources:**

1. **Read [docs/rust-beginner-guide.md](rust-beginner-guide.md)**
   - Start here for UTO-specific Rust explanations
   - Covers ownership, async, traits, and project patterns
   
2. **Work through [The Rust Book](https://doc.rust-lang.org/book/)** (Chapters 1-11)
   - Chapter 1-3: Basic syntax, variables, functions
   - Chapter 4: Ownership (crucial!)
   - Chapter 5-6: Structs and enums
   - Chapter 7: Modules and packages
   - Chapter 8: Collections (Vec, HashMap, etc.)
   - Chapter 9: Error handling (Result, Option)
   - Chapter 10-11: Generics, traits, and testing

3. **Keep [docs/rust-patterns-cheatsheet.md](rust-patterns-cheatsheet.md) handy**
   - Quick reference for common patterns
   - Examples of error handling, async, collections, etc.

**Practice:**

```bash
# Set up Rust environment
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone UTO
git clone https://github.com/adrianpothuaud/UTO.git
cd UTO

# Build the project
cargo build --workspace

# Run tests
cargo test --workspace
```

**Success criteria:**
- ✅ Understand ownership and borrowing rules
- ✅ Can read and understand basic Rust code
- ✅ Know when to use `String` vs `&str`, `Vec` vs `&[T]`
- ✅ Comfortable with `Result<T, E>` and the `?` operator

---

### Stage 2: Async Rust (1 week)

**Goal:** Understand async/await and futures

**Resources:**

1. **Review async section in [docs/rust-beginner-guide.md](rust-beginner-guide.md#async-programming)**
   - Async traits
   - Pin and Box
   - Send + Sync bounds

2. **Read [Async Book](https://rust-lang.github.io/async-book/)** (Chapters 1-4)
   - Chapter 1: Why async?
   - Chapter 2: Futures and tasks
   - Chapter 3: async/await
   - Chapter 4: Pinning

3. **Explore [Tokio Tutorial](https://tokio.rs/tokio/tutorial)**
   - Hands-on async I/O examples
   - Task spawning and coordination

**Practice:**

```bash
# Run async examples in UTO
cargo run --bin uto-poc-phase1

# Explore async session code
# Read: uto-core/src/session/web.rs
# Read: uto-test/src/start.rs
```

**Success criteria:**
- ✅ Understand what `async fn` returns (a Future)
- ✅ Know when to use `.await`
- ✅ Understand `#[tokio::test]` vs `#[test]`
- ✅ Can write simple async functions

---

### Stage 3: UTO Architecture (1-2 weeks)

**Goal:** Understand UTO's structure and patterns

**Resources:**

1. **Read [docs/project-structure-guide.md](project-structure-guide.md)**
   - Crate responsibilities
   - Module organization
   - Common tasks

2. **Read key ADRs:**
   - [0001: Zero-config infrastructure](0001-zero-config-infrastructure.md)
   - [0002: Driver communication layer](0002-driver-communication-layer.md)
   - [0012: uto-test API usage guide](0012-uto-test-api-usage-guide.md)

3. **Explore the codebase:**
   - `uto-core/src/session/mod.rs` — UtoSession trait
   - `uto-core/src/driver/mod.rs` — Process management
   - `uto-test/src/suite.rs` — Test orchestration
   - `uto-core/src/error.rs` — Error types

**Practice:**

```bash
# Run example projects
cd examples/phases/phase4-framework
cargo test

# Read test files with educational comments
# Open: uto-core/tests/session_integration.rs
# Open: uto-test/src/suite.rs
```

**Success criteria:**
- ✅ Understand the crate structure
- ✅ Know where to find driver, session, and test code
- ✅ Can navigate the codebase confidently
- ✅ Understand how tests are organized

---

### Stage 4: Contributing (Ongoing)

**Goal:** Make meaningful contributions to UTO

**Resources:**

1. **Read [docs/testing-guide.md](testing-guide.md)**
   - Writing good tests
   - Test types and assertions
   - Debugging strategies

2. **Study commented code:**
   - `uto-test/src/suite.rs` — Complex generics and trait bounds
   - `uto-core/src/session/mod.rs` — Async traits and trait objects
   - `uto-core/src/driver/mod.rs` — Process groups and Drop trait

**First Contributions:**

1. **Documentation improvements:**
   - Fix typos
   - Add examples to doc comments
   - Clarify unclear explanations

2. **Test additions:**
   - Add tests for edge cases
   - Improve test coverage
   - Add educational comments to tests

3. **Small features:**
   - Add convenience methods to ManagedSession
   - Improve error messages
   - Add new CLI flags or options

**Finding issues to work on:**

```bash
# Look for good first issues on GitHub
# Tag: "good-first-issue"

# Areas suitable for beginners:
# - Documentation improvements
# - Test coverage
# - Error message clarity
# - CLI help text
```

**Success criteria:**
- ✅ Can write tests that follow UTO conventions
- ✅ Understand error handling patterns
- ✅ Can add simple features
- ✅ Ready to tackle more complex issues

---

## 📚 Reference Materials

### Quick References

- [Rust Patterns Cheatsheet](rust-patterns-cheatsheet.md) — Common Rust patterns
- [Project Structure Guide](project-structure-guide.md) — Where things live
- [Testing Guide](testing-guide.md) — How to write tests

### Deep Dives

- [Rust Beginner Guide](rust-beginner-guide.md) — Comprehensive Rust tutorial
- [UTO ADRs](.) — Architecture decision records

### External Resources

- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

---

## 🎓 Study Tips

### 1. Read Code Daily

Spend 15-30 minutes reading UTO code:

```bash
# Read a different module each day
# Monday: uto-core/src/session/mod.rs
# Tuesday: uto-test/src/managed_session.rs
# Wednesday: uto-core/src/driver/mod.rs
# etc.
```

### 2. Experiment in the REPL

Use `cargo run` or `cargo test` to test ideas:

```rust
// Create a scratch test file
// tests/scratch.rs

#[test]
fn experiment_with_option() {
    let x: Option<i32> = Some(5);
    let y = x.map(|n| n * 2);
    assert_eq!(y, Some(10));
}
```

### 3. Read Compiler Errors Carefully

Rust's compiler is your teacher:

```
error[E0382]: borrow of moved value: `session`
  --> src/main.rs:10:5
   |
8  |     let session = create_session();
   |         ------- move occurs because `session` has type `Session`
9  |     consume(session);
   |             ------- value moved here
10 |     consume(session);
   |             ^^^^^^^ value borrowed here after move
```

**What to learn:**
- Line 9 moved (consumed) `session`
- Line 10 tries to use it again (error!)
- **Lesson:** `consume()` takes ownership

### 4. Use Rust Analyzer

Install Rust Analyzer in VS Code for:
- Type hints
- Inline documentation
- Error highlighting
- Quick fixes

### 5. Ask Questions

When stuck:
1. Check the relevant guide in `docs/`
2. Search the UTO codebase for examples
3. Read the Rust Book section
4. Ask in GitHub Discussions

---

## 🛠️ Practice Projects

### Beginner: Add a Helper Method

**Task:** Add a convenience method to `ManagedSession`

```rust
// In uto-test/src/managed_session.rs

impl ManagedSession {
    /// Checks if an element with the given label exists.
    pub async fn has_element(&self, label: &str) -> bool {
        self.select(label).await.is_ok()
    }
}
```

**What you'll learn:**
- Reading existing code
- Following naming conventions
- Writing doc comments
- Adding tests

---

### Intermediate: Add Error Context

**Task:** Improve error messages with context

```rust
// In uto-core/src/session/web.rs

session
    .goto(&url)
    .await
    .map_err(|e| UtoError::SessionCommandFailed(
        format!("goto({url}) failed: {e}")
    ))?;
```

**What you'll learn:**
- Error handling patterns
- The `.map_err()` combinator
- Adding helpful context to errors

---

### Advanced: Add a New WebDriver Command

**Task:** Implement a new session method

1. Add to trait (`uto-core/src/session/mod.rs`)
2. Implement for WebSession (`uto-core/src/session/web.rs`)
3. Implement for MobileSession (`uto-core/src/session/mobile.rs`)
4. Add tests (`uto-core/tests/session_integration.rs`)
5. Add ManagedSession wrapper (`uto-test/src/managed_session.rs`)

**What you'll learn:**
- Trait design
- Cross-platform abstraction
- Test-driven development

---

## 🎉 Milestones

Track your progress:

- [ ] Built UTO from source
- [ ] Ran all tests successfully
- [ ] Read Rust Book chapters 1-11
- [ ] Understood ownership and borrowing
- [ ] Wrote first async function
- [ ] Read three ADRs
- [ ] Navigated codebase confidently
- [ ] Added first doc comment
- [ ] Fixed first typo/bug
- [ ] Wrote first test
- [ ] Added first feature
- [ ] Reviewed someone's PR
- [ ] Became a regular contributor

---

## 🤝 Community

- **GitHub Discussions:** Ask questions, share ideas
- **Issues:** Report bugs, suggest features
- **Pull Requests:** Contribute code
- **Code Reviews:** Learn from feedback

Remember: Everyone was a beginner once. Don't hesitate to ask questions!

---

## Next Steps

1. **Complete Stage 1:** Read [Rust Beginner Guide](rust-beginner-guide.md) and Rust Book chapters 1-11
2. **Build UTO:** Run `cargo build --workspace` and fix any issues
3. **Explore examples:** Run POC binaries and example projects
4. **Pick first issue:** Find a "good-first-issue" on GitHub
5. **Join community:** Introduce yourself in GitHub Discussions

Happy learning! 🦀
