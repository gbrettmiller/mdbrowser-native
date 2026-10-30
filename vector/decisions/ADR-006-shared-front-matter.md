# ADR-006: Read the community front matter vocabulary leniently and show it in the page body

**Date:** 2026-10-30
**Status:** accepted
**Deciders:** gBRETT (product owner)

## Context

Phase 3 (`vector/plans/phase-3-shared-front-matter.md`) acts on the Web of Markdown front matter vocabulary. Today only `title` is read. 16,755 of the 16,869 archive posts carry `author`, `pubDate`, `source`, `urlPrev` and `urlNext`, so readers never see who wrote a post, when, or what comes next. The vocabulary is undocumented and one corpus defines it (ASM-004). The open questions are how much to trust it, where the results appear, what the image fields mean, and what an enclosure does. Doing nothing leaves the ship criterion "Shows title, author and date from front matter, and follows `urlPrev` and `urlNext`" unmet.

## Decision Drivers

- VECTOR.md Design Principle 4: interop before invention. RSS 2.0 field names, `urlPrev` and `urlNext` are used as-is, and what we don't understand is ignored.
- VECTOR.md Constraints: unknown keys are ignored, and malformed front matter never stops a document from rendering.
- VECTOR.md Design Principle 2: the reader owns the presentation, so metadata is shown in the reader's own typography.
- VECTOR.md Design Principle 5: writing and reading only, so no playback or app features.
- The corpus shows `image` and `inlineImage` repeating an image already in the body (example 2, `2022/05/21/a020940`), while `metaImage` is not in the body (example 1). Their intended meaning is unconfirmed (markdownWeb issue #1).
- ARCHITECTURE.md: parsing is pure and belongs in `model`; handing a link to the system belongs to `services::opener`.

## Options Considered

### Option A: Do nothing

Keep reading `title` only.

**Pros:**
- No new code or rules.

**Cons:**
- Misses the ship criterion, and the archive reads as bare text with no way to its neighbours.

### Option B: Lenient parse, show in the page body, link out for enclosures

`model::frontmatter::parse` returns a `FrontMatter` with every field optional: `title`, `author`, `pub_date`, `description`, `source`, `url_prev`, `url_next`, `enclosure`. A value of the wrong type is dropped, and unknown keys are ignored. The byline (author, date, site name from `source.title`), the description and the previous/next row are part of the page the layout builds. They scroll with the post and use the reader's typeface and palette. A `pubDate` in RFC 822 form is shown readably, and anything else is shown as written. The enclosure is a link line handed to the system through `services::opener`, for http and https only. `image`, `inlineImage` and `metaImage` are parsed but draw nothing beyond the body's own images.

**Pros:**
- Matches the community convention and the corpus, with nothing invented.
- No new chrome, so the reader's presentation choices apply to it unchanged.
- Previous/next are ordinary link navigations, so Back, History and cross-site resolution already work.
- Parsing the image fields now means a later decision on them needs no migration.

**Cons:**
- The previous/next row is at the foot of the page, so on a long post the reader scrolls to reach it.
- Posts whose image fields are meant to be shown (if issue #1 says so) lack that image until revisited.
- Enclosures can't be played in the app.

### Option C: Fixed chrome for navigation, draw `image`, play enclosures

Put previous/next in the toolbar, draw `image` as a leading image when the body has none, and play audio enclosures in the app.

**Pros:**
- Navigation is always in reach, and podcasts play without leaving the app.

**Cons:**
- New chrome that the reader's choices don't govern, and a toolbar that varies by document.
- Audio needs a decoding and output stack, which turns a document into an application (Principle 5).
- Drawing `image` guesses at semantics nobody has documented, and can duplicate the body's image.

## Decision

**We will take Option B.**

It follows the community vocabulary as-is (Principle 4), keeps metadata in the reader's typography (Principle 2), and keeps the browser to reading (Principle 5). The image fields are the one place where the vocabulary's meaning is unconfirmed, so we parse them and draw nothing until markdownWeb issue #1 or a wider corpus says otherwise.

## Consequences

**Positive:**
- The byline and previous/next close two ship criteria.
- A malformed or surprising value costs the reader one line of metadata, never the page.
- `FrontMatter` is the single place later phases (reader controls, discovery via `source`) read from.

**Negative:**
- No in-app playback and no `metaImage` card. Changing either needs a new ADR.
- `FrontMatter` replaces `Document::frontmatter_title`, so callers and tests that read the old field change.
- A page that gets its previous/next target wrong shows a not-found screen; we don't probe targets ahead of time.

**Neutral:**
- `generator`, `urlvideo` and `urltoot` are ignored, though the archive uses them.
- Following a post's `source` feed is phase 5; phase 3 shows only the site name.
- Date formatting is a pure function with no new date crate unless the build step shows one is needed. If it is, that is recorded here as an amendment.

## Related Decisions

- ADR-002: Strip raw HTML. Same stance of ignoring what we don't understand.
- ADR-004: Images. Front matter image fields were left to phase 3 there, and are settled here as parse-only.
- ADR-005: Markdown fallbacks. Fetch and address handling are unchanged by this decision.
