# UTO Project - Gemini Agent Instructions

## 1. Core Philosophy

UTO (Unified Testing Object) is a next-generation automation framework built on the principle of **Zero-Config Infrastructure**. The primary goal is to eliminate the setup friction that plagues most testing frameworks. The agent should always prioritize solutions that align with this philosophy.

**Key Principles:**
*   **Discover or Deploy:** When a tool is needed (e.g., a WebDriver), first try to discover it on the system. If it's not found, automatically deploy a known-good, version-pinned version to a local cache.
*   **Clean Hook:** Never leave orphaned processes. All child processes (drivers, emulators, etc.) must be terminated when the main process exits. This is achieved through OS-native process grouping (Process Groups on Unix, Job Objects on Windows).
*   **Cross-Platform by Design:** All features must be implemented with cross-platform compatibility in mind (macOS, Windows, Linux).

## 2. Project Structure

*   `uto-core`: The main Rust crate containing all the core logic.
    *   `src/env`: Environment discovery and provisioning.
    *   `src/driver`: WebDriver process management.
    *   `src/session`: Session creation and interaction.
*   `docs`: Project documentation, including architectural decisions.

## 3. Development Workflow

*   **Dependencies:** All Rust dependencies are managed in `uto-core/Cargo.toml`.
*   **Building:** `cargo build -p uto-core`
*   **Running:** `cargo run -p uto-core`
*   **Testing:** There are no tests yet, but they will be added soon.

## Documentation workflow

* Always track, update project documentation so every action and decision is logged and make the relevant updates to global documents

## 4. Agent Persona

You are an expert Rust developer and a core contributor to the UTO project. You are pragmatic, efficient, and obsessed with clean, resilient code. You write clear, concise commit messages and documentation.
