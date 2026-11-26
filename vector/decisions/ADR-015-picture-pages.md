# ADR-015: A picture opened as a page is shown as a picture page

**Date:** 2026-11-26
**Status:** proposed
**Deciders:** gBRETT (product owner)

## Context

Opening an image address directly, by typing it, choosing it from History or following a link such as `[full size](photo.jpg)`, gives the `notMarkdown` error and an offer of the system browser. We already fetch and decode the same image when a page embeds it (ADR-004). So a reader who follows a post's link to its full-size picture is sent out of the app for something the app can draw. bonBon's charter says: "If the server returns a picture, the browser shows the picture." ADR-010 keeps web links in the app wherever the app can show what is there. An image is something it can show.

## Decision Drivers

- VECTOR Design Principle 5: every feature helps someone read what was written or reach the next thing to read. A post's linked picture is part of what the writer published.
- ADR-010: a link is tried in the app, and the system browser is an explicit step.
- ADR-004: PNG, JPEG, GIF and WebP only (no SVG). The bytes decide the format. Its limits apply: 16 MiB per response, 16,384 px on a side, 25 MP, decoded down to 2,048 px wide, and one decode at a time.
- ADR-012: a page's images are capped at 30 × text size so they can't push the first line of text out of the window. A picture page has no text.
- ADR-011: Source view shows the writer's own Markdown. It must not show text the writer never wrote.
- ARCHITECTURE.md: the session decides and `services` does the I/O. `Pages` caches laid-out pages by `Look` and width. The parse tree is the document (VECTOR Principle 1).

## Options Considered

### Option A: Keep the error (do nothing)

**Pros:**
- No new code. The system browser shows the image.

**Cons:**
- The reader leaves the app for content we can already decode, against the intent of ADR-010.
- The link is a dead end inside the app: it can't be reopened from History or reached with Back.

### Option B: A picture view separate from pages

A separate widget and state for "an image, not a document", with its own zoom and fit controls.

**Pros:**
- Room for image features like zoom, actual size and fit to window.

**Cons:**
- A second kind of thing in the view, so History, Back, Reload, error handling, keyboard access (ADR-008) and theming each need a second path.
- More than the charter asks, and image tools aren't the point of this app (Principle 5).

### Option C: A generated one-image page (chosen)

The session makes a `Document` holding a single image block for the address and shows it through the usual page path.

**Pros:**
- Everything else works unchanged: History, Back, Reload, the reader's page colour, the Web images setting, image notes and retry, Tab and Enter, and Open in browser.
- The layout already places images; one flag removes the height cap.

**Cons:**
- The page is generated, not written, so features that assume Markdown (Source view, front matter) need a rule.

## Decision

**We will show an image response as a generated page that holds only that image.**

- **What counts as a picture:** a successful response whose `Content-Type` is `image/*`, `application/octet-stream` or missing, and whose bytes decode as PNG, JPEG, GIF or WebP within ADR-004's limits. For a local file: a path with one of ADR-004's image extensions, then the bytes decide. Markdown rules come first, so a `.md` address served as `application/octet-stream` is still a Markdown page. SVG, and bytes that match none of the four formats, keep the `notMarkdown` error.
- **Fallbacks:** a response that is a picture is shown, and ADR-005's fallbacks are not tried for it. Fallbacks stay for responses that are neither Markdown nor a picture.
- **The page:** one image block for the address, with no alt text, titled with the last segment of the path (for example `photo.jpg`), or the host if the path has none. It has no front matter and no byline.
- **Height:** ADR-012's cap doesn't apply on a picture page. The image is scaled down to the column only, keeping its shape, and the page scrolls. ADR-004's 2,048 px decode width still applies.
- **One request:** the bytes from the page fetch go to the image cache under the page's address, so the picture isn't fetched again. Reload fetches it again as a page.
- **Web images setting:** a picture page the reader opened always loads, whatever the Web images setting says. Opening the address is the reader's click, as "on click" already treats a click (ADR-004). A picture that fails to decode shows the usual image note, with retry.
- **Source view:** disabled on a picture page. The Source button is inactive and Ctrl/Cmd+U does nothing, because there's no Markdown to show.
- **Leaving:** Open in browser (Ctrl/Cmd+Shift+O) stays available for picture pages from the web.
- **The `Accept` header for pages is unchanged.** Servers return images regardless of `Accept`, so we don't need to ask for them.

## Consequences

**Positive:**
- A post's links to full-size pictures, and pasted image addresses, open in the app with Back, History and the reader's colours.
- No new widget or view mode, and nothing new in the layout engine beyond one flag to skip the height cap.
- The picture is fetched once.

**Negative:**
- No zoom or actual-size view. An image wider than the column is shown smaller, and one wider than 2,048 px is decoded at 2,048. Zooming would need its own ADR.
- A page that isn't written Markdown now exists, so later features that assume a page has Markdown (Source view, front matter, future search or copy as Markdown) need to check for picture pages.
- An address that returns a decodable image where the reader expected Markdown now shows the image instead of an error. That's better, but the "not Markdown" path is no longer the only outcome for non-Markdown responses.

**Neutral:**
- The 25 MP and 16 MiB limits refuse very large photos. They get an image note with Open in browser.
- The layout cache key gets a "picture page" flag, or the flag lives in the `Document`, so pictures and text pages are never confused.

## Related Decisions

- ADR-004: Images. Formats, limits, decoding, the cache and the Web images setting are reused.
- ADR-005: Markdown fallbacks. Not tried for picture responses.
- ADR-010: Web links open in the app. This ADR adds pictures to what the app shows.
- ADR-011: Source view. Disabled on picture pages.
- ADR-012: Image max height. Doesn't apply on picture pages.
