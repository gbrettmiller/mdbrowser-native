# ADR-001: Render Markdown natively with our own layout engine on Rust, iced, cosmic-text and pulldown-cmark

**Date:** 2026-10-28
**Status:** accepted
**Deciders:** [OPERATOR: List who was involved in this decision, or remove this line]

## Context

The predecessor `mdbrowser` (Tauri + Svelte) converted each `.md` file to HTML with markdown-it and displayed it in a webview. That made it an HTML browser that only accepted Markdown. VECTOR.md now forbids any HTML, DOM or webview in the rendering path, and requires that the parse tree *is* the document and the reader owns all presentation. A rendering stack has to be chosen before any feature work. Without that, every phase 1 step is blocked.

## Decision Drivers

- **No HTML, DOM or webview** (VECTOR constraint 1). This rules out every webview-hosted approach, whatever the language.
- **The parse tree is the document** (principle 1). We need direct control from tree nodes to positioned text.
- **High-quality text:** shaping, bidirectional text, font fallback and wrapping at reading-app quality.
- **Selection and copy that cross block boundaries**, as a browser should.
- **Functional style** (ARCHITECTURE conventions): parse, layout and session as pure, testable functions.
- **Cross-platform desktop**, with Linux (Hyprland/Wayland) first.

## Options Considered

### Option A: Keep the status quo (Tauri webview with markdown-it → HTML)

Leave `mdbrowser` as it is.

**Pros:**
- It already works, with a test suite and phase 1 features.

**Cons:**
- It violates the core constraint. Ruled out.

### Option B: Map Markdown nodes to toolkit widgets

A rich-text widget per paragraph, columns for lists, in iced or egui (egui_commonmark, iced's markdown widget).

**Pros:**
- Fastest route to pixels, because the toolkit does the layout.

**Cons:**
- Selection stops at widget boundaries.
- Presentation is limited to what the widgets offer. The "engine" is someone else's.

### Option C: Own layout and painting on Rust + iced + cosmic-text + pulldown-cmark

We build a pure function from tree, theme and width to positioned fragments, paint it in one document widget, and do our own hit-testing and selection.

**Pros:**
- The spike proved it works. All corpus elements rendered. The 206 KB CommonMark spec parses in 3 ms and lays out in about 100 ms.
- Selection and copy work across blocks.
- iced's Elm architecture fits a pure session reducer. Measuring and painting share one font system.

**Cons:**
- More work: we own the layout, selection and caching.
- cosmic-text has quirks (one font per word).

### Option D: GTK4/libadwaita

A TextView with tags, or a widget per block.

**Pros:**
- The most native look on Linux, and Pango text quality.

**Cons:**
- Object-oriented, mutable widget trees fight the functional style.
- Weaker on macOS and Windows. Selection would still split across widgets.

### Option E: Terminal UI (ratatui-style)

**Pros:**
- Markdown suits plain text naturally, and the scope is small.

**Cons:**
- No real typography, images or mixed font sizes, so it fails the reading-quality driver.

### Option F: JavaScript/Node with a non-DOM renderer

**Pros:**
- It's the owner's primary language.

**Cons:**
- There is no credible desktop text stack without a DOM (Skia/canvas bindings are immature for shaped rich text). Ruled out in practice.

## Decision

**We will build our own layout and painting engine (Option C) in Rust on iced 0.14, using cosmic-text through iced's shared font system and pulldown-cmark for parsing.**

Option C alone satisfies every driver. A, and in practice F, fail the hard constraint. B gives up cross-block selection and reader-owned presentation. D and E fail the style and quality drivers. The spike (`vector/research/2026-10-28-spike-findings.md`) is the evidence that the work is tractable.

## Consequences

**Positive:**
- The document is the parse tree end to end. No HTML can sneak in.
- Model, layout and session are pure and unit-testable without a window. Golden layout tests over `corpus/` become possible.
- Full control of typography, selection, find-in-page and future features like anchors and reading modes.

**Negative:**
- We own layout correctness: tables, nested blocks, images and RTL edge cases are ours to get right.
- **Reading fonts must be bundled.** Host font metadata proved unreliable: iA Writer's Bold is mislabelled, and Adwaita Sans is variable, so it looks like a single weight to fontdb.
- **cosmic-text shapes each word with a single font.** We keep the zero-width-separator workaround and its regression test until upstream fixes it.
- **The canvas prototype shapes text twice.** It must move to a custom widget that paints our shaped buffers directly.
- **Re-layout of huge documents (50–100 ms) is too slow for live resize.** We need per-block layout caching, viewport culling, and off-thread layout for large files.
- Moving from Rust/iced to another toolkit would mean rewriting the UI and paint layers. Model, layout and session stay portable.

**Neutral:**
- The JS test suite from `mdbrowser` doesn't carry over. Its Gherkin scenarios do (`features/browse.feature`).
- iced 0.14 is pre-1.0, so upgrades may bring breaking changes.

## Related Decisions

None. This is the first ADR in this repository. The predecessor's ADR-001 (local read scope hardening) was Tauri-specific and doesn't apply.
