# 0007: Add Simplicity by Default as a Core Pillar

## Status

Accepted

## Context

UTO already defines strong foundations for discovery/provisioning and protocol communication, but real automation flows still suffer from repeated, low-value mechanics that distract from test intent.

Examples observed repeatedly:

- web flows that require manual iframe context switching before every action
- mobile flows that require hand-crafted scroll/fling loops to reach off-screen elements
- brittle retries around stale/re-rendered elements and transient UI interruptions

These concerns are cross-platform, high-frequency, and mostly orthogonal to business assertions.

## Decision

Promote **Simplicity by Default** to an explicit project pillar.

This pillar defines a direction: encapsulate recurring automation mechanics behind predictable, reusable primitives so user-facing test logic stays intent-centric.

## Scope Examples

Initial examples this pillar should cover:

- **Web iframe handling:** detect and switch into relevant frame contexts before element interaction, then restore parent context safely.
- **Mobile scroll/fling handling:** provide consistent helpers for scrollable screens and long lists (including fling-style navigation) on Appium-backed sessions.
- **Stale element recovery:** standardized retry strategy when UI re-renders invalidate references.
- **Context transitions:** explicit utilities for native/WebView context boundaries where supported.
- **Common interruptions:** reusable handling patterns for frequent modal/permission prompts.

## Consequences

- Test scripts become shorter and more declarative.
- Cross-platform behavior becomes more consistent because these mechanics are implemented once in core layers.
- Documentation and site messaging now include this pillar to align architecture and user expectations.
