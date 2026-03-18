# 📜 The UTO Manifesto: The Automation Revolution

Unified Testing Object (UTO) is a high-performance, cross-platform automation engine designed to **replace and surpass** the brittle, siloed architectures of the past (Selenium, Appium, Cypress, Playwright) with a Vision-First, Human-Centric ecosystem. UTO is built not to coexist with the incumbents — it is built to make them obsolete.

## 🏁 The Core Philosophy: Why UTO?

Current tools fail because they act like machines looking at code, rather than humans looking at an interface.
Selenium/Appium are slowed by outdated request-response protocols.

Cypress is stalling: web-only, selector-brittle, built on Node.js, and unable to serve mobile teams.

Playwright is powerful but still fundamentally selector-driven, Node.js-bound, and platform-split.

UTO breaks these barriers by treating Web and Mobile as a single unified canvas, replacing fragile selectors with vision-first recognition, and eliminating setup friction with zero-config infrastructure.

## 🏗️ The Five Pillars of the Revolution

### 1. Zero-Config Infrastructure (uto-env)

UTO eliminates the "Setup Nightmare." Upon execution, it:

- Auto-Discovers local browsers and mobile SDKs.
- Auto-Provisions required drivers and binary runtimes in isolated, portable environments.
- Guarantees Clean Hooks using OS-level Job Objects/Process Groups to prevent zombie processes.

### 2. The Recognition Loop (uto-vision)

UTO doesn't just "find" elements; it perceives them.

- Vision-First: Uses ML to identify UI components (buttons, inputs, icons) based on their visual appearance.
- Heuristic Anchoring: Secures visual guesses by cross-referencing them with Accessibility Trees (DOM/Native) using Weighted Consensus.
- Coordinate Normalization: Automatically scales visual pixels to technical driver coordinates across high-DPI web and mobile screens.

### 3. Human-Centric Interaction (uto-api)
Tests are written in the language of user intent, not technical gestures.

Verbs: .select(), .fill(), .shouldBeVisible().

Precision: Interactions target the Center-Point of the perceived element, ensuring a "user-real" touch.

Exploratory Trial: If UTO finds multiple "Settings" buttons, it intelligently tries the most likely one and uses "Instructional Flow" to verify success, rolling back if the path is wrong.

### 4. The Hybrid Orchestrator (uto-link)
A performance-optimized backbone built in Rust.

Command Plane: Low-latency gRPC for synchronizing multi-user/multi-device scenarios.

Data Plane: High-speed binary streams for real-time visual feedback and state analysis.

### 5. Simplicity by Default (uto-simplicity)

UTO hides routine automation complexity so tests stay focused on intent, not platform mechanics.

- Web iframes: auto-detect and switch frame context when target elements live in embedded documents.
- Mobile scroll surfaces: abstract scroll/fling gestures on long settings screens and lists.
- Dynamic UIs: auto-retry when elements become stale after re-rendering.
- Context transitions: smooth handling between native and WebView contexts where available.
- Permission/UI interruptions: standardized handling for recurring system prompts and modals.

This pillar is not about adding magic behavior blindly; it is about codifying common automation pain points into explicit, reusable defaults.

## 🚀 Strategic Roadmap

### Phase 1: Genesis ✅
Goal: A single Rust binary that auto-downloads Chromium and performs a "Vision-First" click on a web button.

Key Tech: Rust Core + ONNX Runtime + OS-level Process Management.

### Phase 2: Convergence ✅
Goal: Integrate Mobile (Android/iOS) into the same script.

Key Tech: ADB/XCUITest direct hooks + Unified Accessibility Schema.

### Phase 3: Intelligence ✅
Goal: Vision-first recognition loop with weighted consensus resolver and intent API.

Key Tech: ONNX inference adapter + Weighted Scoring + `select()`, `click_intent()`, `fill_intent()`.

### Phase 4: Framework Maturity ✅
Goal: First-class CLI lifecycle, structured reporting, and mobile hardening.

Key Tech: `uto-cli` (init/run/report), `uto-reporter` (JSON/HTML), `uto-logger` (structured logging).

