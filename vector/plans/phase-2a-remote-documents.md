# Plan: Phase 2a: remote Markdown documents

**Created**: 2026-10-29
**Branches**: `feat/strip-html` (PR A, step 1), then `feat/phase-2a` (PR B, steps 2–10), both from `main`
**Status**: approved (2026-10-29, revision 3)

## Goal

Read Markdown on the web, not only on disk. Typing an `http(s)` address fetches it with an `Accept` header that prefers Markdown, and shows the page if Markdown comes back. Links between `.md` pages work across sites, so a local note can link to a remote post and a remote post to its neighbours. Back, forward, reload and History behave as they do for files. Raw HTML in any document is stripped (ADR-002), because remote browsing will bring a lot more of it to the reader. Image rendering is phase 2b.

Decisions already made by the product owner (2026-10-29):
- Phase 2 is split into 2a (remote documents) and 2b (image rendering, local and remote).
- ADR-002: strip raw HTML and keep the text between tags. Code is never stripped.
- Typed `http(s)` addresses are always fetched, whatever their extension. Links inside documents follow the extension rule: `.md`/`.markdown` opens in the app, and any other `http(s)` link goes to the system browser.

**Delivery.** The work ships as two PRs. PR A is step 1, HTML stripping. It is useful on its own and is a small, low-risk merge. PR B is remote browsing.

**User-facing terms**, used in `content.rs`:
- **address**: what the reader types
- **page**: a remote document
- **file**: a local document
- **site**: the server

"URL" is used only in code and tests.

## Acceptance Criteria

1. [ ] **Typed addresses.** An `http://` or `https://` address (scheme in any case) is fetched with an `Accept` header that ranks `text/markdown` highest. A Markdown response is shown with its title. The address bar always shows the full final URL, including its scheme.
2. [ ] **What counts as Markdown.**
   - Markdown: Content-Type `text/markdown` or `text/x-markdown` (case-insensitive, parameters allowed). Also `text/plain` or no Content-Type, when the final URL's *path* ends in `.md` or `.markdown`. Query and fragment are ignored, and case doesn't matter.
   - `notMarkdown`: anything else.
   - An empty 2xx body is a valid, empty document.
3. [ ] **Open in browser.** On `notMarkdown` for any web location (typed or followed), the error offers **Open in browser**, which hands the URL to the system browser. Local files never offer it.
4. [ ] **Redirects.** Statuses 301, 302, 303, 307 and 308 are followed up to 5 hops. The address bar, History and relative links use the final URL.
   - These give `badRedirect`:
     - a 6th hop
     - https→http
     - a non-http(s) target
     - a missing or empty `Location` header
     - a public host redirecting to a loopback, private or link-local address, or to `localhost` (ADR-003)
   - Any other 3xx gives `serverError`. Any 2xx is a success.
   - User names and passwords in a redirect target are dropped, never sent.
5. [ ] **Failures.**
   - 404 and 410 → `notFound`.
   - Other 4xx and 5xx → `serverError`.
   - Refused connection, DNS, TLS and timeout failures → `unreachable`.
   - A body over 16 MiB, invalid UTF-8, or a declared charset other than UTF-8 or US-ASCII → `unreadable`.
   - Every web error shows the address it was for, plus a hint: a non-empty sentence saying what to try next. The content test asserts this for every web error code.
6. [ ] **Links in a remote page.** Relative, root-relative and scheme-relative links resolve against the page's final URL. `.md`/`.markdown` links open in the app; other `http(s)` and `mailto:` links go to the system browser; `#fragment` links scroll.
7. [ ] **Remote pages can't reach the disk.** `file:` links, drive-letter paths (`C:\x.md`) and UNC paths (`\\host\x.md`) are refused with the notice "Web pages can't open files on your computer." The page stays put, and nothing is read from disk. Bare absolute paths resolve as URLs on the same site. A public page's Markdown link into the local network (loopback, private, link-local, `localhost`, however spelled) is refused with "Web pages can't open addresses on your local network." (added in step 8).
8. [ ] **Links in a local file.** An `http(s)` link to `.md`/`.markdown` opens in the app. Other web links still go to the system browser.
9. [ ] **History and navigation.** Back, forward and reload work across files and pages, and between pages. Reload re-fetches. Remote visits go into History under their final URL and can be reopened from there.
10. [ ] **Stale loads.** A fetch still running when the reader navigates away is ignored when it finishes.
11. [x] **HTML stripping.** Raw HTML is stripped in files and pages, in paragraphs, headings (including the title), list items, table cells and link text. Tags, comments and declarations go, quoted attributes containing `>` included. The text between them stays. Code spans, fenced code and indented code are untouched.
12. [ ] **CI.** `format`, `platform (ubuntu-22.04)`, `platform (ubuntu-24.04-arm)`, `platform (macos-latest)`, `platform (windows-latest)` and `ci` are all green (`.github/workflows/ci.yml`). The acceptance suite runs against a real HTTP server on 127.0.0.1 on all four platforms.

