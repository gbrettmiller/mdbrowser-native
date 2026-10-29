---
# VECTOR.md — Project Doctrine
# Single source of truth for project intent, audience, and knowledge.
# Read this before CLAUDE.md. Read CLAUDE.md before writing code.
vector_version: "0.1"
project:
  name: "mdbrowser-native"
  description: "A desktop browser for the Web of Markdown that parses .md into a document tree and draws it natively, with no HTML, DOM or webview anywhere."
  stage: "development"  # discovery | definition | development | delivery | maintenance
  started: "2026-10-28"
  repo: "https://github.com/gbrettmiller/mdbrowser-native"
owner:
  name: "gBRETT"
  role: "Developer and product owner"
knowledge:
  research: "./vector/research/"
  schemas: "./vector/schemas/"
  decisions: "./vector/decisions/"
---

# Identity

## Problem Statement
Markdown is now a common format on the web (READMEs, docs sites, `llms.txt`, notes, whole blog archives), but every way of reading it first converts it to HTML. That pulls in the whole web platform, including writer-controlled CSS and scripts, and lets HTML's presentation model decide how Markdown looks. The predecessor `mdbrowser` was, in effect, an HTML browser that only accepted Markdown.

The people who feel this most are the early **Web of Markdown** community (INT-001): writers who already publish Markdown from their own tools, and readers who want writing from many authors to look the way *they* like to read. For them, an HTML browser defeats the point. It hands appearance back to the writer, carries "all the dangerous stuff", and has no notion of the shared metadata (author, date, previous/next post) that the Markdown files carry.

## Target Audience
Readers who follow independent writers (blogs, newsletters and documentation) and want one comfortable, consistent presentation they control, with nothing running but the text. Writers who publish Markdown with the editor they already use (Obsidian, Jekyll, Hugo, Eleventy, Astro, or a converter like `opmlToMarkdown`) and want their words to arrive intact and link to other Markdown pages. Today that is a small community growing around the Web of Markdown proposal (Dave Winer, Andrew Shell, Dirk Primbs). Interop with what they publish is the first measure of success.

## Core Value Proposition
Read any Markdown on the web exactly as the writer wrote it, presented the way you like to read, with nothing else running.

