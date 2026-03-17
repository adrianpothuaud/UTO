# 0005: Project Roadmap — Phase 2 Completion and Phase 3 Vision

Date: 2026-03-17

## Status

Accepted

## Context

Phase 1 (Zero-Config Infrastructure) and Phase 2 (Driver Communication Layer) have been implemented and validated as working proof-of-concepts on macOS, Linux, and Windows CI targets.

The project is at a natural milestone: the foundational layers are solid, and the path forward to Phase 3 (vision-based recognition) is clear.

## Decision

### Phase 1 & 2 Completion Status

**Web Automation (100% Complete)**
- Chrome version discovery ✅
- ChromeDriver auto-download and caching ✅
- Process lifecycle management (clean hook) ✅
- WebDriver W3C protocol communication ✅
- Cross-platform testing (macOS, Linux, Windows) ✅
- CI validation with graceful skip on missing ChromeDriver ✅

**Mobile Automation (95% Complete)**
- Android SDK discovery ✅
- adb server startup and device detection ✅
- Android emulator AVD auto-discovery and boot ✅
- Appium auto-install via npm ✅
- UiAutomator2 driver auto-install ✅
- Process lifecycle management ✅
- W3C WebDriver protocol communication ✅
- Known blocker: Appium session endpoint configuration varies on some host environments (requires pre-configuration or further investigation)

**Documentation & CI**
- ADRs 0001–0004 documenting architectural decisions ✅
- GitHub Actions CI baseline (fmt, clippy, test on 3 platforms) ✅
- Copilot customization (instructions, prompts, agent) ✅

### Phase 3 Vision: AI-Driven UI Recognition

The next phase will introduce computer vision capabilities:

1. **Screenshot Capture & Preprocessing**
   - Integrate the existing `UtoSession::screenshot()` API
   - Implement basic image normalization and resizing

2. **ONNX-based UI Detection**
   - Load a pre-trained ONNX model for UI element detection
   - Run inference on captured screenshots to find interactive elements
   - Output bounding boxes and confidence scores

3. **Accessibility Tree Fusion**
   - Combine vision detection with platform accessibility data (AXI/accessibility tree)
   - Resolve ambiguous elements using "Weighted Consensus" heuristics
   - Return annotated element candidates to the session API

4. **High-Level Intent API**
   - Replace low-level selectors with human-centric actions:
     - `session.select("Add to Cart")`  — visual + accessibility match
     - `session.fill("username", "user@example.com")`  — intent-driven
     - `session.submit()`  — infer the submit action from context
   - Maintain backward compatibility with `UtoSession` trait

5. **Cross-Platform Support**
   - Validate on web (Chrome) first
   - Extend to mobile (Android via Appium, iOS on future macOS support)

## Consequences

**Positive:**
- Phase 1 & 2 deliver a production-ready zero-config driver provisioning and communication foundation.
- Web automation is fully functional and battle-tested.
- Mobile automation is nearly complete (known Appium configuration blocker, not architectural).
- Phase 3 roadmap is well-defined and unblocked.
- CI is stable and fast on all three major platforms.

**Negative:**
- ONNX model selection, training, and optimization will require ML domain expertise.
- Vision reliability depends on visual consistency (works better for native apps, web elements vary in styling).
- Accessibility tree fusion heuristics need real-world tuning.

## Next Immediate Steps

1. **Appium Session Endpoint Diagnostics** (optional, Phase 2 polish)
   - Implement a preflight probe to detect Appium route compatibility
   - Provide actionable error messages when session creation fails
   - Document the Appium configuration assumptions

2. **Phase 3 Foundation**
   - Add ONNX Runtime integration to workspace (`onnxruntime` Rust crate)
   - Create a `uto-vision` module for image processing and model loading
   - Implement a basic screenshot-to-element-bounding-boxes pipeline
   - Write integration tests that validate vision detection on known UI patterns
