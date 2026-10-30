# Plan: Phase 2d: Markdown fallbacks

**Created**: 2026-10-29
**Branch**: `feat/phase-2d`, from `main` after phase 2c (#10)
**Status**: built (2026-10-29); decisions in ADR-005

## Goal

Many sites that publish Markdown link to the HTML page (`/post`), not the file (`/post.md`). Today that address fails with "This page isn't Markdown." 2d looks for a Markdown version before giving up, and says so when it finds one. Example to test against: `every.to/context-window/…` and the same URL with `.md`.

## Where it fits today

- `services::web::fetch` asks with `Accept: text/markdown, …` and `session::web::body_policy` refuses anything that isn't Markdown (`FetchError::WrongType` → `ErrorCode::NotMarkdown`). Content negotiation is already the first fallback.
- Redirects are decided in pure `session::web::next_hop`; `Origin` (Trusted/Public) guards the local network.
- A fallback only starts from `WrongType`. A 404, a timeout or a bad body never triggers one.

## Proposed rules (recommendation)

1. **Trigger**: the response is a success whose type isn't Markdown (in practice `text/html`). Nothing else.
2. **Order**, first hit wins, one level only (a fallback never starts another):
   1. `Link: <…>; rel="alternate"; type="text/markdown"` on that response. Costs no request to discover.
   2. The `.md` sibling: `/post` → `/post.md`, and `/post/` → `/post.md`, then `/post/index.md`. At most 2 extra requests. A candidate that duplicates the page or another candidate is skipped.
3. **Not in 2d**: scanning the HTML body for `<link rel="alternate" type="text/markdown">`. It means reading HTML, which VECTOR.md rules out of the rendering path, so it needs its own ADR and is left for later.
4. **`llms.txt`** is never substituted. When every fallback misses, the "not Markdown" screen offers "Try this site's llms.txt" as an explicit button, only when the site has one (one extra probe of `/llms.txt`, made only after both fallbacks miss, read under the same type rules a click would use, so `text/plain` is accepted at that path), which opens `/llms.txt` as a normal navigation (the site root, same scheme and host).
5. **Safety**: a fallback URL must be the same scheme, host and port as the page, or a host `next_hop` would allow, and it is fetched with the original `Origin` (as Public if a redirect moved the page to another host), so a public page can't steer into the local network. Same size and type limits as any page; the page, its candidates and the `llms.txt` probe (at most 5 requests) share one time limit.
6. **What the reader sees**: the address bar and History hold the Markdown URL actually shown, since that is the one that reloads. A notice reads "Showing the Markdown version of <original address>" with the existing "Open in browser" action pointed at the original. Back returns to the page before, not to the HTML address.

## Decisions (2026-10-29)

See ADR-005: `Link` alternates on the same host only; `/post.md` then `/post/index.md` for a trailing slash; the address bar and History hold the Markdown URL; the notice shows every time.

## Acceptance Criteria

1. [x] An HTML page whose response has a `Link` alternate for Markdown opens that Markdown.
2. [x] An HTML page with a `.md` sibling opens it, in the order above.
3. [x] With no Markdown version, the error is unchanged, plus the `llms.txt` button when the site has an `llms.txt`.
4. [x] A notice names the original address, and the address bar and History hold the shown URL.
5. [x] Fallbacks never chain, never leave the host rule, and respect `Origin`.
6. [x] Local files and non-HTML failures (404, timeouts) never trigger a fallback.
7. [x] Strings in `content`, features in `features/`, all gates green.

## Steps

1. ~~ADR-005~~ done: `vector/decisions/ADR-005-markdown-fallbacks.md`.
2. Capture response headers on a `WrongType` result (`Link` parsing as a pure function in `session::web`).
3. Pure `candidates(url, link_header) -> Vec<Url>` in `session::web`, test first.
4. Fetch loop in `services::web`: try candidates after `WrongType`, return the URL found.
5. Reducer: notice and address handling. `features/fallbacks.feature`.
6. `llms.txt` button on the error screen.
7. Docs, `/code-review`, `/pr`.
