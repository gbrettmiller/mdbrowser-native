# Acceptance scenarios

Gherkin specs for user-visible behaviour, ported from the predecessor `mdbrowser` and adapted for a renderer with no HTML.

`cargo test` runs them through the `cucumber` crate (`tests/acceptance/`). Steps drive the real session reducer, read real files in a temporary home, fetch real pages from a local web server, and persist through the real store. Only the system opener is faked. Steps assert on session state and recorded effects, never on pixels. The one exception is "set in", which inspects the font faces that shaped the laid-out page.

- `browse.feature` — addresses, links, back/forward, reload, errors and History (ported from `mdbrowser`)
- `reading.feature` — choosing the reading font
- `remote.feature` — pages over `http(s)`: what counts as Markdown, redirects, fetch failures, links across sites, and Open in browser

**The web server fixture.** Each scenario gets its own std-only HTTP server on `127.0.0.1` with a random port (`tests/support/http.rs`, also used by `services::web`'s unit tests). Its routes are listed at the top of `remote.feature` and defined in `tests/acceptance/world.rs`. A step can change a route (`"/broken.md" now serves …`), hold a load until released, and count the requests the server saw. The world fails any load addressed anywhere but the test server, so scenarios never reach the internet.

Not ported yet, waiting on decisions: images (see VECTOR open questions) and theme switching (dark palette not designed).
