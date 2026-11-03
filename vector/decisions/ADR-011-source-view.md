# ADR-011: The source view: a page's Markdown shown in the app as one code block

**Date:** 2026-11-03
**Status:** accepted
**Deciders:** gBRETT (product owner)

## Context

Some web pages have no front matter, so the Markdown version has no byline, date or previous/next links, and a reader cannot tell whether that is the author's choice or the browser's reading. The core promise is "read any Markdown exactly as the writer wrote it" (VECTOR.md), and the reader has no way to see what the writer wrote. Plan: `vector/plans/source-view.md`.

## Decision Drivers

- VECTOR.md: nothing is converted to HTML and nothing runs; the parse tree is the document, and the reader owns the presentation (font, size, spacing, palette).
- ADR-001: the app draws its own page, so a new view should reuse the layout engine, selection and copy, not add a second renderer.
- ADR-007 and ADR-008: reader controls and keyboard access must apply to every view, and the Settings palettes already guarantee AA contrast for code text on the code background.
- ARCHITECTURE.md: browsing logic in the pure session reducer, drawing in `ui`, text in `content`.
- Open time: VECTOR's bar for the largest document we ship (`corpus/spec.md`, 206 KB, 9,814 lines).
- The view is a lens on a page, not a place: it must not enter History or change a saved setting.

## Options Considered

### Option A: Open the source in an external editor or browser

Hand the file or URL to the system.

**Pros:**
- No new view to build.

**Cons:**
- Leaves the app for the one thing the app exists to show, and local files would depend on the reader's editor.
- A fallback find shows a different address from the one typed; the system would not know which.

### Option B: A separate window or a side panel

Show the source beside or apart from the page.

**Pros:**
- Page and source visible together.

**Cons:**
- A panel gives up page width (the 620 px minimum window leaves one panel only, ADR-008), and a second window adds window management for a rare action.

### Option C: Swap the page for its source in the same view, as one code block (chosen)

Ctrl/Cmd+U and a toolbar Source/Page button toggle the document area between the page and a one-block document of the fetched text.

**Pros:**
- Reuses the layout engine, `Pages` cache, `Viewport`, selection, copy, text size, font and palette without a second renderer.
- Keyboard and mouse routes both exist; one rule for Esc.

**Cons:**
- One large text block; see consequences.

### Option D: Swap the view, but chunk the source into many code blocks

Split the text every N lines to keep each layout small.

**Pros:**
- Might let layout work be spread or skipped.

**Cons:**
- Measured below: layout cost is linear in the text, so chunking costs the same in total and adds seams (padding, background and gaps) between chunks.

## Decision

**We will show a page's source in the same view, as one `Block::Code`, toggled by Ctrl/Cmd+U or a toolbar button.**

This keeps the promise in VECTOR.md with the machinery ADR-001 already gave us. It adds no setting and no History entry.

- **State.** `session::View` is `Page` or `Source(Arc<Document>)`, held in the reducer. `Shown` keeps the fetched text as `Arc<str>`. The toggle works only with a page shown.
- **Text.** The byte order mark is dropped, and CRLF and CR become `\n`. An empty file shows an empty source. The text is otherwise exactly what was fetched, front matter included.
- **Leaving.** Any navigation (follow, Enter, Home, Back, Forward, a failed load) returns to the page. Reload keeps the source view and rebuilds it from the new fetch. After a fallback find the history entry is already the found `.md`, so Reload fetches that file.
- **Esc** closes Settings, then History, then leaves the source view.
- **Chrome.** The button reads Source or Page, is disabled without a page, and its tooltip names the chord (Cmd on macOS). The title line reads "Source of <title>". Focus goes to the page after a toggle.
- **Scroll.** The `Viewport` holds a second offset. Toggling swaps them (`swap_view`), so each view keeps its place; a new page or a reload resets both.
- **Large files.** Measured 2026-11-03 (release build, `corpus/spec.md`): the source lays out in 256 ms at 720, 540 and 360 px; the rendered page of the same file takes 216 ms. Cost is linear (about 28 µs a line: 1,226 lines in 45 ms, 4,907 in 155 ms). The bar is parity, not a fixed number: the source's first layout is at most 1.5 times the page's, asserted by the ignored test `source_timing` (`cargo test --release --lib -- --ignored --test-threads=1`, not run in CI). Layouts over `EXPENSIVE_LAYOUT` (32 ms) are deferred during a resize like any page. Sizes beyond `corpus/spec.md`, up to the 16 MiB fetch cap, are not guaranteed fast.

## Consequences

**Positive:**
- A reader can check what the writer wrote against what the browser drew, for files, web pages and fallback finds.
- Font, size, spacing and every palette apply. Code text is AA in all appearances (a test covers each).
- Selection and copy give back the source. As part of this, copying from any code block no longer inserts a space where a long line wraps mid-word, and keeps blank lines (`Joint::Lines`).

**Negative:**
- The source is held twice while the view is open (the fetched text and the laid-out block), and a very large file stalls the UI thread for the length of one layout. There is no virtual scrolling; adding it would be a layout change, and is the way to go if sizes beyond `corpus/spec.md` matter.
- Accessibility is the same limit as the page and no better: one large text block in a custom-drawn widget, so there is no structure for a screen reader (ADR-008).
- Known gaps, accepted for now: Esc after editing the address bar, with no panel open, also leaves the source view (the app does not know which control had focus); blank lines at the very start of a source are not copied; the `Document` widget's identity is the document's pointer, which could repeat after a freed source is reallocated.

**Neutral:**
- The source is never syntax-highlighted; reading it in the reader's own colours is the point.
- Chunking was rejected on measurement, so `source_document` stays a single block.

## Related Decisions

- ADR-001: Native rendering with our own layout engine (the reason one code block is enough).
- ADR-005: Markdown fallbacks (what Source and Reload show after a fallback find).
- ADR-007: Reader controls (palettes and AA).
- ADR-008: Keyboard access (the Esc order, focus, and the accessibility limit).
