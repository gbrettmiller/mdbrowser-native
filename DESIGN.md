# Design

The visual system is **the reader stylesheet** in `src/theme/`: colour tokens, the type scale, spacing, and bundled fonts. This file documents it and does not define it. When the two disagree, the code wins and this file gets updated (`/impeccable document`).

Documents carry no presentation (VECTOR principle 2), so every visual decision lives here.

## Starting point (from the spike)

| Token | Value | Notes |
|---|---|---|
| Body | 18 px serif, line height 1.6 | Spike used C059. To be replaced by a bundled static family. |
| Code | Monospace at 0.88× body | Spike used Adwaita Mono. To be bundled. |
| Measure | 720 px max, 32 px gutter | Text column centred in the window |
| Headings | 1.9 / 1.5 / 1.25 / 1.1 / 1.0 / 0.9 × body, bold | Rule under H1 and H2 |
| Block gap | 16 px; list items 4 px; indent 28 px | |
| Palette (light) | background `#fbfaf7`, ink `#222`, muted `#6b6b6b`, link `#1f5fbf`, code background `#efede8`, rule `#d8d5ce` | Dark palette still to come |

Accessibility (contrast, focus, keyboard reach) is owned by `a11y-review`. Where Impeccable's rules overlap with it, they are advisory only.
