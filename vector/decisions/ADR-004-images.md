# ADR-004: Decode images in services and draw them with iced, within fixed limits

**Date:** 2026-10-29
**Status:** accepted
**Deciders:** gBRETT (product owner)

## Context

Phase 2b draws Markdown images (`![alt](src)`) in the page, from local files and from the web (plan: `vector/plans/phase-2b-images.md`). Today they render as *[image: alt]*. The first-release ship criterion asks for the markdownWeb examples and the scripting.com archive "with images shown", and 4,144 of the archive's 16,869 posts use Markdown images. Images are untrusted input: the bytes come from any web server, and a 16 MiB file can claim to be millions of pixels. Something has to decode PNG, JPEG, GIF and WebP, and the iced document widget (ADR-001) has to draw the result without stalling frames or exhausting memory.

## Decision Drivers

- `services` does all I/O and runs blocking work on background threads (ARCHITECTURE.md). The render thread must never decode.
- Formats needed by real posts: PNG (4,193 sources in the archive), GIF (104) and JPEG (16). WebP is common on the wider web.
- Untrusted input must be bounded before pixels are allocated, as ADR-003 bounds page bodies.
- No HTML, DOM or webview (VECTOR.md). HTML `<img>` is stripped (ADR-002), so only Markdown images reach this code.
- Each release binary is built for four platforms in CI without system libraries.

## Options Considered

### Option A: Decode with the `image` crate in services, draw a ready RGBA handle

`image` 0.25, the version iced already uses, is built with only its `png`, `jpeg`, `gif` and `webp` features. A background thread decodes to RGBA and downscales to at most 2,048 pixels wide. It then hands over `iced::widget::image::Handle::from_rgba`, which iced's `image-without-codecs` feature draws.

**Pros:**
- Decoding happens off the render thread, under our limits, before the UI sees anything.
- Four codecs only, so the attack surface and binary size stay small.
- The image size is known before layout, so layout stays pure: it never asks the renderer to measure.
- Pure-Rust codecs build on every CI platform.

**Cons:**
- We own the decode pipeline: limits, downscaling and the error mapping.
- RGBA in memory costs 4 bytes per pixel, which needs a cache budget.

### Option B: Let iced decode lazily (`image` feature, `Handle::from_bytes`)

Pass the fetched bytes to iced and let its renderer decode them when they are first drawn.

**Pros:**
- Least code.

**Cons:**
- Decoding happens on the render thread when the image is first drawn, so a large image stalls frames.
- There is no place to apply size limits before allocation.
- Dimensions are unknown until drawn, so layout would have to re-run after the renderer measures.
- iced's `image` feature enables every default codec in `image`: AVIF, TIFF, BMP, ICO, PNM, TGA, DDS, HDR, EXR and more. That is far more parser surface for untrusted bytes.

### Option C: Also support SVG

Add an SVG renderer, `resvg` through iced's `svg` feature.

**Pros:**
- Logos and diagrams on some sites are SVG.

**Cons:**
- SVG is a document format, with scripts, external references, fonts and CSS. Rendering it safely is its own decision, close to ADR-001's "no HTML" boundary.
- The archive uses no SVG images.

### Option D: Do nothing

Keep the placeholder text.

**Pros:**
- No new risk.

**Cons:**
- Fails the ship criterion. Posts that are about pictures don't read.

## Decision

**We will decode images with the `image` crate (PNG, JPEG, GIF and WebP only) in `services`, on a background thread, to downscaled RGBA within fixed limits, and draw the ready handles with iced's `image-without-codecs`.**

Option A is the only one that keeps decoding off the render thread and bounded before allocation. It also keeps layout pure, with sizes known up front. Option B fails the threading and limits drivers. Option C is deferred to its own ADR, and Option D fails the ship criterion.