## Direction
Each phase has its own plan. This is the order, not a commitment to dates.
1. **Local browsing** (built): addresses, links, back/forward, History, bundled reading fonts.
2. **Remote browsing**: `http(s)` with `Accept: text/markdown`, relative links between `.md` files across sites, remote images.
3. **Shared front matter**: act on the community vocabulary, including a byline from `author` and `pubDate`, previous/next navigation from `urlPrev` and `urlNext`, `description`, `image`, `inlineImage` and `metaImage`, an `enclosure` link, and `source` (the post's feed). Unknown keys are ignored.
4. **Reader controls**: text size, spacing and colours (including a dark palette), beyond the typeface choice.
5. **Discovery**: `llms.txt` as a site index, RSS 2.0 feeds and OPML.

Downloadable apps for Linux, Windows and macOS, so non-technical readers can install without building from source, come after phase 2. The plan is in `vector/plans/release-distribution.md`.

## What This Is Not
- Not an HTML browser. Nothing is ever converted to HTML, and HTML files are not rendered. Non-Markdown links are handed off to the system.
- Not an app platform. Documents carry no styling beyond Markdown's own emphasis, and no scripts. The web of Markdown is for writing and reading only.
- Not a new format. We adopt the community's conventions rather than inventing our own front matter, discovery or link rules.
- Not an editor (at least not yet). Writers keep their own tools.
- Not a webview wrapper. No Chromium, WebKit or DOM in the rendering path.

# Design Principles
1. **The parse tree is the document.** The CommonMark/GFM tree is the document model, the way the DOM is for the web. Nothing downstream ever sees HTML.
2. **The reader owns the presentation.** Documents bring content, and the browser brings all of the style. Font, size, spacing and colour belong to the reader. Raw HTML in a document is never interpreted.
3. **Borrow the hard primitives, own the engine.** Parsing, text shaping and windowing come from libraries. The flow from tree to layout, and selection, are ours.
4. **Interop before invention.** Where the community has a convention (RSS 2.0 field names, `urlPrev`/`urlNext`, `llms.txt`, GitHub heading slugs), use it as-is. Follow the [rules for standards-makers](https://this.how/standards/). Ignore what we don't understand instead of failing on it.
5. **Writing and reading only.** Every feature should help someone read what was written or get to the next thing to read. Anything that turns a document into an application doesn't belong here.

# Constraints
- No HTML, DOM or webview anywhere in the rendering path.
- Phase 1 reads local files only. Phase 2 adds `http(s)` with `Accept: text/markdown`.
- Reading fonts are bundled with the app, not resolved from the host (spike finding 1).
- Front matter is YAML. Keys we don't recognise are ignored, and malformed front matter never stops a document from rendering.
- Writers get no control over appearance. A document can't choose its font, colour or layout.

# Quality Gates

## Definition of Done
- [ ] Tests written first and passing; `cargo test && cargo build && cargo clippy --all-targets -- -D warnings && cargo fmt --check` green.
- [ ] Acceptance scenarios in `features/` for the behaviour pass.
- [ ] User-facing strings in `content`, and visual values in `theme`.

## Ship Criteria
First public release:
- [ ] Browses local files and `http(s)` Markdown, including relative links between `.md` pages on different sites.
- [ ] Renders the markdownWeb examples 0–8 and the scripting.com archive with no crashes, no HTML interpreted, and images shown.
- [ ] Shows title, author and date from front matter, and follows `urlPrev` and `urlNext`.
- [ ] The reader can set typeface, text size, spacing and a light or dark palette, and the choices persist.
- [ ] Opening a typical post takes under 250 ms on a local file. Scrolling and resizing don't stutter.
- [ ] Builds and passes CI on Linux. macOS and Windows build in CI.

# Knowledge Map

## Key Assumptions
Detail and test methods are in `vector/research/assumptions/`.
1. **ASM-001.** Enough content is published as Markdown to make browsing worthwhile. *Hypothesis, medium confidence:* the scripting.com archive (2017–2026) and community sites exist, but the community is small.
2. **ASM-002.** A purpose-built layout engine for Markdown's closed set of node types gives reading quality equal to or better than HTML rendering. *Testing:* spike and phase 1 screenshots. Next, compare against the markdownWeb examples.
3. **ASM-003.** Re-layout and painting stay smooth for 200 KB+ documents. *Testing:* a full re-layout of the 206 KB spec takes 150–230 ms, so resizes wait until the width settles. Blog posts are far smaller.
4. **ASM-004.** Writers and tools converge on the RSS 2.0-derived front matter vocabulary. *Hypothesis, low confidence:* only one corpus uses it, and it is undocumented.
5. **ASM-005.** Readers want full control of appearance and don't miss writer styling. *Hypothesis, medium confidence.*
6. **ASM-006.** Discovery works through `llms.txt`, RSS 2.0 and OPML without a new protocol. *Hypothesis, low confidence.*

## Open Questions
- **HTML in documents: strip it or show it literally?** markdownWeb strips HTML; we currently show it as literal text. Both keep principle 2. Recommendation: decide by ADR before phase 2. Stripping matches the community and keeps stray tags from becoming reader noise, but it can silently drop content.
- **Image semantics.** What exactly do `image`, `inlineImage` and `metaImage` mean, and where should each appear? They are undocumented, so ask in markdownWeb issue #1.
- **Remote images: load, click-to-load, or block?** Real posts depend on them (examples 2 and 3), which argues for loading by default with a reader setting.
- **Enclosures.** Link out to the system player, or play audio in the browser?
- **Which discovery convention comes first?** `llms.txt`, a post's `source` feed, or OPML?
- **Interop with other Markdown-web browsers.** Should we share a test corpus, for example with Andrew Shell's browser?
- Live updates over WebSockets: noted in INT-001 and not yet in scope.
- ~~How should in-page anchors map to headings (slug rules)?~~ Resolved in phase 1: GitHub-style slugs (`model::slug`), so anchors written for GitHub work unchanged.

## Research Status
| Artifact | Status | Location |
|----------|--------|----------|
| Rendering spike | Done (2026-10-28) | `./vector/research/2026-10-28-spike-findings.md` |
| Phase 1 (local browsing) | Built (2026-10-28) | `features/browse.feature`, `features/reading.feature` |
| Interviews | 1, draft. The markdownWeb readme was treated as a transcript: a published manifesto, not a live session. | `./vector/research/interviews/INT-001.json` |
| Assumptions | 6, draft | `./vector/research/assumptions/` |
| Community examples | Not yet in the corpus | [scripting/markdownWeb examples](https://github.com/scripting/markdownWeb/blob/main/examples/readme.md) |
