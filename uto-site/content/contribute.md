+++
title = "Contribute to UTO"
description = "UTO is open-source and actively seeking contributors. Join the automation revolution."
template = "page"
slug = "contribute"
+++

# 🤝 Join the Automation Revolution

UTO is an open-source project and we are **actively looking for contributors** at every level —
from seasoned Rust engineers to curious newcomers who want to learn.

---

## Why Contribute?

- **Real-world impact:** Automation is used in thousands of software projects every day.
  UTO aims to replace decades-old tooling with something fundamentally better.
- **Cutting-edge tech:** Work with Rust, ONNX ML models, WebDriver, OS-level process management,
  gRPC and more — all in a single project.
- **Shape the architecture:** We are early.  Your contributions will influence the design of a
  framework used by developers globally.
- **Open and welcoming:** Every commit, review comment, bug report or documentation improvement
  counts.

---

## What We're Building

UTO is structured around four major work streams. Pick the one that excites you most:

### 🏗️ Zero-Config Infrastructure *(Mature)*

The foundation is working and battle-tested — Chrome/SDK discovery, driver provisioning, clean process hooks.
We need help with:

- **Firefox & Safari** driver support
- **iOS / Xcode** SDK discovery
- Improved error messages and recovery strategies
- Integration tests on Windows and macOS

### 👁️ The Recognition Loop *(Phase 3 MVP Complete; Hardening Phase)*

The heart of UTO's "vision-first" approach is now functional:

- **3.1 Foundation:** Deterministic preprocessing (resize/normalize) + postprocessing (NMS)
- **3.2 Resolver:** Weighted consensus (ML confidence + accessibility tree) with intelligent fallbacks
- **3.3 Latency:** Median/P95 SLA enforcement (≤50ms vision-only, ≤60ms with accessibility)
- **Intent API:** `select(label)`, `click_intent(label)`, `fill_intent(label, value)` on web and mobile

Next priorities:

- Real ONNX model integration (currently stub)
- Fixture library expansion and consensus weight tuning
- Performance profiling and p99/max latency trends
- Accessibility quality improvement for mobile

> **Good area for contribution:** real model sourcing, ground-truth fixture collection,
> latency trend analytics, or accessibility tree improvements on mobile.

### 🧑 Human-Centric API *(Design phase)*

- Design and implement the chainable, intent-based test API
- Explore natural-language test interpretation
- Build self-healing test trial loops

### ⚡ Hybrid Orchestrator *(Future)*

- gRPC command plane design with `tonic`
- Binary data streaming with `tokio-tungstenite`
- Multi-device synchronisation primitives

---

## How to Get Started

1. **Read the codebase** — start with [`uto-core/src/lib.rs`](https://github.com/adrianpothuaud/UTO/blob/main/uto-core/src/lib.rs)
   and the [project manifesto](https://github.com/adrianpothuaud/UTO/blob/main/docs/manifesto.md).

2. **Run the POC** — make sure Chrome is installed, then:

   ```sh
   git clone https://github.com/adrianpothuaud/UTO.git
   cd UTO
   cargo run -p uto-poc --bin phase1_verify_or_deploy_drivers
   ```

3. **Pick an issue** — browse [open issues](https://github.com/adrianpothuaud/UTO/issues)
   or open a new one describing what you'd like to work on.

4. **Submit a PR** — all pull requests are welcome.  Small and focused is better
   than large and sweeping.

---

## Contribution Guidelines

- Follow Rust formatting conventions (`cargo fmt`).
- Run `cargo clippy` and address all warnings before submitting.
- Add or update Rustdoc comments (`///`) for all public items.
- Write tests for new functionality — look at existing tests in `uto-core` as examples.
- Keep commit messages clear: *what* changed and *why*.

---

## Community & Contact

| Channel | Link |
|---------|------|
| GitHub Discussions | [github.com/adrianpothuaud/UTO/discussions](https://github.com/adrianpothuaud/UTO/discussions) |
| Issues & PRs | [github.com/adrianpothuaud/UTO](https://github.com/adrianpothuaud/UTO) |

We look forward to building with you. 🚀
