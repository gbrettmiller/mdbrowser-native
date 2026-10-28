# Spike findings: a DOM-free Markdown browser

**Question:** Can we parse Markdown into a tree, lay it out with our own engine, and paint it natively — no HTML, no webview — with quality good enough to read?

**Verdict: yes. Go with strategy 2 (own layout and painting) on Rust + iced + cosmic-text + pulldown-cmark.**

## What was built (about 1,400 lines including tests)

| Module | Role | Pure? |
|---|---|---|
| `model.rs` | pulldown-cmark events → owned block/span tree; HTML shown as literal text | yes |
| `layout.rs` | tree + theme + width → positioned fragments and fills; hit-testing, carets, selection, copy | yes (uses the shared font DB) |
| `session.rs` | navigate / back / forward | yes |
| `links.rs` | href → document / external / anchor | yes |
| `theme.rs` | the reader stylesheet | data |
| `view.rs` | iced canvas: paint, link clicks, drag-select, Ctrl+A / Ctrl+C | I/O edge |
| `main.rs` | browser shell: address bar, history, status bar, local file fetch | I/O edge |

26 unit tests; clippy `-D warnings` clean.

## Verified

- Headings, paragraphs, bold, italic, strike, inline code, links, nested lists, ordered lists with a start number, task lists, loose lists, nested quotes, tables with column alignment, fenced and indented code, rules, YAML frontmatter skipped, inline and block HTML rendered as literal text.
- Wrapping, hanging indents and mixed scripts (Greek, Cyrillic, Hebrew, Arabic, CJK via fallback).
- Ctrl+A / Ctrl+C through the real event loop: the highlight crosses blocks, and copied text keeps paragraph structure (soft wraps become spaces).
- Performance (release build): the 206 KB CommonMark spec parses in 3 ms and lays out in about 100 ms (7.8k fragments). A 140 KB synthetic document lays out in 50 ms (12k fragments).

## Not verified

Mouse clicks (on links, and drag-select) and hover could not be automated in this environment. The logic behind them is unit-tested, but it needs a manual check.

## Problems found, and what the real project should do

1. **Font metadata on the host is unreliable.** iA Writer Quattro labels its Bold face as Regular, and Adwaita Sans is variable, which fontdb sees as a single weight, so bold silently fell back to Noto Sans. → **Bundle static font files with the app** and load them via `iced::font::load`. Don't depend on system fonts for the reading face.
2. **cosmic-text picks one font per word.** A font change mid-word (`(`code`)`) shaped the code in the serif font, so the painted text overflowed its measured box. → Worked around by inserting a zero-width space at mixed-font boundaries (regression test included). Keep that test, and watch upstream.
3. **Text is shaped twice** (once to measure, once when iced paints each fragment). It's correct today because both use iced's shared font system, but it's wasted work and two shapings could drift apart. → Use a custom widget that paints our shaped cosmic buffers directly (iced's raw-text primitive) in place of the canvas `fill_text`.
4. **Re-layout on resize costs 50–100 ms** for very large documents, and the canvas holds the whole document's geometry. → Lay out per block with a cache keyed by width, add viewport culling, and lay out large documents off the UI thread.

## Still out of scope

Images, in-page anchors, http(s) fetching with `Accept: text/markdown`, find-in-page, dark theme, zoom, and keyboard scrolling.

## Parser choice

pulldown-cmark (event stream) was sufficient. Building an owned tree from it took about 200 lines, so comrak isn't needed.