**Limits.** These are named constants in `session::images::limits`, quoted by the plan, the tests and this ADR:
- At most 16 MiB per file or response, counted after decompression (ADR-003).
- At most 16,384 pixels on a side and 25 megapixels. Both are checked from the header before decoding, and are also enforced by `image::Limits`.
- Images wider than 2,048 pixels are downscaled to 2,048 when decoded.
- Decodes run one at a time, behind a mutex in `services::images` that also covers downscaling and conversion. Downscaling box-samples straight into the output (`imageops::thumbnail`), with no intermediate buffer. Peak decode memory is about one 25 MP image as RGBA, 100 MB, plus a 16 MiB input. A progressive JPEG can need about twice that, in its decoder's internal buffers, which `image::Limits` doesn't count. That is bounded, and accepted.
- At most 4 image loads at once, across the app.
- A 256 MiB least-recently-used cache of decoded pixels, keyed by requested location. The current page's images are pinned, so a very image-heavy page can hold more than the budget while it is shown.

**Fetching** reuses the page fetch loop (ADR-003): the same redirect rules, deadline and cap, no cookies, no `Referer` and no credentials. A public page can't reach the disk or the local network.

**Images fetch without a click.** That raises the stakes of ADR-003's private-network rules, so the product owner decided the following during step 3's security review (2026-10-29):
- **Local-network detection is wider.** It now covers names that only mean something at home (single-label names such as `router`, and `.local`, `.home.arpa`, `.internal` and `.lan`). It also covers more address ranges:
  - 100.64.0.0/10, which carrier NAT and Tailscale use
  - 198.18.0.0/15, 192.0.0.0/24 and 0.0.0.0/8
  - multicast and broadcast
  - IPv6 site-local addresses
  - IPv4 carried inside IPv6: mapped, compatible, NAT64 and 6to4

  Links and redirects share this check.
- **The connect-time check ships in 2b (step 5).** The fetcher resolves each name, refuses if any resolved address is local, and connects only to the addresses it checked. That closes ADR-003's DNS-rebinding follow-up, for pages and images alike.
  - **Which fetches are checked.** Links and images from a public page are always checked. The reader's own requests are not: what they type, reopen from History, or follow from a local file or local page. Those may reach intranet names that resolve to private addresses. A redirect within the host the reader asked for (http to https, another path) is unchecked; a redirect to any other host was the site's choice, so it is checked. An entry that settled on another public host is checked when reloaded or revisited with Back.
  - **Proxies.** A proxy resolves names itself, out of the check's sight. So checked fetches **go direct** and skip any `HTTP_PROXY`, `HTTPS_PROXY` or `ALL_PROXY` (product owner, 2026-10-29). The reader's own fetches still use the proxy. Behind a mandatory corporate proxy, checked fetches fail as unreachable, as corporate MITM proxies already do (ADR-003).
- **Bad ports are refused for web images:** the Fetch standard's blocked-port list, plus Redis on 6379.
- **Accepted risk: local files load local-network images with no click.** A local file is the reader's own, so its images load like any other, including from `127.0.0.1` or the LAN, the moment it opens. A downloaded README can therefore make the app send GET requests on the local network, and tell a remote image host the reader's IP. This matches how a local HTML file behaves in a browser.
- **Follow-up: mixed content.** An `https` page can still show `http` images, which a network attacker could swap. Browsers upgrade or block these.

## Consequences

**Positive:**
- Posts with pictures read as written, from files and the web.
- A hostile image can cost at most one bounded decode. It can't stall the UI or exhaust memory.
- Layout stays a pure function of blocks, theme, width and known image sizes.

**Negative:**
- SVG, AVIF, TIFF and the other formats show their description instead. Adding one is a deliberate change to the feature list.
- Animated GIFs and WebPs show their first frame.
- Downscaling to 2,048 pixels softens very large images on HiDPI screens. That is acceptable at column widths around 700 pixels.
- **Binary size:** the four codecs add to the release binary. The baseline before 2b is 32.6 MB, and the measured cost is added here at the end of phase 2b.

**Neutral:**
- `iced::widget::image::Handle` is held in session state as a value type: reference-counted, compared by id and cheap to clone. ARCHITECTURE.md's import rules name it.
- Front matter image fields (`image`, `inlineImage`, `metaImage`) are not covered here. They are phase 3 (VECTOR.md, "Image semantics").

## Related Decisions

- ADR-001: Render Markdown natively with our own layout engine. Images are boxes in our layout, drawn by the document widget.
- ADR-002: Strip raw HTML from documents, keeping the text between tags. `<img>` never becomes an image.
- ADR-003: Fetch remote documents with ureq and handle URLs with the url crate. Images use the same fetch loop and refusals.
