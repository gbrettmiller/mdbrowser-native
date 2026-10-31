# ADR-010: Web links open in the app; only mail leaves, after the reader agrees

**Date:** 2026-10-31
**Status:** accepted
**Deciders:** gBRETT (product owner)

> Numbered 010 because the release plan (`vector/plans/release-distribution.md`) reserves ADR-009 for the distribution approach.

## Context

Until PR #17 a link followed from a page went to the system browser unless its address looked like Markdown (`.md`, `.markdown`). That was wrong in two ways found by using the app on `longform.asmartbear.com`. A fallback page's link to `/llms.txt` left the app, and a same-site link such as `/fail` left the app instead of falling back to `/fail/index.md`. ADR-005 gave typed addresses a Markdown fallback, but a followed link never reached it, because the extension test sent it away first. The reader also got no warning that a click was about to open another program. Doing nothing leaves links on sites that serve Markdown at extensionless addresses (the common case in the Web of Markdown) dead.

## Decision Drivers

- A link is an address the reader chose to visit, the same as typing it. Typed addresses already try the app first, with the Markdown fallback (ADR-005).
- Principle 4, interop before invention: sites in the community publish Markdown at extensionless addresses and at `/index.md`, and expect links between them to work.
- Principle 5, writing and reading only: leaving for another program should be rare, visible and the reader's call.
- ADR-003 stays in force: no cookies, a fixed `Accept` header, no `Referer`, and a public page can't reach the local network, a file path or a loopback address. These guards sit on the fetch and on link classification, not on whether the link is in the app.
- The system opener must stay a narrow door: only `http`, `https` and `mailto`, no shell (`services::opener`).

## Options Considered

### Option A: Same host in the app, everything else to the system browser

Links within the page's own site try the app and its fallbacks. Links to other sites go to the system browser, as before.

**Pros:**
- A cross-site link never sends a request from the app to a site the reader has not heard of.
- Smallest change from the previous behaviour.

**Cons:**
- The Web of Markdown is made of links between sites, so the most useful links stay dead.
- Clicking a link would behave differently from typing the same address, which the reader cannot predict.
- The privacy gain is small: the request carries no cookies and no `Referer`, and the system browser would send its own cookies.

### Option B: Every web link opens in the app; mail leaves, after confirmation

Any `http(s)` link navigates in the app through the normal load path, with redirects, the Markdown fallbacks and the local-network guard. A page that has no Markdown version ends on the error page with **Open in browser**. A `mailto:` link is the only link that leaves: the page shows where it goes and offers **Open** or **Stay here**.

**Pros:**
- Clicking a link and typing its address behave the same.
- Followed links get ADR-005's fallbacks, so `/post` finds `/post.md` and `/llms.txt` opens in the app.
- Leaving is always announced, and nothing opens another program without the reader saying so.
- Less classification code: the extension test and its helpers are gone.

**Cons:**
- A link to a page with no Markdown now takes an extra step (the error page, then **Open in browser**) instead of opening the system browser at once. A failed fallback search costs up to five requests (ADR-005).
- Enclosure ("Listen:") links are ordinary web links, so they are fetched, refused as not Markdown and end on the error page.
- A fallback page shows no banner, so the shortcut for opening the original (Ctrl/Cmd+Shift+O) is the only route to it.

### Option C: Confirm before every link that would leave, including web pages

As Option B, but a page that turns out not to be Markdown asks first before the system browser opens.

**Pros:**
- No surprise hand-offs at all.

**Cons:**
- The error page already makes the reader choose **Open in browser**, so a second prompt repeats it.

## Decision

**We will take Option B.**

It matches what a reader expects from an address bar, uses the fallbacks ADR-005 built, and keeps the system browser behind an explicit step. Option A's privacy argument did not hold up: an in-app fetch sends less than the system browser would. The remaining exits are `mailto:` after confirmation, and **Open in browser** (a button, or Ctrl/Cmd+Shift+O) for the original of a fallback page or a page with no Markdown. The opener keeps its allowlist.

The session holds the pending link in `State.leaving` and sends the effect only on `ConfirmLeaving`. `StayHere`, and any navigation, drop it.

## Consequences

**Positive:**
- Links between Markdown sites work on extensionless addresses, folder indexes and `llms.txt`.
- One path for typed and followed addresses, so ADR-003's guards and ADR-005's fallbacks cannot be skipped by clicking.
- A reader is never moved to another program silently.

**Negative:**
- Enclosure links cost a failed fetch before the offer of the system browser. Telling enclosures apart from other links would let them go straight to the system. That is a possible follow-up, and it would need to keep the confirmation.
- Every link click is a request from the app to that site (no cookies, no `Referer`), as any typed address is.
- Non-Markdown destinations such as PDFs and HTML-only pages take two steps to open externally.

**Neutral:**
- The fallback banner of ADR-005 is gone (amended there), since the address bar already shows where the reader is.
- Links to local-network addresses from a public page are still refused (ADR-003).

## Related Decisions

- ADR-003: HTTP client. Its guards still apply to every link; its note that non-Markdown links go to the system browser is amended.
- ADR-005: Markdown fallbacks. Followed links now use them; the banner is removed.
- ADR-006: Shared front matter. The enclosure link now follows this path.