## User-Facing Behavior

No `/specs` artifacts exist for this phase. These scenarios are drafted here and land in `features/` test-first in the step that implements them. Existing steps (`I viewed … then …`, `loading … is slow`, `nothing was read from disk`, `I am told`) are reused where they fit.

`features/browse.feature` (step 1). This replaces "Raw HTML is shown as literal text".

```gherkin
  Scenario Outline: Raw HTML is stripped, its text kept
    Given a file "h.md" containing "<content>"
    When I enter the absolute path of "h.md"
    Then the document text is "<text>"
    Examples:
      | content                          | text              |
      | before <b>bold</b> after         | before bold after |
      | <img src=x onerror=alert(1)>tail | tail              |
      | <!-- a comment -->kept           | kept              |
      | <a title='a>b'>x</a>             | x                 |
      | <script>alert(1)</script>        | alert(1)          |
      | `<b>code</b>` stays              | <b>code</b> stays |
      | a < b and <3                     | a < b and <3      |

  Scenario: HTML in a heading is stripped from the title
    Given a file "t.md" containing "# <b>Title</b>"
    When I enter the absolute path of "t.md"
    Then I see the document titled "Title"
```

`features/remote.feature` (steps 7–9) is a new file.

```gherkin
@remote
Feature: Browse remote Markdown
  As a reader of the web of Markdown
  I want to open Markdown pages by address and follow links between sites
  So that writing published anywhere reads like one web

  # Fixture: a std-only HTTP server on 127.0.0.1:0, fresh per scenario. It records each
  # request (path, Accept, Cookie). Routes:
  #   /index.md      text/markdown   "# Remote Home", "## Intro"; links notes/a.md, /notes/b.md, #intro,
  #                                  /missing.md, https://example.com/about, mailto:a@b.co
  #   /notes/a.md    text/markdown   "# Remote Note"        /notes/b.md   text/markdown "# Note B"
  #   /post          text/markdown   "# Post Without Extension"
  #   /plain.md      text/plain      "# Plain Served"       /plain.txt    text/plain    "# Not this"
  #   /untyped.md    (no type)       "# Untyped"            /page.html    text/html     "<h1>HTML</h1>"
  #   /fake.md       text/html       "<h1>Not Markdown</h1>"
  #   /moved.md      301 → /index.md                        /loop.md      302 → /loop.md
  #   /to-file.md    302 → file:///etc/hosts                /no-location.md 302, no Location
  #   /missing.md    404   /gone.md 410   /broken.md 500
  #   /latin1.md     text/markdown; charset=iso-8859-1      /bad-utf8.md  text/markdown, invalid bytes
  #   /huge.md       body over the world's size limit (the limit is lowered for the test)
  #   /slow.md       held until released
  # Steps assert on session state, recorded effects and the requests the server saw.

  Scenario: Open a remote page by address
    When I enter the URL of "/index.md"
    Then I see the document titled "Remote Home"
    And the address shows the URL of "/index.md"
    And the site was told this reader prefers Markdown

  Scenario Outline: Which responses are Markdown
    When I enter the URL of "<path>"
    Then I see the document titled "<title>"
    Examples:
      | path        | title                    |
      | /post       | Post Without Extension   |
      | /plain.md   | Plain Served             |
      | /untyped.md | Untyped                  |

  Scenario Outline: A page that isn't Markdown can go to the system browser
    When I enter the URL of "<path>"
    Then I see the error "notMarkdown"
    When I choose to open it in the browser
    Then the system opened the URL of "<path>"
    Examples:
      | path       |
      | /page.html |
      | /plain.txt |

  Scenario: A followed link to a page that isn't Markdown offers the system browser
    Given I am viewing the URL of "/index.md"
    When I follow "/fake.md"
    Then I see the error "notMarkdown"
    When I choose to open it in the browser
    Then the system opened the URL of "/fake.md"

  Scenario: A local file that isn't Markdown offers no browser
    When I enter the absolute path of "data.txt"
    Then I see the error "notMarkdown"
    And I am not offered to open it in the browser

  Scenario: Redirects are followed and the final address is shown
    When I enter the URL of "/moved.md"
    Then I see the document titled "Remote Home"
    And the address shows the URL of "/index.md"
    And History lists the URL of "/index.md" titled "Remote Home"

  Scenario Outline: Redirects that can't be followed
    When I enter the URL of "<path>"
    Then I see the error "badRedirect"
    Examples:
      | path            |
      | /loop.md        |
      | /to-file.md     |
      | /no-location.md |

  Scenario Outline: Fetch failures
    When I enter the URL of "<path>"
    Then I see the error "<code>"
    And the error shows the URL of "<path>"
    Examples:
      | path         | code        |
      | /missing.md  | notFound    |
      | /gone.md     | notFound    |
      | /broken.md   | serverError |
      | /latin1.md   | unreadable  |
      | /bad-utf8.md | unreadable  |
      | /huge.md     | unreadable  |

  Scenario: A site that refuses the connection
    When I enter a URL whose connection is refused
    Then I see the error "unreachable"

  Scenario: Reload after a failure tries again
    Given I entered the URL of "/broken.md" and see the error "serverError"
    And "/broken.md" now serves "# Fixed"
    When I reload
    Then I see the document titled "Fixed"

  Scenario: Reloading a remote page fetches it again
    Given I am viewing the URL of "/index.md"
    When I reload
    Then I see the document titled "Remote Home"
    And the site has seen 2 requests for "/index.md"

  Scenario Outline: Links in a remote page resolve against its address
    Given I am viewing the URL of "/notes/a.md"
    When I follow "<href>"
    Then I see the document titled "<title>"
    And the address shows the URL of "<path>"
    Examples:
      | href         | title       | path        |
      | b.md         | Note B      | /notes/b.md |
      | ../index.md  | Remote Home | /index.md   |
      | /notes/b.md  | Note B      | /notes/b.md |

  Scenario: A fragment link in a remote page scrolls
    Given I am viewing the URL of "/index.md"
    When I follow "#intro"
    Then the pending fragment is "intro"
    And the address shows the URL of "/index.md"

  Scenario: A link to a missing page shows an error, and back returns
    Given I am viewing the URL of "/index.md"
    When I follow "/missing.md"
    Then I see the error "notFound"
    When I go back
    Then I see the document titled "Remote Home"

  Scenario Outline: Non-Markdown links leave the app
    Given I am viewing the URL of "/index.md"
    When I follow "<href>"
    Then the system opened "<href>"
    And I am still viewing the URL of "/index.md"
    Examples:
      | href                      |
      | https://example.com/about |
      | mailto:a@b.co             |

  Scenario Outline: A remote page can't open local files
    Given a file "secret.md" containing "# Secret"
    And I am viewing the URL of "/index.md"
    When I follow <link>
    Then I am told "Web pages can't open files on your computer."
    And I am still viewing the URL of "/index.md"
    And nothing was read from disk
    Examples:
      | link                                 |
      | the file:// URL of "secret.md"       |
      | "C:\\secret.md"                     |
      | "\\\\host\\share\\secret.md"         |

  Scenario: An absolute path in a remote page is a path on the same site
    Given I am viewing the URL of "/index.md"
    When I follow "/etc/hosts.md"
    Then I see the error "notFound"
    And the site has seen 1 request for "/etc/hosts.md"
    And nothing was read from disk

  Scenario: A local file links to a remote page
    Given a file "links.md" linking to the URL of "/index.md"
    And I am viewing "links.md"
    When I follow the URL of "/index.md"
    Then I see the document titled "Remote Home"

  Scenario: A local file's non-Markdown web link goes to the system browser
    Given a file "web.md" linking to "https://example.com/"
    And I am viewing "web.md"
    When I follow "https://example.com/"
    Then the system opened "https://example.com/"

  Scenario: Back and forward between files and pages
    Given a file "links.md" linking to the URL of "/index.md"
    And I viewed "links.md" then the URL of "/index.md"
    When I go back
    Then the address shows the absolute path of "links.md"
    When I go forward
    Then I see the document titled "Remote Home"

  Scenario: Back and forward between pages
    Given I viewed the URL of "/index.md" then the URL of "/notes/a.md"
    When I go back
    Then I see the document titled "Remote Home"
    When I go forward
    Then I see the document titled "Remote Note"

  Scenario: Reopen a remote visit from History
    Given I viewed the URL of "/index.md" then the URL of "/notes/a.md"
    When I open the URL of "/index.md" from History
    Then I see the document titled "Remote Home"

  Scenario: A slow fetch doesn't replace a newer page
    Given loading the URL of "/slow.md" is slow
    When I enter the URL of "/slow.md"
    And I enter the URL of "/index.md"
    And "/slow.md" finishes loading
    Then I see the document titled "Remote Home"
```

