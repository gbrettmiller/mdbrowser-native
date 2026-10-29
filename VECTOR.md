---
# VECTOR.md — Project Doctrine
# Single source of truth for project intent, audience, and knowledge.
# Read this before CLAUDE.md. Read CLAUDE.md before writing code.
vector_version: "0.1"
project:
  name: "mdbrowser-native"
  description: "A desktop browser for the web of Markdown that parses .md into a document tree and draws it natively, with no HTML, DOM or webview anywhere."
  stage: "development"  # discovery | definition | development | delivery | maintenance
  started: "2026-10-28"
  repo: "https://github.com/gbrettmiller/mdbrowser-native"
owner:
  name: "gBRETT"
  role: ""
knowledge:
  research: "./vector/research/"
  schemas: "./vector/schemas/"
  decisions: "./vector/decisions/"
---

# Identity

## Problem Statement
Markdown is now a common format on the web (READMEs, docs sites, `llms.txt`, notes), but every way of reading it first converts it to HTML. That pulls in the whole web platform and lets HTML's presentation model decide how Markdown looks. The predecessor `mdbrowser` was, in effect, an HTML browser that only accepted Markdown.
[Refine: who feels this most, and why does it matter to them? Elicit with `/invest-interview`.]

## Target Audience
[2 sentences: who reads Markdown as a primary format today, what do they use, and what do they wish existed?]

## Core Value Proposition
[One outcome sentence, not features.]

## What This Is Not
- Not an HTML browser. Nothing is ever converted to HTML. Non-Markdown links are handed off to the system.
- Not an editor (at least not yet).
- Not a webview wrapper. No Chromium, WebKit or DOM in the rendering path.

# Design Principles
1. **The parse tree is the document.** The CommonMark/GFM tree is the document model, the way the DOM is for the web. Nothing downstream ever sees HTML.
2. **The reader owns the presentation.** Documents bring content, and the browser brings all of the style. Raw HTML in a document is shown as literal text, never interpreted.
3. **Borrow the hard primitives, own the engine.** Parsing, text shaping and windowing come from libraries. The flow from tree to layout, and selection, are ours.

# Constraints
- No HTML, DOM or webview anywhere in the rendering path.
- Phase 1 reads local files only. A later phase adds `http(s)` with `Accept: text/markdown`.
- Reading fonts are bundled with the app, not resolved from the host (spike finding 1).

# Quality Gates

## Definition of Done
- [ ] Tests written first and passing; `cargo test && cargo build && cargo clippy --all-targets -- -D warnings && cargo fmt --check` green.
- [ ] Acceptance scenarios in `features/` for the behaviour pass.

## Ship Criteria
- [ ] [criterion]

# Knowledge Map

## Key Assumptions
1. Enough content is published as Markdown (`.md` URLs, `Accept: text/markdown`, `llms.txt`) to make browsing worthwhile — hypothesis
2. A purpose-built layout engine for Markdown's closed set of node types gives reading quality equal to or better than HTML rendering — testing (spike and phase 1 screenshots)
3. Re-layout and painting can stay smooth for 200 KB+ documents — testing: painting is culled to the viewport and text is shaped once, but a full re-layout of the 206 KB spec takes 150–230 ms, so resizes defer it until the width settles

## Open Questions
- Which discovery conventions to support, and in what order?
- Remote images: load, click-to-load, or block?
- ~~How should in-page anchors map to headings (slug rules)?~~ Resolved in phase 1: GitHub-style slugs (`model::slug`), so anchors written for GitHub work unchanged.

## Research Status
| Artifact | Status | Location |
|----------|--------|----------|
| Rendering spike | Done (2026-10-28) | `./vector/research/2026-10-28-spike-findings.md` |
| Phase 1 (local browsing) | Built (2026-10-29) | `features/browse.feature`, `features/reading.feature` |
| Interviews | Not started | `./vector/research/` |
| Assumptions | Not started | `./vector/research/` |
