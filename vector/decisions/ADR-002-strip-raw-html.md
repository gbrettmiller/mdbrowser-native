# ADR-002: Strip raw HTML from documents, keeping the text between tags

**Date:** 2026-10-29
**Status:** accepted
**Deciders:** gBRETT (product owner)

## Context

CommonMark lets a document embed raw HTML, both inline (`<b>`, `<img …>`) and as whole blocks (`<div>…</div>`, `<!-- … -->`). Phase 1 shows it as literal text (`src/model/inlines.rs`, `src/model/blocks.rs`), so nothing is interpreted and nothing is lost. Real posts carry stray markup, though, and the literal tags read as noise. VECTOR.md listed this as an open question to settle before phase 2. Remote browsing will bring far more third-party documents to the reader, so the noise will be seen far more often.

## Decision Drivers

- Constraint: no HTML, DOM or webview anywhere in the rendering path. HTML is never interpreted.
- Principle 2: the reader owns the presentation. A document's markup must not show up as reader-facing noise or styling.
- Principle 4, interop before invention: markdownWeb, the community reference, strips HTML.
- Malformed input never stops a document from rendering. The same tolerance applies to broken tags.
- Under principle 1 (the parse tree is the document), any handling happens in `model`, a pure layer, so it needs no I/O and adds no dependency.

## Options Considered

### Option A: Keep showing HTML as literal text (status quo)

**Pros:**
- Nothing in the source is ever hidden from the reader.
- Already built and tested.

**Cons:**
- Tags such as `<img src=… onerror=…>` and `<!-- … -->` appear verbatim, so posts look broken.
- It diverges from markdownWeb, so the same post reads differently in our browser.

### Option B: Strip tags and keep the text between them

The model drops every HTML tag, comment and declaration from `Html` and `InlineHtml` events. Any text outside the angle brackets is kept as ordinary text in the surrounding style.

**Pros:**
- Matches the community. Stray markup disappears, and words wrapped in tags (`<b>hi</b>` becomes `hi`) survive.
- It is still a removal, not an interpretation. No HTML semantics are applied: no bold from `<b>`, no image from `<img>`.
- It is a pure function in `model`, testable against the corpus.

**Cons:**
- What sits inside `<script>` or `<style>` survives as plain text (`<script>alert(1)</script>` becomes `alert(1)`). It is visible but inert.
- Information carried only in attributes is lost, for example `<img alt="…">` or `<a href="…">`.

### Option C: Strip tags and drop the contents of known non-text elements

Option B, but the whole contents of `script`, `style` and similar elements are dropped too.

**Pros:**
- Cleaner output for pages that inline scripts or styles.

**Cons:**
- To know which elements count as "non-text", the model needs a list of HTML element semantics. That edges towards interpreting HTML.
- Unclosed or misnested tags could swallow real content silently.

## Decision

**We will strip raw HTML tags, comments and declarations, and keep the text between them.**

This matches markdownWeb (principle 4) and stops writer markup reaching the reader (principle 2). It does that without the model learning what any element means, so the browser still never interprets HTML. Option C was rejected because swallowing an element's content requires knowing its semantics, and a broken tag could silently hide prose.

## Consequences

**Positive:**
- Remote posts with embedded markup read cleanly.
- The rule is simple and pure, so it gets unit tests and corpus tests.

**Negative:**
- A small amount of content can disappear silently: attribute-only content, and text a writer deliberately put inside a tag.
- Script and style bodies show as plain text. This is accepted as rare. Revisit if the corpus shows otherwise.
- A document that is *about* HTML must use code spans or fenced code to show tags. Code is never stripped.

**Neutral:**
- `features/browse.feature` "Raw HTML is shown as literal text" becomes "Raw HTML is stripped". The new version asserts that no tag appears, the surrounding text survives, and nothing is interpreted. That change lands with the code, test-first.
- ARCHITECTURE.md "What Not to Do" changes from "shown as literal text" to "stripped, never interpreted". The VECTOR.md open question is resolved.
- `<img>` in HTML never becomes an image, even after phase 2b adds image rendering. Only Markdown `![]()` images render.

## Related Decisions

- ADR-001: Render Markdown natively with our own layout engine (this ADR keeps its "no HTML in the rendering path" premise)