## Design notes

- **`location.rs` is split before growing** (it is 281 lines now):
  - `session/error.rs` holds `ErrorCode`.
  - `session/address.rs` holds `parse_address`, home expansion and the file-URL and path helpers.
  - `session/location.rs` keeps `Location`, `Target` and one shared `is_markdown_name(&str)` used by paths, URLs and links.
- **`Location`** is an enum, `File(PathBuf) | Web(Url)`. The `Url` has no fragment. `address()` gives the path or the full URL. `label()` gives the file name, the last path segment, or the host. The runner matches on it exhaustively, with no wildcard, so a web location can never reach `files`.
- **`url` crate (WHATWG)** handles parsing and `join`. It is a pure dependency of `session`, recorded in ARCHITECTURE.md in the same step it is added.
- **`session::web`** is pure and owns every response decision:
  - `ACCEPT`
  - `classify_status(u16)`: ok, redirect, not found, or server error
  - `next_hop(from, location_header, hops) -> Result<Url, ErrorCode>`
  - `body_policy(content_type, final_url) -> Result<(), ErrorCode>`, covering type and charset
- **`services::web::fetch(url, Limits)`** only does I/O. It uses ureq 3 with rustls and webpki roots, sets `max_redirects(0)`, and has no cookie store. Its loop is a fold over `next_hop`. It reads at most `limit + 1` bytes and decodes strict UTF-8. `Limits` carries the timeouts (10 s connect, 30 s overall) and the size cap (`files::MAX_BYTES`). Tests override them.
- **`Event::Loaded.result`** becomes `Result<Loaded { source, location }, ErrorCode>`. A pure `Navigation::settle(location)` swaps the current entry's location for the final one, keeping the entry's fragment. History records the final address.
- **New `ErrorCode`s** are `ServerError`, `Unreachable` and `BadRedirect`. Their keys and strings arrive in the same step as the codes, with an exhaustive content test.
- **New `NoticeKind`**: `LocalFromWeb`, with the string "Web pages can't open files on your computer."
- **Loading** already shows `content::LOADING` in the body. For pages it becomes "Loading example.com…". The address bar keeps the typed text on failure (as it does today), and Reload retries from an error.
- **Shared test server**: `tests/support/http.rs`, std-only, about 120 lines. The acceptance world includes it with `mod`, and `services::web`'s unit tests include it with `#[cfg(test)] #[path = "../../tests/support/http.rs"]`. This avoids a test-only public API in the library and the server itself adds no dev-dependency (the gzip test in step 6 uses flate2).

