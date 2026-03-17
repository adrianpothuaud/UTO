# Gemini Code Understanding (GEMINI.md)

## Project Overview

This project, "UTO" (Unified Testing Object), is a next-generation, cross-platform automation engine written in Rust. Its goal is to provide a "Vision-First, Human-Centric" alternative to traditional automation frameworks like Selenium and Appium.

The core philosophy is to interact with UIs (Web and Mobile) as a human would, by perceiving visual elements rather than relying solely on brittle DOM/Accessibility tree selectors.

### Key Technologies

*   **Core:** Rust
*   **Async Runtime:** `tokio`
*   **WebDriver:** `thirtyfour`
*   **Vision Engine (Future):** ONNX Runtime
*   **Communication (Future):** gRPC (`tonic`) and WebSockets (`tokio-tungstenite`)

### Architecture

The project is designed around four pillars:
1.  **Zero-Config Infrastructure (`uto-env`):** Automatically discovers browsers/SDKs and provisions the necessary drivers in isolated environments.
2.  **The Recognition Loop (`uto-vision`):** Uses ML to identify UI components visually, anchored by traditional accessibility data for resilience.
3.  **Human-Centric Interaction (`uto-api`):** Provides a high-level API focused on user intent (e.g., `select("Add to Cart")`).
4.  **The Hybrid Orchestrator (`uto-link`):** A high-performance Rust backbone for orchestrating complex, multi-device tests.

The current implementation is a Proof-of-Concept (POC) that demonstrates the Zero-Config Infrastructure pillar by discovering the local Chrome browser, automatically downloading the corresponding ChromeDriver, and launching a browser session.

## Current Status

The initial Proof-of-Concept (POC) for the `uto-env` pillar is complete and successful. The `main` branch contains a working implementation that can:

1.  **Discover:** Automatically find the installed version of Google Chrome on the host system.
2.  **Provision:** Download the matching version of ChromeDriver from the official Google repository.
3.  **Execute:** Launch the browser and create a controllable session using the `thirtyfour` WebDriver client.

This establishes the foundation for the UTO engine.

## Next Steps

With the `uto-env` foundation in place, the next focus is to develop the `uto-api` pillar. This involves creating a high-level, human-centric API that abstracts away the underlying WebDriver commands.

The immediate goal is to create a simple, chainable API that models user interactions. For example:

```rust
// (Future) Example of the target API
uto::run!(|session| {
    session
        .goto("https://example.com")
        .select("Username")
        .fill("my_user")
        .select("Password")
        .fill("my_pass")
        .click("Login")
});
```

This will require building a new `uto-api` crate and defining the core interaction traits and structs.

## Building and Running

This is a standard Rust project. The main application logic is in the `uto-core` crate.

### Build

To build the project, run the following command from the root directory:

```sh
cargo build
```

### Run

To run the main proof-of-concept application:

```sh
cargo run --package uto-core
```

This will execute the `main.rs` file, which discovers Chrome, provisions ChromeDriver, and opens a browser window to Google.com for 5 seconds.

### Test

To run any tests, use:

```sh
cargo test
```

## Development Conventions

*   **Package Management:** Dependencies are managed via `Cargo.toml`.
*   **Project Structure:** The project is a Cargo workspace, with the primary application logic located in the `uto-core` crate.
*   **Code Style:** Follow standard Rust conventions and formatting (`rustfmt`).
*   **Error Handling:** The project uses the `thiserror` crate for standardizing application errors.
*   **Linting:** Use `clippy` for identifying common mistakes and improving code quality: `cargo clippy`

## Documentation Habits

*   **`GEMINI.md`:** This file is the primary source of truth for understanding the project at a high level. Keep it updated as the architecture, build process, or core concepts evolve.
*   **Rustdoc:** All public functions, structs, and enums should be thoroughly documented using standard Rustdoc comments (`///`). This is crucial for generating useful library documentation.
*   **Design Documents:** For significant changes or new features, consider updating or adding to the design documents in the `/docs` directory. This includes the `manifesto.md` and architectural decision records.
*   **Commit Messages:** Write clear and concise commit messages that explain the "what" and "why" of a change.