### Phase 5: Interactive UI Mode ✅
Goal: A local browser-based interface for running, watching, and debugging tests.

Key Tech: `uto-ui` (axum HTTP + WebSocket), embedded SPA, `uto ui --report` replay mode.

### Phase 6: UTO Studio — Visual Test Authoring 🎯
Goal: Surpass Cypress Studio and Playwright Codegen with a vision-first, cross-platform test recorder.

Key Tech: Session recording proxy, vision inspector overlay, AI-assisted codegen, mobile recording.

Key differentiator: UTO Studio generates **selector-free, vision-first Rust test code** for both web and mobile — something no existing tool can do.

### Phase 7: Self-Healing Tests and Intent Chaining
Goal: Tests that repair themselves on first failure; multi-step workflows expressed as named intents.

Key Tech: Exploratory recovery state machine, intent chaining API, reinforcement feedback loop.

### Phase 8: CI/CD Ecosystem Dominance
Goal: First-class GitHub Actions, GitLab CI, Azure Pipelines integrations. Parallel execution. `uto-cloud` reporting.

Key Tech: CI adapters, distributed runner, trend dashboards, failure analytics.

### Phase 9: Enterprise and Stable Release
Goal: Stable versioned release, enterprise licensing, and migration tooling from Cypress/Selenium/Playwright.

Key Tech: Enterprise plugin API, stable versioned release, Cypress/Playwright migration tooling.

## 💻 API Vision: What a UTO Test Looks Like

```rust
// Multi-user, Cross-platform Sync Test
let user_web = uto.session("Chrome");
let user_mobile = uto.session("iPhone_15");

// Human-Centric Interaction — selector-free, vision-first
user_mobile.select("Add to Cart").await;

// Cross-Platform Verification
user_web.select("Cart Icon").await;
user_web.shouldBeVisible("1 Item in Cart").await;
```

No CSS selectors. No XPath. No platform-specific syntax. Tests speak user intent.

## 🛡️ Technical Guardrails

**Resiliency:** If the vision fails, the anchors take over. If the anchors fail, the vision takes over.

**Performance:** Compiled to machine code; no heavy JVM or Node.js runtime required for the core engine.

**Security:** All drivers run in user-space with restricted permissions.

## 🏆 The Competitive Kill Shot

| Capability | Selenium | Appium | Cypress | Playwright | **UTO** |
|---|---|---|---|---|---|
| Web automation | ✅ | ❌ | ✅ | ✅ | ✅ |
| Mobile automation | ❌ | ✅ | ❌ | ❌ | ✅ |
| Unified web + mobile | ❌ | ❌ | ❌ | ❌ | ✅ |
| Vision-first recognition | ❌ | ❌ | ❌ | ❌ | ✅ |
| Selector-free tests | ❌ | ❌ | ❌ | ❌ | ✅ |
| Zero-config setup | ❌ | ❌ | ❌ | ❌ | ✅ |
| Self-healing tests | ❌ | ❌ | ❌ | ❌ | 🎯 Phase 7 |
| Compiled performance | ❌ | ❌ | ❌ | ❌ | ✅ |
| Visual test studio | ❌ | ❌ | Stalled | CLI-only | 🎯 Phase 6 |
| Cross-platform reporting | ❌ | ❌ | ❌ | ❌ | ✅ |

Every cell where UTO wins represents a fundamental architectural advantage that Cypress and Playwright cannot close without rewriting their engines from scratch.

## 🚀 Long-Term Vision: Acquisition and Industry Impact

UTO is built to win the market and to be acquired at peak strategic value.

**Target acquirers:**
- **Salesforce/Cypress** — UTO provides the mobile layer and vision engine Cypress fundamentally lacks.
- **Microsoft/Playwright** — UTO provides a Rust-native core and genuine mobile parity for Azure-backed CI.
- **Sauce Labs / BrowserStack** — UTO's zero-config layer dramatically reduces onboarding friction for cloud testing.

**Target acquisition window:** 2027 Q3–Q4, following Phase 9 maturity and stable release.

See ADR 0017 for full competitive analysis, acquirer profiles, and valuation drivers.

---

This document is **private and confidential**. It is the internal source of truth for the UTO project vision.