## Steps

### Step 1: Strip raw HTML (ADR-002), PR A — done

**Complexity**: standard
**RED**: Add the two browse scenarios above, plus the step "the document text is". Model unit tests:
- inline tags, comments, declarations, CDATA and processing instructions are removed and text is kept
- quoted attributes containing `>` are handled
- an HTML block keeps its text across lines
- a block that strips to nothing produces no block
- non-tags are kept (`a < b`, `<3`)
- HTML in a heading, list item, table cell or link text is stripped
- code spans, fenced code and indented code keep `<b>`

Replace `html_is_shown_as_literal_text_not_interpreted`. **Before GREEN**, list the golden snapshots expected to change. `grep '<'` finds HTML only in `typography.md`, `tables-and-code.md` and `spec.md`. Any other snapshot diff is a failure.
**GREEN**: Add a pure `model::html::strip(&str) -> String`, a scanner over CommonMark's raw-HTML token shapes. Apply it to `Event::InlineHtml` in `inlines.rs` and to `HtmlBlock` text in `blocks.rs`.
**REFACTOR**: Update the comments in `inlines.rs`/`blocks.rs`.
**Files**: `src/model/html.rs` (new), `src/model/{mod,inlines,blocks}.rs`, `features/browse.feature`, `tests/acceptance/steps.rs`, the listed snapshots
**Commit**: `feat(model): strip raw HTML and keep the text between tags`

