# ADR-003: Fetch remote documents with ureq and handle URLs with the url crate

**Date:** 2026-10-29
**Status:** accepted
**Deciders:** gBRETT (product owner)

## Context

Phase 2a adds remote Markdown: a typed `http(s)` address is fetched with an `Accept` header that prefers Markdown, and `.md` links inside a remote page open in the app (plan: `vector/plans/phase-2a-remote-documents.md`). The crate has no HTTP client, no TLS and no URL type yet. Relative links on a remote page must resolve against the page's URL exactly as browsers do, or links between Markdown pages on different sites (a ship criterion) break. Choosing a client also chooses how blocking work, TLS and redirects fit the existing layers.

## Decision Drivers

- `services` is the only layer that does I/O. Blocking work already runs on its own thread through `services::background::run_blocking`, and iced owns the async runtime.
- No HTML, DOM or webview anywhere (VECTOR.md constraint). The client must stay a plain HTTP fetcher, with no cookies or browser state.
- The release build must work on Linux x64, Linux ARM64, macOS and Windows in CI with no system libraries to install, so no OpenSSL.
- Redirect policy is product behaviour (cross-scheme rules, the final URL shown in the address bar). It belongs in pure `session` code where it can be tested, not inside a client's defaults.
- Relative-reference resolution must follow the WHATWG URL standard, because writers test their links in web browsers.
- Remote input is untrusted: every fetch needs time limits and a size cap, like the 16 MiB cap `services::files` already applies.

## Options Considered

### Option A: ureq 3 with rustls, plus the url crate

ureq is a blocking HTTP/1.1 client. Its default features are rustls with the ring provider and Mozilla's webpki roots, and gzip. `url` is the WHATWG URL implementation from the Servo project.

**Pros:**
- Blocking calls drop straight into `background::run_blocking`. There is no second runtime.
- Pure-Rust TLS with bundled roots, so all four CI platforms build with nothing extra installed.
- Redirects can be switched off (`max_redirects(0)`), so `session::web::next_hop` decides every hop. Cookies are an opt-in feature that we leave off.
- Connect and overall timeouts, and a body-size limit, are per-request settings.
- `url` gives standard-conforming joins, and `session` can use it without doing I/O.

**Cons:**
- Adds about 2 MB to the release binary (rustls, ring, webpki roots).
- Bundled roots ignore the OS trust store. Behind a corporate MITM proxy with a private root, every fetch fails and the reader sees `unreachable`.
- HTTP/1.1 only. That is fine for fetching documents.

### Option B: reqwest with tokio

The most widely used Rust HTTP client, async on tokio.

**Pros:**
- Mature and full-featured: HTTP/2, the system trust store via native-tls, proxy support.

**Cons:**
- Brings a tokio runtime alongside the executor iced already runs, or needs its blocking feature, which starts a hidden runtime anyway.
- A much larger dependency tree and binary for features a document fetcher doesn't use.
- Its defaults (following redirects, optional cookie store) have to be turned off to keep the policy in `session`.

### Option C: Hand-rolled HTTP over std::net

Write a minimal HTTP/1.1 client ourselves.

**Pros:**
- No new dependency.

**Cons:**
- No TLS, and almost all the web of Markdown is `https`. Writing TLS ourselves is out of the question, so this only moves the dependency choice elsewhere.
- Chunked encoding, compression and edge cases in status lines all become our bugs.

### Option D: Do nothing (local files only)

**Pros:**
- No new risk surface.

**Cons:**
- Fails VECTOR.md's value proposition and the first-release ship criterion (browsing `http(s)` Markdown).

## Decision

**We will fetch with ureq 3 (default rustls features, no cookies) and represent and resolve URLs with the url crate.**

ureq's blocking model fits the existing `run_blocking` service with no second runtime. Its TLS needs no system libraries on any CI platform, and with redirects off and cookies absent the client stays a plain fetcher, so all policy lives in pure `session` code. Option B costs a second runtime and a heavier tree for features we don't need. Options C and D fail the drivers outright.

Fetch settings, all owned by `services::web` and passed as `Limits`:
- `max_redirects(0)`. `session::web::next_hop` follows hops, up to its own limit.
- 10 s to connect and 30 s overall.
- Bodies are capped at 16 MiB, counted after gzip decompression. A body in an encoding the client didn't decode is unreadable.
- No cookie store.
- `Accept` comes from `session::web::ACCEPT` and prefers `text/markdown`.

## Consequences

**Positive:**
- Remote documents and links between Markdown sites become possible (phase 2a).
- Redirect, status and body rules are pure functions with unit tests. Only the byte transfer is I/O.
- One URL type for addresses, links, History and the address bar.

**Negative:**
- About 2 MB more binary.
- Corporate MITM proxies with private roots fail as `unreachable`. If readers report this, revisit it with ureq's `platform-verifier` feature (the OS trust store).
- **Private-network fetches, narrowed (amended 2026-10-29).** A public page can no longer open a loopback, private or link-local address, or `localhost`, in the app. The link is refused with "Web pages can't open addresses on your local network." The original mitigation was "visible before the click", and it failed: the hover shows the raw link, and spellings like `//2130706433/x.md` or `//[::ffff:127.0.0.1]/x.md` are 127.0.0.1 in disguise. Such links that aren't Markdown still go to the system browser, which applies its own rules. Pages that are themselves on the local network may link within it. A public *name* that resolves to a private address (DNS rebinding) is still not caught, and stays a follow-up.
- **Accepted risk: links in local files.** A local file counts as local, so it may link into the local network. A downloaded README can link to `http://127.0.0.1:…/x.md`, and clicking it fetches that page in the app. Links, unlike redirects, may also go from an `https` page to an `http` one. The full URL, with its scheme, is shown in the address bar once the page loads.
- **Redirects follow the same rule.** `next_hop` refuses a redirect from a public host to a loopback, private, link-local or unspecified address, or to `localhost`. Links and redirects share one check of the parsed host (`session::web::is_local_network`), so `0x7f.1` and `[::ffff:127.0.0.1]` count. DNS rebinding needs a check at connect time, which belongs to the same follow-up.
- No HTTP/2. No proxy support beyond what ureq reads from the environment.

**Neutral:**
- ARCHITECTURE.md:
  - the stack table lists ureq and url
  - `session` may use `url`
  - `services::web` is the only code that reaches the network
- The ring crypto provider builds from C and assembly with the platform compiler that the Rust toolchain already needs. There is no cmake or OpenSSL.

## Related Decisions

- ADR-001: Render Markdown natively with our own layout engine. The fetcher brings in bytes only, and HTML responses are never rendered.
- ADR-002: Strip raw HTML from documents, keeping the text between tags. Remote documents go through the same model.
