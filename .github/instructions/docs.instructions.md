---
description: "Use when editing GEMINI.md, docs, ADRs, manifesto text, or project guidance. Covers architecture logging, decision records, and keeping Copilot/Gemini documentation aligned."
name: "UTO Documentation Guidance"
applyTo: "GEMINI.md, docs/**/*.md, .github/copilot-instructions.md"
---
# UTO Documentation Guidance

- Treat `GEMINI.md` and the ADRs in `docs/` as the project source of truth for architecture and workflow decisions.
- When a code change alters behavior or direction, update the most relevant ADR or add a new one instead of leaving the decision implicit.
- Keep Copilot-facing guidance aligned with the same architectural facts already recorded in `GEMINI.md` and the ADRs.
- Prefer concise, decision-oriented documentation over marketing copy or broad restatements.
- Document the "what" and the "why", especially for cross-platform tradeoffs, provisioning strategy, and driver/session boundaries.
- Keep framework UX direction (CLI lifecycle and reporting-first observability) synchronized across ADRs, `GEMINI.md`, README, and Copilot/Gemini instruction files.