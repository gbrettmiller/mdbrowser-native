# Design

The visual system is **the reader stylesheet** in `src/theme/`: colour tokens, the type scale, spacing, and bundled fonts. This file documents it and does not define it. When the two disagree, the code wins and this file gets updated (`/impeccable document`).

Documents carry no presentation (VECTOR principle 2), so every visual decision lives here.

## Typefaces

All faces are bundled static files (OFL), so the app never depends on host font metadata. The reader picks the body face in the Settings panel, and the choice persists.

| Menu name | Family in file | Role |
|---|---|---|
| **Literata** (default) | Literata | Body. A serif designed for long-form screen reading. |
| Source Serif 4 | Source Serif 4 | Body. A traditional book serif. |
| iA Writer Quattro | iA Writer Quattro S | Body. A "writing app" sans with deliberately wide spacing. Its Bold file declares weight 400, which `fonts::install()` corrects. |
| Inter | Inter | Body. A neutral sans. Also the face for the app's own chrome. |
| JetBrains Mono | JetBrains Mono | Code, and list bullets (the only bundled face with • ◦ ▪) |

Task-list checkboxes are drawn as shapes rather than typed glyphs, because none of the body faces except Source Serif has ☐ and ☑.

## Tokens

| Token | Value | Notes |
|---|---|---|
| Body | 18 px by default, line height 1.6 | Text size is the reader's: 12 to 32 px in steps of 2 (Settings, or Ctrl+= , Ctrl+- and Ctrl+0; Cmd on macOS). Headings, code and image notes are multiples of it |
| Code | Mono at 0.88× body | Code blocks wrap by glyph |
| Measure | 40 em (720 px at 18), gutter 32 px at 18 | Text column centred in the window. The measure, gutter and indent scale with the text size, so characters per line stay about the same. Code and cell padding, the 6 px gap after a heading and the 3 px quote bar stay fixed |
| Headings | 1.9 / 1.5 / 1.25 / 1.1 / 1.0 / 0.9 × body, bold | Rule under H1 and H2 |
| Spacing | Compact: line height 1.45, block gap 12 px, item gap 2 px. **Comfortable (default): 1.6, 16 px, 4 px.** Spacious: 1.8, 22 px, 8 px | One choice sets all three. Indent is 28 px at 18 and scales with size. A sublist hugs its item |
| Palette (light) | background `#fbfaf7`, ink `#222`, muted `#6b6b6b`, link `#1f5fbf`, code background `#efede8`, rule `#d8d5ce`, selection link at 22% | Chosen in Settings: System (follows the OS, light if it reports nothing), Light or Dark |
| Palette (dark) | background `#1b1b1a`, ink `#e4e2dc`, muted `#a19f98`, link `#7faff0`, code ink `#d6d4ce`, code background `#262624`, rule `#3a3936`, selection link at 30% | Ink, muted and link meet WCAG AA (4.5:1) on the page and code backgrounds; a unit test checks it |
| Palette (custom) | The reader types page, text and link as `#rgb` or `#rrggbb`. Code ink is the text colour. Muted is the text faded toward the page (up to 35%), the code background is the page nudged toward the text (up to 6%), and each backs off until it meets 4.5:1. Rule is the page 15% toward the text. Selection is the link at 22% on a light page and 30% on a dark one | A set whose text or links are below 4.5:1 on the page is refused with a message and the old colours stay, because the Settings panel wears the palette too. Choosing Custom the first time starts from the palette in use. The colours are kept when another appearance is chosen. A unit test checks every derived pair across a spread of inputs |

### Focus and chrome text

| Token | Value | Notes |
|---|---|---|
| Focus ring | The link colour, 2 px, outside the shape (2 px clear of a button). Over every line of a wrapped link | `Theme::ring()`. Needs no new colour: link on page is at least 4.5:1 in every palette, and a unit test checks the ring meets 3:1 |
| Chrome text | `NOTE` 13, `LABEL` 14, `BODY` 16, `HEADING` 18, `HEADLINE` 20 px (`theme::chrome_text`) | Fixed: it does not follow the reader's text size, which is for the document. A 300 px panel and a one-line toolbar would break at 32 px. A reader who needs larger chrome has the OS scale. The floor is 13 px; a test fails if the UI sets a literal size |
| Panels | 300 px each; the page column is at least 320 px; window at least 480 by 360 | Opening a second panel closes the older one when the page would be narrower. A resize never closes a panel |

Decision and reasons: ADR-008.

The bundled fonts add about 5.3 MB to the binary (`du -sh assets/fonts`, 2026-10-28).

Accessibility (contrast, focus, keyboard reach) is owned by `a11y-review`. Screen readers are a known gap: iced 0.14 has no accessibility tree (ADR-008). Where Impeccable's rules overlap with it, they are advisory only.
