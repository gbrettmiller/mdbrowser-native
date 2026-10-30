# Acceptance scenarios

Gherkin specs for user-visible behaviour, ported from the predecessor `mdbrowser` and adapted for a renderer with no HTML.

`cargo test` runs them through the `cucumber` crate (`tests/acceptance/`). Steps drive the real session reducer, read real files in a temporary home, fetch real pages from a local web server, and persist through the real store. Only the system opener is faked. Steps assert on session state and recorded effects, never on pixels. The one exception is "set in", which inspects the font faces that shaped the laid-out page.

- `browse.feature` — addresses, links, back/forward, reload, errors and History (ported from `mdbrowser`)
- `reading.feature` — choosing the reading font
- `reader-controls.feature` — text size and its limits, the three spacings, custom colours (hex, refusals, readability), reset, and settings files from earlier phases
- `settings.feature` — appearance (System, Light, Dark), the home page and the Home button, and the built-in welcome page
- `fallbacks.feature` — a page that isn't Markdown: its Link alternate or `.md` sibling is shown, the reader is told, and `llms.txt` is offered, never substituted
- `frontmatter.feature` — the shared front matter: byline, description, previous and next, the enclosure line, and values that are missing, of the wrong kind or unknown
- `remote.feature` — pages over `http(s)`: what counts as Markdown, redirects, fetch failures, links across sites, and Open in browser
- `keyboard-access.feature` — Tab through a page's links, Enter, scrolling by key, and panels giving way in a narrow window. Driven through the pure rules the widgets use; focus on the toolbar and Settings controls is covered by unit tests and screenshots
- `images.feature` — images from files and the web: formats, what can't be shown, the Web images setting, clicks, linked images, four loads at a time, and the cache

**The web server fixture.** Each scenario gets its own std-only HTTP server on `127.0.0.1` with a random port (`tests/support/http.rs`, also used by `services::web`'s unit tests). Its routes are listed at the top of `remote.feature` and defined in `tests/acceptance/world.rs`. A step can change a route (`"/broken.md" now serves …`), hold a load until released, and count the requests the server saw. The world fails any load addressed anywhere but the test server, so scenarios never reach the internet.

**Image fixtures** are generated when the suite starts (`tests/acceptance/fixtures.rs`), so no binary files are checked in. That includes size-bomb PNGs that only claim a size in their header. A step can hold image loads under a path ("loading each image under … is slow") and shrink the image cache.

Nothing is waiting on a decision. Following the OS light or dark mode is checked by unit tests and screenshots, not here.
