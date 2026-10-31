# ADR-005: Look for a Markdown version when a page isn't Markdown, never substitute silently

**Date:** 2026-10-29
**Status:** accepted
**Deciders:** gBRETT (product owner)

## Context

Writers often link to the HTML page (`/post`) rather than the file (`/post.md`), so today those addresses end in "This page isn't Markdown." Phase 2d (`vector/plans/phase-2d-markdown-fallbacks.md`) looks for a Markdown version before giving up. Content negotiation through `Accept` already happens (ADR-003). The open questions are what to try, in what order, which URL the reader then sees, and how a substitution is made visible. Doing nothing leaves most real-world links dead.

## Decision Drivers

- VECTOR.md: no HTML in the rendering path, so no reading of an HTML body to find alternates.
- VECTOR.md Design Principle 4: interop before invention. `Link: rel="alternate"` and the `.md` sibling convention already exist.
- A public page must not steer a fetch into the reader's own network (`Origin`, ADR-003).
- The address bar and History must name something that reloads to what is on screen.
- The reader must be able to tell that the content isn't what the link pointed at.

## Options Considered

### Option A: Do nothing

Keep the "not Markdown" error and the Open in browser action.

**Pros:**
- No new requests or rules.

**Cons:**
- Most links to blog posts on Markdown-publishing sites stay dead.

### Option B: Try alternates in a fixed order, tell the reader

On a successful response whose type isn't Markdown: (1) the `Link: <…>; rel="alternate"; type="text/markdown"` header on that response, same scheme, host and port only; (2) the `.md` sibling (`/post` → `/post.md`; with a trailing slash, `/post.md` then `/post/index.md`). First hit wins, with one level only. The shown URL goes in the address bar and History, and a notice names the original address on every such load. `llms.txt` is offered as a button on the error screen when nothing is found and the site answers for `/llms.txt` (probed once, on the address the reader asked for, after the fallbacks miss, under the same checks as any page). The whole search shares one deadline.

**Pros:**
- Uses existing conventions, with at most two extra requests, and only after a type mismatch.
- The address always reloads to what is shown.
- Nothing is substituted without saying so.

**Cons:**
- A sibling guess can hit an unrelated `.md` file at that path. The notice and the original-address link are the check.
- The notice appears every time, even for sites the reader trusts.

### Option C: Also scan the HTML for `<link rel="alternate">`, or fall back to `llms.txt` automatically

**Pros:**
- Finds more Markdown versions.

**Cons:**
- Reading HTML conflicts with VECTOR.md's constraint, and needs its own decision.
- `llms.txt` is a site index, not the page the reader asked for, so showing it automatically misleads.

## Decision

**We will adopt Option B.**

The trigger is only a type mismatch on a success response. A 404, a timeout or a bad body never starts a fallback, so failures keep their meaning. Fallbacks use the original `Origin`, the same limits and the same type checks, and never chain. A `Link` alternate on another host is ignored, so a page can't redirect the reader's fetch elsewhere. The reader's address bar and History hold the Markdown URL actually shown. Option C is left for later: HTML scanning needs an ADR of its own, and `llms.txt` stays an explicit choice.

**Amended 2026-10-31.** The notice naming the original address is gone: after a successful fallback the address bar already says where the reader is, and "this site doesn't serve Markdown at …" was inaccurate. The session still records the address asked for, and Ctrl/Cmd+Shift+O opens it in the system browser. The same fallback now runs for links followed from a page, not only for typed addresses, so a same-site link such as `/post` finds `/post.md` and `/llms.txt` opens in the app. A page with no Markdown version still ends in the error and its Open in browser button.

## Consequences

**Positive:**
- Links to HTML pages on sites that publish Markdown open as Markdown.
- The fallback rules are pure functions in `session::web`, testable without a server.

**Negative:**
- A site that serves Markdown only through `<link rel="alternate">` in its HTML is not covered yet.
- A failed type mismatch costs up to five requests before the error appears: the page, the `Link` alternate, two siblings and the `llms.txt` probe, all within one shared deadline.
- Back, Reload and History use the Markdown URL, so the original HTML address isn't kept except for Open in browser (Ctrl/Cmd+Shift+O).

**Neutral:**
- No setting to turn fallbacks off yet. If it's wanted, it belongs in the Settings panel.

## Related Decisions

- ADR-002: Strip raw HTML from documents. HTML scanning is left out for the same reason.
- ADR-003: Fetch remote documents with ureq and handle URLs with the url crate. Fallbacks use its fetch loop, limits and `Origin`.
