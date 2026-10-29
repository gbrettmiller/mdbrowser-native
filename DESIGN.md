# Design

The visual system is **the reader stylesheet** in `src/theme/`: colour tokens, the type scale, spacing, and bundled fonts. This file documents it and does not define it. When the two disagree, the code wins and this file gets updated (`/impeccable document`).

Documents carry no presentation (VECTOR principle 2), so every visual decision lives here.

## Typefaces

All faces are bundled static files (OFL), so the app never depends on host font metadata. The reader picks the body face from the toolbar's font menu, and the choice persists.

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
| Body | 18 px, line height 1.6 | |
| Code | Mono at 0.88× body | Code blocks wrap by glyph |
| Measure | 720 px max, 32 px gutter | Text column centred in the window |
| Headings | 1.9 / 1.5 / 1.25 / 1.1 / 1.0 / 0.9 × body, bold | Rule under H1 and H2 |
| Block gap | 16 px; list items 4 px; indent 28 px | A sublist hugs its item |
| Palette (light) | background `#fbfaf7`, ink `#222`, muted `#6b6b6b`, link `#1f5fbf`, code background `#efede8`, rule `#d8d5ce`, selection link at 22% | Dark palette still to come |

The bundled fonts add about 5.3 MB to the binary.

Accessibility (contrast, focus, keyboard reach) is owned by `a11y-review`. Where Impeccable's rules overlap with it, they are advisory only.
