# Plan: Phase 3: shared front matter

**Created**: 2026-10-30
**Branch**: `feat/phase-3`, from `main` after phase 2d (#11)
**Status**: decisions made (2026-10-30), ADR-006; not started

## Goal

Act on the community front matter vocabulary. Today only `title` is read (`model::frontmatter::title`); the rest of the block is dropped. 16,755 of the 16,869 archive posts carry `author`, `pubDate`, `source`, `urlPrev` and `urlNext`, so readers see none of who wrote a post, when, or what comes next. 3 adds a byline, previous/next navigation, a description, and an enclosure link, and ignores every other key (VECTOR.md, Constraints).

This phase closes two ship criteria: "Shows title, author and date from front matter, and follows `urlPrev` and `urlNext`", and part of "Renders the scripting.com archive".

## Where it fits today

- `model::frontmatter::split` already isolates the block, and `title` parses it with yaml-rust2. `Document` holds only `frontmatter_title`.
- Relative `urlPrev: "a164241.md"` and `urlNext: "../22/a153158.md"` resolve like any link, so `session::links` and the `url` crate already do the hard part.
- The page is a flat `Page` built by `layout::layout(theme, width, …)`. A byline and navigation can be ordinary blocks or a header/footer built from `Document`, so layout needs no new concept.
- Strings go in `content`, values in `theme`.

## What the corpus shows

Surveyed in `corpus/markdownweb/` (examples 0–8 and three extras):

| Key | Seen as | Notes |
|---|---|---|
| `author` | string | Always present with `pubDate`. |
| `pubDate` | RFC 822 string (`Sun, 31 Mar 2024 16:17:30 GMT`) | RSS 2.0 date format. Must parse it; show the original text if it doesn't parse. |
| `source` | map: `url`, `title` | The post's feed. Title is the site name. |
| `urlPrev` / `urlNext` | relative or absolute path | Newest post has no `urlNext`; the first may have no `urlPrev`. Targets often don't exist locally. |
| `description` | string | 713 files. |
| `enclosure` (+ `enclosureType`, `enclosureLength`) | URL, MIME type, byte count as strings | 193 files, podcasts. |
| `image` | URL | **Repeats the leading body image** (example 2). |
| `inlineImage` | URL | **Repeats the one body image** (`a020940`). |
| `metaImage` | URL | **Not in the body** (example 1). Reads as a social-card image. |
| `generator`, `urlvideo`, `urltoot` | strings | Unknown to the vocabulary. Ignored. |

## Proposed rules (recommendation)

1. **Parse into a struct, once.** `model::frontmatter::parse(yaml) -> FrontMatter` with `title`, `author`, `pub_date`, `description`, `source`, `url_prev`, `url_next`, `enclosure`. A pure function; every field is optional; wrong-typed values (a list where a string belongs) are dropped, never an error. `Document` carries a `FrontMatter` instead of `frontmatter_title`.
2. **Byline**: under the title/first heading, one muted line: `author · date`, plus the site name from `source.title` when present. Absent fields drop out; nothing shows if none are present. Date is shown in the reader's locale-neutral form (`31 March 2024`); a `pubDate` that doesn't parse is shown as written.
3. **Previous / next**: a row at the foot of the page, `← Previous` and `Next →`, each a normal link navigation (Back returns to the post, History records the target). Targets resolve against the page address, so they work across files and across `http(s)`. A missing target shows the usual not-found screen. Also keyboard shortcuts (proposal: Alt+Left/Right are Back/Forward already, so use `[` and `]`).
4. **Description**: shown as a muted lead paragraph under the byline. If the body already begins with the same text, show it once.
5. **Enclosure**: a link line, `Listen: <file name> (<type>, <size>)`, that hands off to the system via `services::opener`. No in-app playback (VECTOR open question "Enclosures" resolved as link-out; see decisions).
6. **Images**: the body is the source of truth. `image` and `inlineImage` add nothing, since the corpus repeats the body image. `metaImage` is not drawn. Recorded as an assumption pending markdownWeb issue #1; the fields are parsed so a later decision costs no migration.
7. **`source`** is shown as the site name in the byline only. Following the feed is phase 5.
8. **Unknown keys** are ignored, and malformed front matter never stops rendering (existing behaviour, kept and tested).
9. **The notice rule from 2d is unchanged**: none of this alters fetch, fallbacks or the address bar.

## Decisions (2026-10-30)

See ADR-006: the byline, description and previous/next sit in the page body, not fixed chrome; `image`, `inlineImage` and `metaImage` are parsed but draw nothing beyond the body's images (until markdownWeb issue #1); enclosures link out to the system, with no in-app playback.

## Out of scope

Reader controls (phase 4), `llms.txt` index, RSS and OPML (phase 5), `urlvideo` and `urltoot` (not in the vocabulary), floating a leading image to the right (VECTOR open question), downloadable apps.

## Acceptance Criteria

1. [ ] A post with `author` and `pubDate` shows a byline; with neither, nothing is added.
2. [ ] `pubDate` in RFC 822 form is shown readably; an unparseable value is shown as written.
3. [ ] `urlNext` and `urlPrev` links appear when present, resolve relative to the page (including `../`), work for files and `http(s)`, and a missing target gives the usual error.
4. [ ] The newest post (`urlPrev` only) and a post with neither show only the links that exist.
5. [ ] `description` shows once; the `enclosure` line opens the system handler for http(s) only.
6. [ ] Wrong-typed, empty or malformed values are dropped without affecting the rest of the page, and unknown keys are ignored.
7. [ ] `image`, `inlineImage` and `metaImage` never add an image beyond the body's.
8. [ ] Examples 0–8 and the three extras render with no crash; screenshots on every font and both palettes are read and checked.
9. [ ] Strings in `content`, values in `theme`, scenarios in `features/frontmatter.feature`, all gates green.

## Steps

1. ~~ADR-006~~ done: `vector/decisions/ADR-006-shared-front-matter.md`.
2. `features/frontmatter.feature` from the acceptance criteria.
3. `model::frontmatter::parse` and `FrontMatter`, test first, including the corpus `front-matter/` cases. Move `title` onto it.
4. RFC 822 `pubDate` formatting as a pure function (no new crate if `time`/`chrono` isn't already needed; otherwise justify one in the ADR).
5. Layout: byline and description blocks, and the previous/next row, as part of the built `Page`. Golden snapshots for the corpus posts whose glyphs are in the bundled fonts.
6. Session and reducer: prev/next as link events resolved against the current address; shortcuts in `ui::keys`.
7. Enclosure line through `services::opener`.
8. `content` strings and `theme` tokens, then UI wiring.
9. Docs: VECTOR.md (phase 3 built, ship criteria, open questions), `features/README.md`, DESIGN.md if tokens change.
10. `/code-review`, `/pr`.

## Open risks

- The vocabulary is undocumented and one corpus defines it (ASM-004). Parsing is lenient on purpose.
- `pubDate` formats outside RFC 822 may exist in the wild; the show-as-written fallback covers them.
- Prev/next targets often don't exist in the bundled corpus, so manual checks use the live site and the acceptance server.