### Step 2: ADR-003, the HTTP client and URL handling, plus doctrine — done

**Complexity**: trivial
**GREEN**: Use `invest-adr` to record ureq 3 (blocking, which fits `background::run_blocking`; rustls with webpki roots, so no OpenSSL anywhere) and `url`. The alternatives are reqwest with tokio (a second runtime alongside iced's) and hand-rolled HTTP (no TLS). Costs to record: about 2 MB of binary, and corporate MITM proxies fail as `unreachable`. Also record the accepted **private-network risk**: a page can link to `http://192.168.x.x/…`, which is fetched only on a click, with no cookies. It is tracked as a follow-up.

In ARCHITECTURE.md:
- add `ureq` and `url` to the stack table
- the Session row and import rule: `session` may use `url`
- the Services row: `web` fetches over http(s), and `std::net` is reached through ureq only from `services`
- the pipeline's Fetch line
**Files**: `vector/decisions/ADR-003-http-client.md`, `ARCHITECTURE.md`
**Commit**: `docs(vector): choose ureq and url for remote documents (ADR-003)`

### Step 3: Split `location.rs`, then add error codes — done

**Complexity**: standard
**RED**: A content test: every `ErrorCode` has a non-empty, distinct string, and so does every `NoticeKind`. It is iterated over an `ALL` list that a match keeps exhaustive. A unit test for `is_markdown_name`: `a.md` and `A.MARKDOWN` are accepted, case-insensitively. `a.md?x` and `a.txt` are rejected, because callers pass bare path names with any query already removed.
**GREEN**:
- Move `ErrorCode` to `error.rs` and the parsing to `address.rs`, re-exporting so callers don't change.
- Add `ServerError`, `Unreachable` and `BadRedirect` with their keys, and `NoticeKind::LocalFromWeb`.
- Add the strings, with what to try next:
  - `serverError`: "The site had a problem sending this page. Try again later."
  - `unreachable`: "Couldn't reach this site. Check your connection, then reload."
  - `badRedirect`: "This site redirected somewhere that can't be followed safely. Try opening the address in your web browser."
  - `notMarkdown` (pages): "This page isn't Markdown. You can open it in your web browser instead." (reworded in step 9)
  - Pages only:
    - `notFound`: "There's no page at this address. Check it for typos."
    - `unreadable`: "This page couldn't be read as text. It may be too large or not in UTF-8."
  - The web strings come from `content::web_error(code)`. The content test asserts that every code has both a message and a hint.
**REFACTOR**: None needed (it is a move).
**Files**: `src/session/{error,address,location,mod}.rs`, `src/content.rs`
**Commit**: `refactor(session): split addresses from locations and add web error codes`

### Step 4: `Location` can be a web URL — done

**Complexity**: standard
**RED**: Unit tests:
- `Location::web` drops the fragment
- `address()` shows the full URL with its scheme
- `label()` gives the last segment, or the host for `/`
- equality ignores the fragment

The existing file tests still pass.
**GREEN**: Make `Location` an enum and add `url`. `path()` becomes `as_file() -> Option<&Path>`. The runner's `match` is exhaustive, and its `Web` arm returns `Err(Unreachable)` until step 6. Nothing creates a web location yet.
**Files**: `Cargo.toml`, `src/session/location.rs`, `src/session/links.rs`, `src/services/runner.rs`, `src/ui/*` call sites
**Commit**: `refactor(session): let a location be a file or a web URL`

### Step 5: Web response policy, pure — done

**Complexity**: complex (security-sensitive)
**RED**: Unit tests in `session::web`:
- **Types.** `text/markdown; charset=utf-8`, `Text/Markdown`, `text/x-markdown`, `text/plain ; charset=UTF-8` with a `.md` path, `text/plain` with `/a.md?v=1`, `.markdown`, and no type with `.MD` are all Markdown. `text/plain` with `.txt`, `text/html` and `application/json` are `notMarkdown`.
- **Charset.** `utf-8`, `"UTF-8"` and `us-ascii` pass. `iso-8859-1` is `unreadable`.
- **Status.** Any 2xx is ok (200, 203, 204 and 206 are tested). 301, 302, 303, 307 and 308 are redirects. 300 and 304 are `serverError`. 404 and 410 are `notFound`. 403 and 500 are `serverError`.
- **Hops.** A relative `Location` resolves against the request URL. http→https and http→http are allowed. These are `badRedirect`: https→http, `file:`, `ftp:`, `javascript:`, and a missing `Location`. Hop 5 is allowed and hop 6 is `badRedirect`. Credentials in the target are stripped.
- **`ACCEPT`.** `text/markdown` has the highest q.
**GREEN**: Implement `session::web`.
**Files**: `src/session/web.rs` (new), `src/session/mod.rs`
**Commit**: `feat(session): decide which web responses are Markdown`

### Step 6: Fetch service — done

**Complexity**: complex (I/O, network, security)
**RED**: Add `tests/support/http.rs`. Tests in `services::web` against it:
- the `Accept` header is sent, and `source` comes back with the final URL
- it follows a 301 and reports the new URL; a loop gives `badRedirect` after 6 requests
- a 302 with no `Location` gives `badRedirect`
- 404 and 410 give `notFound`; 500 gives `serverError`
- with a small test limit, an over-limit body is `unreadable`
- invalid UTF-8 and `charset=iso-8859-1` are `unreadable`
- `text/html` is `notMarkdown`; a missing type with `.md` is ok
- a silent server times out as `unreachable`, using a short test timeout
- a refused connection is `unreachable`: bind port 0, drop it, connect at once
- **cookies:** the first response sets `Set-Cookie` and redirects, and the second request carries no `Cookie`
- **from the step 5 security review:**
  - a gzip body that inflates past the test limit is `unreadable`, because the cap applies after decompression
  - a typed `https://user:pw@host/` sends no `Authorization` to a redirect target on another host; every hop is a fresh request carrying only `Accept`
  - a `Location` header that isn't valid UTF-8 is `badRedirect`
**GREEN**: Implement `fetch` with ureq. It does I/O only and calls `session::web` for every decision.
- The hop number comes from the loop itself (`1..=MAX_REDIRECTS + 1`), so it can't be skipped.
- Bytes are decoded with `String::from_utf8`, never unchecked; a UTF-8 BOM is dropped.
**REFACTOR**: Share `MAX_BYTES` with `files`.
**Files**: `Cargo.toml` (ureq; flate2 as a dev-dependency for the gzip test, already in the tree through ureq), `src/services/{web,mod,runner}.rs`, `src/session/web.rs` (`decode`), `tests/support/http.rs`
**Commit**: `feat(services): fetch Markdown over http(s)`

### Step 7: Typed web addresses load — done

**Complexity**: complex (parse → reducer → runner → acceptance)
**RED**:
- **`parse_address`.** `http(s)` is accepted whatever the extension. `HTTP://X.ORG/a` and a fragment are accepted. `http://` with no host is `notAbsolute`. Userinfo is stripped. `ftp:` is still `unsupportedScheme`.
- **Existing test.** Rewrite the `location.rs` test that expects `https://example.com/readme.md` to be `UnsupportedScheme`.
- **Reducer.**
  - `Loaded` with a different final location settles the navigation entry, records History under the final address, and keeps the typed fragment.
  - A redirect to an address already in History moves that visit to the top. It doesn't duplicate it.
  - A stale web load is ignored.
  - `Loading` is set for web loads.
- **Acceptance.** Add `features/remote.feature` with the server world and these scenarios: "Open a remote page", "Which responses are Markdown", "Redirects…" (both), "Fetch failures", "A site that refuses the connection", "Reload after a failure", "Reloading a remote page", "Reopen a remote visit from History" and "A slow fetch…".
**GREEN**:
- Change `parse_address`, `Event::Loaded`/`Loaded` and `Navigation::settle`.
- The runner dispatches to `files` or `web`.
- Show "Loading <host>…" for pages.
- Wire the acceptance world: routes, request log, held routes, and size limit.
- New step definitions:
  - `I enter the URL of`, `I am viewing the URL of`, and `I viewed … then the URL of` (all forms)
  - `the address shows the URL of`, `the error shows the URL of`, `History lists the URL of … titled`
  - `the site was told this reader prefers Markdown` (checks that the q-values rank Markdown first)
  - `the site has seen N request(s) for`
  - `I entered the URL of … and see the error`, `… now serves …`
  - `I enter a URL whose connection is refused`
  - `loading the URL of … is slow`, `… finishes loading`
  - `I open the URL of … from History`
- Steps 8 and 9 add `I follow the URL of`, `I follow the file:// URL of`, `a file … linking to …`, `I choose to open it in the browser`, `I am not offered to open it in the browser` and `the system opened the URL of`.
**REFACTOR**: Split `steps.rs` into `steps/{local,remote}.rs` if it passes about 300 lines.
**Files**: `src/session/{address,reducer,navigation,mod}.rs`, `src/services/runner.rs`, `src/content.rs`, `src/ui/app.rs`, `tests/acceptance/*`, `features/remote.feature`
**Commit**: `feat(session): open Markdown by http(s) address`

### Step 8: Links across sites — done

**Complexity**: complex (security-sensitive)
**RED**: `links::classify` unit tests.

With a web base:
- these navigate on the web: `b.md`, `../index.md`, `/abs.md`, `//other.host/p.md`, `HTTP://X/A.MD`, and `https://x/a.md?x=1#f` (the fragment is kept)
- `page.html` and `https://e.com/about` are External; `mailto:` is External
- `#x` scrolls
- these are LocalFromWeb: `file:///etc/x.md`, `C:\x.md` and `\\host\x.md`
- `javascript:`, `data:`, `ftp:` and `""` are Unsupported

There is also a property-style test: about 40 assorted hrefs from a web base never yield a `File` location.

With a file base:
- `https://e.com/p.md` navigates on the web
- `https://e.com/` is External
- `//host/x.md` stays Unsupported (unchanged)

Rewrite `only_web_and_mail_links_leave_the_app`. Acceptance: the remaining `remote.feature` link scenarios.
**GREEN**: Split `classify` by base kind. File bases keep `local_target`, and `http(s)` `.md` links navigate. Web bases use `Url::join` and the extension rule on the resolved path. Disk-shaped references are `LocalFromWeb`. The reducer maps `LocalFromWeb` to its notice.
**Files**: `src/session/{links,reducer,mod}.rs`, `features/remote.feature`, `tests/acceptance/steps*`
**Commit**: `feat(session): follow Markdown links between sites`

### Step 9: Open in browser — done

**Complexity**: standard
**RED**: Reducer tests. `OpenInBrowser` emits `OpenExternal(url)` only for a web location in `Failed(NotMarkdown)`. These emit nothing: `Failed(NotFound)` on a web location, `NotMarkdown` on a file, `Ready` and `Loading`. Acceptance: "A page that isn't Markdown can go to the system browser" and "A local file that isn't Markdown offers no browser".
**GREEN**: Add `Event::OpenInBrowser`. The error view gets a button using the toolbar's button style, with its label in `content`. For pages, `notMarkdown` reads "This page isn't Markdown. You can open it in your web browser instead."
**Files**: `src/session/{mod,reducer}.rs`, `src/content.rs`, `src/ui/body.rs`, `tests/acceptance/steps*`. Also, found while checking by hand: `services::files::from_command_line` turned a URL argument into a path; it now passes anything with a scheme through (`session::address::has_scheme`).
**Commit**: `feat(ui): offer the system browser for pages that aren't Markdown`

### Step 10: Docs and doctrine — done

**Complexity**: trivial
**GREEN**:
- **README:** what it does today, plus a remote address to try (`https://andrewshell.org/` serves Markdown when asked).
- **VECTOR.md:**
  - Direction: phase 2 explicitly split, with 2a built and 2b (images) next.
  - The phase-2 constraint line.
  - The remote-images open question tagged to 2b.
  - Research status.
- **`features/README.md`:** the server fixture.
**Files**: `README.md`, `VECTOR.md`, `features/README.md`
**Commit**: `docs: describe remote browsing`

## Pre-PR Quality Gate (each PR)

- [ ] `cargo test` passes (unit and acceptance)
- [ ] `cargo build --release` passes
- [ ] `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass
- [ ] `/code-review` passes, including `arch-review` (session may use `url`, and `services::web` matches ARCHITECTURE.md) and `a11y-review` on the new button
- [ ] Manual check (PR B): open `https://raw.githubusercontent.com/scripting/markdownWeb/main/readme.md` and `https://andrewshell.org/`, and follow a link from one page to another
- [ ] All six CI checks are green
- [ ] Documentation is updated

## Risks & Open Questions

- **Real Content-Types**, checked 2026-10-29 with `curl` and a Markdown `Accept`:
  - raw GitHub serves `text/plain; charset=utf-8` (covered by the `.md` rule)
  - `andrewshell.org/` negotiates `text/markdown` with `Vary: Accept` (covered, and it confirms that typed addresses without an extension are worth trying)
  - github.com blob pages are `text/html` (correctly `notMarkdown`, with Open in browser)
  - scripting.com served HTML at `/` and reset the connection on a guessed `.md` path

  `application/octet-stream` is not accepted. If the manual check finds real pages served that way, widening the rule is a deliberate, test-first change.
- **Private-network fetches.** Narrowed in step 8 at the product owner's call: a public page can't open local-network addresses in the app, by link or redirect. Local files still may, and DNS rebinding is not caught (ADR-003, a follow-up).
- **Plain `http`.** The full URL with its scheme is always in the address bar, but there is no "not secure" label yet. That label is a candidate follow-up, tracked in VECTOR open questions in step 10.
- **No cache.** Back and forward re-fetch, so the reader sees "Loading…" briefly, and scroll position resets to the fragment or the top. That is fine for blog-sized posts.
- **No stop button.** Navigating away makes a fetch stale, and the overall timeout is 30 s.
- **Keyboard access.** iced buttons, including the existing toolbar ones, aren't Tab-focusable today. The new button matches the existing pattern, and `a11y-review` will flag the app-wide gap. Fixing it is separate work, not 2a.
- **The link's destination isn't shown before clicking** (in the app or in the browser). That is phase 4 or later.
- **Addresses without a scheme** (`example.com/x.md`) stay `notAbsolute`. Auto-prefixing `https://` is a possible small follow-up.
- **Script and style text** stays visible after stripping (ADR-002, accepted).
- **Certificate failures look like outages** (step 6 security review). Every failure before a response, a bad certificate included, is `unreachable`. A separate `insecure` error would make a MITM visible; it is a follow-up, beside ADR-003's corporate-proxy note.
- **Slow DNS leaves a thread behind.** ureq resolves on its own thread when a global timeout is set, so a stuck lookup outlives the fetch until the system resolver gives up. Bounded, and irrelevant at one fetch per click.
- **Windows CI and loopback servers.** A refused connection is asserted with a bind-and-drop port. Timeouts are unit-tested only, with short test limits.

## Plan Review Summary

Four personas reviewed revision 1. The two with blockers re-reviewed revision 2. Revision 3 applies the remaining acceptance findings, which were mechanical fixes. Two iterations is the limit, so no third run.

| Reviewer | Revision 1 | Final |
|---|---|---|
| Strategic | approve (5 warnings) | approve |
| UX | needs-revision (6 warnings) | addressed in revision 2 |
| Design | needs-revision (2 blockers) | approve (2 small warnings, both fixed) |
| Acceptance | needs-revision (7 blockers) | needs-revision (1 blocker), fixed in revision 3 |

**Changes made in response:**
- **Strategic**: step 1 now ships alone as PR A. Real Content-Types were checked with `curl` before step 5 (see Risks). VECTOR gets an explicit 2a/2b split, with the remote-images question tagged 2b. The private-network risk moved into ADR-003. The address bar always shows the scheme.
- **UX**:
  - errors carry the address plus a hint
  - "Loading <host>…"
  - reload after a failure
  - a distinct "Web pages can't open files on your computer." notice
  - a terms glossary
  - a11y-review in the gate
- **Design**:
  - `location.rs` is split, and the error codes arrive before first use
  - all response policy is in `session::web`
  - the shared test server goes through `#[path]`
  - doctrine edits come with ADR-003
  - the runner match is exhaustive
  - a property test checks that a web base never yields a file
  - `Navigation::settle` is pure
- **Acceptance**:
  - redirect failure, 410, oversize, non-UTF-8, reload, History, stale-fetch, fragment, broken-link-and-back, local web link, negative Open-in-browser and followed-HTML scenarios
  - the absolute-path contradiction fixed: such paths are site paths, with their own scenario
  - redirect statuses named, and `badRedirect` added
  - CI checks named
  - new step definitions listed

**Still open (accepted):**
- Step 7 is the widest commit. It may be split into a pure part and a wiring part during `/build`.
- The app-wide keyboard-focus gap for iced buttons is separate work.
- The link destination isn't shown before clicking. That is phase 4 or later.
