# Acceptance scenarios

Gherkin specs for user-visible behaviour, ported from the predecessor `mdbrowser` and adapted for a renderer with no HTML.

`cargo test` runs them through the `cucumber` crate (`tests/acceptance/`). Steps drive the real session reducer, read real files in a temporary home, and persist through the real store. Only the system opener is faked. Steps assert on session state and recorded effects, never on pixels. The one exception is "set in", which inspects the font faces that shaped the laid-out page.

- `browse.feature` — addresses, links, back/forward, reload, errors and History (ported from `mdbrowser`)
- `reading.feature` — choosing the reading font

Not ported yet, waiting on decisions: images (see VECTOR open questions) and theme switching (dark palette not designed).
