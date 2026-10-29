# Plan: Phase 2b: images

**Created**: 2026-10-29
**Branch**: `feat/phase-2b`, from `main` after phase 2a (#8)
**Status**: approved (2026-10-29, revision 3)

## Goal

Show the pictures in a post. Markdown images (`![alt](src)`) are drawn in the page by our own layout engine. They can come from local files, or from the web over `http(s)`. Today every image renders as the placeholder text *[image: alt]*. That is the gap between 2a and the ship criterion "renders the markdownWeb examples and the scripting.com archive … with images shown".

**Decisions already made by the product owner (2026-10-29):**
- 2b is images only. The Settings panel and a home page are phase 2c.
- Remote images load by default. The reader can switch to "on click" or "off". The control is a minimal toolbar menu, like the font picker, and it is saved in the existing settings store.
- Raw HTML `<img>` is stripped and never becomes an image (ADR-002).
- Image fetches follow ADR-003: ureq, no cookies, size caps counted after decompression. A public page can't reach the disk or the local network.

**What real posts do** (the markdownWeb `scriptingInMarkdown` archive, 16,869 posts, checked 2026-10-29):
- 4,144 posts use `![alt](src)`. Many put the image at the very start of a text paragraph (`![](…/cherries.png)I gotta say…`). scripting.com draws those as right-margin images. Images mixed into paragraphs are therefore the common case.
- Only 2 posts use `<img>`.
- 2,628 posts use front matter `image:`, 450 use `metaImage:` and 159 use `inlineImage:`. Those fields are phase 3 (VECTOR open question "Image semantics").
- Formats in `![]()` sources: PNG 4,193, GIF 104, JPEG 16.

**User-facing terms**, used in `content.rs`:
- **image**: the picture
- **description**: what stands in for an image that isn't shown. It is the alt text, written `[image: …]`, the convention since phase 1. An empty description is written `[image]`.
- **web images**: images fetched over `http(s)`, which the setting governs

Revisions 2 and 3 answer the plan reviews; see the Plan Review Summary.

## Acceptance Criteria

1. [ ] **Images are drawn.**
   - Formats: PNG (any colour type, with alpha), JPEG (EXIF orientation is ignored), GIF (its first frame) and WebP (its first frame).
   - Size: the image's width in logical pixels, scaled down to fit the text column, with the aspect ratio kept. It is never scaled up. Height = `round(h × shown_width / w)`, at least 1. An image exactly as wide as the column is drawn at its own size.
   - A resize relays images out with the text.
2. [ ] **Images in paragraphs.** A paragraph is split at each image: the text before it, the image as its own block, then the text after it. Reading order is kept.
   - Whitespace at the cut is trimmed, and empty text parts are dropped.
   - Emphasis and links carry across the cut: `[see ![a](b.png)](url)` is the linked text "see", then the image linked to `url`.
   - An image inside a heading or a table cell stays as its description text. An image in a code span or code block stays literal Markdown.
   - The same split happens inside list items and quotes.
3. [ ] **Sources resolve like links** (ADR-003).
   - In a local file: relative and absolute paths, `~/` and `file://` read from disk, and `http(s)` loads from the web.
   - In a web page: the source resolves against the page's final URL. Credentials in a source are dropped.
   - Refused, so the description shows and no request or disk read happens:
     - `file:` URLs, drive paths and `\\shares` from a web page
     - loopback, private and link-local addresses from a public page, including disguised spellings
     - `data:`, `javascript:` and other non-`http(s)` schemes
     - an empty source
   - A redirect from a public image URL into the local network, to another scheme, or round in a loop is refused. It shows the "refused" note, because retrying can't help.
4. [ ] **What can be shown.**
   - **Web images:** any `image/*` type, `application/octet-stream` or no type is accepted. The type is matched case-insensitively and its parameters are ignored, and the bytes then decide. Any other type, such as `text/html`, is refused without decoding.
   - **Local images:** the extension is `.png`, `.jpg`, `.jpeg`, `.gif` or `.webp`, in any case. The bytes decide the format.
   - **Shown:** bytes that decode as PNG, JPEG, GIF or WebP, whatever the declared type.
   - **Not shown:** SVG, AVIF, anything else, and bytes that don't decode.
   - **Limits, inclusive:** at most 16 MiB per file or response (after decompression), at most 16,384 pixels on a side, and at most 25 megapixels. One over any limit is refused before any pixels are allocated.
   - An image that isn't shown leaves its description and a note (AC 6). The rest of the page is unaffected.
5. [ ] **Fetching.**
   - Web images are fetched with an image `Accept` header, the same redirect rules as pages, no cookies, no `Referer`, and no credentials.
   - At most 4 image loads run at once **across the app**.
   - The same location used several times is requested once, even across pages, while it is loading or cached.
   - Leaving a page drops its queued loads. Loads already running finish, count towards the 4, and are cached. They appear only if the current page uses the same image.
   - Failures are not cached. Revisiting a page, or reloading it, retries every image that isn't shown.
6. [ ] **What the reader sees in place of an image.** The description, then a second line saying why:

   | State | Second line | Click on that line |
   |---|---|---|
   | Loading | "Loading image…" | nothing |
   | Waiting (on click) | "Click to load" | loads the image |
   | Off | "Web images are off. Press Ctrl+Shift+I to load this page's." (Cmd on macOS) | nothing |
   | Failed (network, server or timeout) | "Couldn't load this image. Click to try again." | retries |
   | Not supported (type, format or decode) | "This image's format isn't supported." | nothing |
   | Too large | "This image is too large to show." | nothing |
   | Refused (AC 3) | "This page can't show this image." | nothing |

   The clickable lines are styled as links, underlined in the link colour, with the pointer cursor. The description of a linked image is itself a link, so the destination stays reachable before the image loads.
7. [ ] **The Web images menu.** The toolbar shows **Web images: automatic / on click / off**, next to the font picker. It applies to web images only; local images always load.
   - **automatic** (the default, and the fallback for a missing or unknown stored value): web images load with the page.
   - **on click**: web images wait.
   - **off**: web images are not requested.
   - Changing the setting never hides an image already shown and never cancels a running load.
   - Moving to *automatic* loads the waiting and off images on the current page.
   - Moving to *on click* turns queued and off images into waiting ones.
   - Moving to *off* turns queued and waiting images into off ones.
   - The choice is saved and comes back after a restart.
8. [ ] **This page only.** Ctrl+Shift+I (Cmd+Shift+I on macOS) loads every waiting, off or failed web image on the current page, without changing the saved setting. It is the keyboard route to everything a click does, because iced can't focus items inside the page.
9. [ ] **Linked images.** Clicking a shown image inside a link follows the link, and hovering it shows the address, as for text links. On a placeholder, the description follows the link and the second line does what AC 6 says.
10. [ ] **Memory.**
    - Decoded images wider than 2,048 pixels are downscaled to 2,048 when decoded.
    - Decodes run one at a time.
    - Decoded images are cached, keyed by their requested location, up to 256 MiB of pixels.
    - When the cache is over budget, the image least recently used by a page is dropped. An image on the current page is never dropped.
    - Going back to a page whose images are still cached makes no request. If one was dropped, it is requested again.
11. [ ] **Reading position.** When images arrive, the text the reader is looking at stays where it is on screen. Relayouts for arriving images are coalesced to at most one per 150 ms. A pending `#anchor` is re-applied after each relayout until the reader scrolls.
12. [ ] **Selection and copy.** Copying a selection that spans an image, shown or not, copies `[image: description]` (or `[image]`) on its own line, as plain text does today.
13. [ ] **CI.** `format`, the four `platform` jobs and `ci` are green. The acceptance suite serves generated PNG, GIF, JPEG and WebP bytes from the test server on all four platforms.

## User-Facing Behavior

No `/specs` artifacts exist for this phase. The scenarios below are drafted here, and each lands in `features/images.feature` test-first, in the step that implements it (marked *step n*). They reuse existing steps where they fit:
- `a file … containing`
- `I view`
- `I am viewing`
- `I am viewing the URL of`
- `I viewed … then the URL of …`
- `nothing was read from disk`
- `the site has seen N requests for`
- `I restart the app`
- `I go back`
- `I reload`
- `the system opened`
- `the document text is`

The local-network rules can't be shown end to end, because the test server is itself on 127.0.0.1, so every page it serves counts as local. As in 2a, they are covered by unit tests (steps 3 and 5). The acceptance suite covers the disk side and the redirect-to-`file:` side.

```gherkin
@images
Feature: Images
  As a reader of the web of Markdown
  I want the pictures in a post drawn in the page
  So that posts read as their writers meant, with nothing else running

  # Fixtures: the 2a test server (its routes, such as /index.md, stay) plus images
  # generated in the test, with no binary fixtures:
  #   /img/red.png      image/png                  40×20
  #   /img/alpha.png    image/png, with alpha      8×8
  #   /img/wide.png     image/png                  4000×100
  #   /img/anim.gif     image/gif, 2 frames        30×30
  #   /img/photo.jpg    image/jpeg                 24×24
  #   /img/photo.webp   image/webp                 24×24
  #   /img/untyped.png  (no type), PNG bytes       10×10
  #   /img/mislabel.png image/gif, PNG bytes       10×10
  #   /img/logo.svg     image/svg+xml
  #   /img/page.png     text/html
  #   /img/missing.png  404
  #   /img/moved.png    302 → /img/red.png
  #   /img/to-file.png  302 → file:///etc/hosts
  #   /img/tall.png     PNG header claiming 16385×1
  #   /img/huge.png     PNG header claiming 6000×6000 (36 MP)
  #   /img/flaky.png    500 the first time, then red.png
  #   /img/slow-1.png … /img/slow-6.png: red.png, held by the world until released
  # Local: "an image file … of W by H pixels" writes a generated PNG to the temporary home.
  # "linking an image to X" writes "![<file name of X>](X)", so the description is the file name.
  # The world runs loads synchronously and parks held ones, as in 2a, so request counts
  # are exact when each step returns. No polling or settling is needed.
  # "the site serves P containing T" adds a text/markdown route for the scenario.

  # ── step 2: images become blocks (placeholders only) ───────────────────────

  Scenario: An image in the middle of a paragraph keeps reading order
    Given a file "mid.md" containing "Before ![a box](none.png) after"
    When I view "mid.md"
    Then the document text is "Before\n[image: a box]\nafter"

  Scenario: An image in a heading stays its description
    Given a file "h.md" containing "# Title ![logo](none.png)"
    When I view "h.md"
    Then I see the document titled "Title [image: logo]"
    And no image is shown

  # ── step 4: the setting ─────────────────────────────────────────────────────

  Scenario: Web images load automatically on first run
    Then web images are set to "automatic"

  Scenario Outline: The image setting is remembered
    Given I set web images to "<mode>"
    When I restart the app
    Then web images are set to "<mode>"
    Examples:
      | mode     |
      | on click |
      | off      |

  # ── step 8: drawn images (local files, then the web) ───────────────────────

  Scenario: A local image is drawn at its own size
    Given an image file "pic.png" of 12 by 8 pixels
    And a file "local.md" containing "![a red box](pic.png)"
    When I view "local.md"
    Then the image "a red box" is shown at 12 by 8

  Scenario: A web image on a web page is drawn
    Given the site serves "/post.md" containing "![red](img/red.png)"
    When I enter the URL of "/post.md"
    Then the image "red" is shown at 40 by 20
    And the site was asked for an image at "/img/red.png" with no cookie and no referrer

  Scenario Outline: Formats that are drawn
    Given the site serves "/post.md" containing "![x](<src>)"
    When I enter the URL of "/post.md"
    Then the image "x" is shown at <width> by <height>
    Examples:
      | src               | width | height |
      | /img/alpha.png    | 8     | 8      |
      | /img/anim.gif     | 30    | 30     |
      | /img/photo.jpg    | 24    | 24     |
      | /img/photo.webp   | 24    | 24     |
      | /img/untyped.png  | 10    | 10     |
      | /img/mislabel.png | 10    | 10     |
      | /img/moved.png    | 40    | 20     |

  Scenario Outline: Images that can't be shown say why
    Given the site serves "/post.md" containing "Before ![a logo](<src>) after"
    When I enter the URL of "/post.md"
    Then the image "a logo" is not shown and says "<note>"
    And the document text is "Before\n[image: a logo]\nafter"
    Examples:
      | src              | note                                           |
      | /img/logo.svg    | This image's format isn't supported.           |
      | /img/page.png    | This image's format isn't supported.           |
      | /img/missing.png | Couldn't load this image. Click to try again.  |
      | /img/tall.png    | This image is too large to show.               |
      | /img/huge.png    | This image is too large to show.               |
      | /img/to-file.png | This page can't show this image.               |

  Scenario: An image wider than the column is scaled down
    Given the site serves "/post.md" containing "![wide](img/wide.png)"
    When I enter the URL of "/post.md"
    Then the image "wide" is as wide as the column, with its aspect ratio kept

  Scenario: A local file can show web images, including from the local network
    Given a file "local.md" linking an image to the URL of "/img/red.png"
    When I view "local.md"
    Then the image is shown at 40 by 20

  Scenario Outline: A web page can't show files from the disk
    Given an image file "secret.png" of 4 by 4 pixels
    And the site serves "/post.md" containing an image from <source>
    When I enter the URL of "/post.md"
    Then the image is not shown and says "This page can't show this image."
    And nothing was read from disk
    Examples:
      | source                          |
      | the file:// URL of "secret.png" |
      | "C:\\secret.png"                |
      | "\\\\host\\share\\secret.png"   |
      | "data:image/png;base64,AAAA"    |

  Scenario: A repeated image is fetched once
    Given the site serves "/post.md" containing "![a](img/red.png) ![b](img/red.png)"
    When I enter the URL of "/post.md"
    Then the image "a" is shown at 40 by 20
    And the image "b" is shown at 40 by 20
    And the site has seen 1 request for "/img/red.png"

  Scenario: A failed image can be tried again
    Given the site serves "/post.md" containing "![x](img/flaky.png)"
    And I am viewing the URL of "/post.md"
    And the image "x" is not shown and says "Couldn't load this image. Click to try again."
    When I click the note under "x"
    Then the image "x" is shown at 40 by 20

  Scenario: Reloading retries images that failed
    Given the site serves "/post.md" containing "![x](img/flaky.png)"
    And I am viewing the URL of "/post.md"
    When I reload
    Then the image "x" is shown at 40 by 20

  Scenario: Copying across an image copies its description
    Given an image file "pic.png" of 12 by 8 pixels
    And a file "c.md" containing "Before\n\n![a red box](pic.png)\n\nafter"
    And I am viewing "c.md"
    When I select everything and copy
    Then the copied text is "Before\n[image: a red box]\nafter"

  Scenario: HTML img is never an image
    Given the site serves "/post.md" containing "<img src='img/red.png'> text"
    When I enter the URL of "/post.md"
    Then no image is shown
    And the site has seen 0 requests for "/img/red.png"

  # ── step 9: the setting, clicks, links, loading and the cache ──────────────

  Scenario: On click: web images wait, local images load
    Given I set web images to "on click"
    And an image file "pic.png" of 12 by 8 pixels
    And a file "both.md" linking an image to "pic.png" and an image to the URL of "/img/red.png"
    When I view "both.md"
    Then the image "pic.png" is shown at 12 by 8
    And the image "red.png" is not shown and says "Click to load"
    And the site has seen 0 requests for "/img/red.png"
    When I click the note under "red.png"
    Then the image "red.png" is shown at 40 by 20
    And the site has seen 1 request for "/img/red.png"

  Scenario: Off: web images are not requested and can't be clicked
    Given I set web images to "off"
    And the site serves "/post.md" containing "![red](img/red.png)"
    When I enter the URL of "/post.md"
    Then the image "red" is not shown and says "Web images are off. Press Ctrl+Shift+I to load this page's."
    And clicking the note under "red" does nothing
    And the site has seen 0 requests for "/img/red.png"

  Scenario Outline: Switching to automatic loads the images on the page
    Given I set web images to "<mode>"
    And the site serves "/post.md" containing "![red](img/red.png)"
    And I am viewing the URL of "/post.md"
    And the site has seen 0 requests for "/img/red.png"
    When I set web images to "automatic"
    Then the image "red" is shown at 40 by 20
    Examples:
      | mode     |
      | on click |
      | off      |

  Scenario: Switching off keeps images already shown
    Given the site serves "/post.md" containing "![red](img/red.png)"
    And I am viewing the URL of "/post.md"
    When I set web images to "off"
    Then the image "red" is shown at 40 by 20

  Scenario: Switching from off to on click makes images clickable
    Given I set web images to "off"
    And the site serves "/post.md" containing "![red](img/red.png)"
    And I am viewing the URL of "/post.md"
    When I set web images to "on click"
    Then the image "red" is not shown and says "Click to load"

  Scenario: Loading this page's images leaves the setting alone
    Given I set web images to "off"
    And the site serves "/post.md" containing "![red](img/red.png)"
    And I am viewing the URL of "/post.md"
    When I load this page's images
    Then the image "red" is shown at 40 by 20
    And web images are set to "off"

  Scenario: A linked image follows its link
    Given the site serves "/post.md" containing "[![red](img/red.png)](https://example.com/)"
    And I am viewing the URL of "/post.md"
    When I click the image "red"
    Then the system opened "https://example.com/"

  Scenario: A linked image waiting for a click
    Given I set web images to "on click"
    And the site serves "/post.md" containing "[![red](img/red.png)](https://example.com/)"
    And I am viewing the URL of "/post.md"
    When I click the note under "red"
    Then the image "red" is shown at 40 by 20
    And nothing is opened externally
    When I click the description "red"
    Then the system opened "https://example.com/"

  Scenario: At most four images load at once
    Given the site serves "/gallery.md" with images "/img/slow-1.png" to "/img/slow-6.png"
    And loading each image under "/img/slow-" is slow
    When I enter the URL of "/gallery.md"
    Then the site has seen 4 image requests
    When "/img/slow-1.png" finishes loading
    Then the site has seen 5 image requests

  Scenario: Leaving a page drops its waiting loads
    Given the site serves "/gallery.md" with images "/img/slow-1.png" to "/img/slow-6.png"
    And loading each image under "/img/slow-" is slow
    And I entered the URL of "/gallery.md"
    When I enter the URL of "/index.md"
    And every slow image finishes loading
    Then the site has seen 4 image requests
    And I see the document titled "Remote Home"
    And no image is shown

  Scenario: Going back shows cached images without fetching again
    Given the site serves "/post.md" containing "![red](img/red.png)"
    And I am viewing the URL of "/post.md"
    And the image "red" is shown at 40 by 20
    And I enter the URL of "/index.md"
    When I go back
    Then the image "red" is shown at 40 by 20
    And the site has seen 1 request for "/img/red.png"

  Scenario: An image dropped from the cache is fetched again
    Given the image cache has room for one 40 by 20 image
    And the site serves "/a.md" containing "![red](img/red.png)"
    And the site serves "/b.md" containing "![photo](img/photo.jpg)"
    And I viewed the URL of "/a.md" then the URL of "/b.md"
    When I go back
    Then the image "red" is shown at 40 by 20
    And the site has seen 2 requests for "/img/red.png"
```

## Design notes

- **ADR-004: decoding and drawing.**
  - iced's `image-without-codecs` feature draws the images. The `image` crate decodes them, with only its `png`, `jpeg`, `gif` and `webp` features.
  - Decoding happens in `services`, on a background thread. It produces RGBA, downscales to 2,048 pixels wide if needed, and hands the UI a ready `Handle::from_rgba`. iced's lazy `from_bytes` would decode on the render thread, with no limits.
  - Pixels are capped before they are allocated: `image::Limits` on width, height and allocation, plus a 25 MP check against the header's dimensions.
  - Decodes run one at a time, behind a mutex in `services::images`. Peak decode memory is then about one 25 MP image (100 MB of RGBA) plus a 16 MiB input. That figure is recorded in the ADR.
  - The images on the current page are pinned, so a very image-heavy page can hold more than the 256 MiB budget while it is shown. The ADR says so.
  - SVG needs its own ADR: it is a document format, with scripts and external references.
  - **Limits as code:** the limits live as named constants in `session::images::limits`. The ADR, the criteria and the tests all quote those constants, so they can't drift.
- **Model.**
  - `model::inlines` yields `Vec<Inline>`, where `Inline` is `Span(Span)` or `Image { src, alt, link }`. Style and the link stack carry across images.
  - `flatten` turns images back into `[image: alt]` spans, for headings and table cells.
  - `split` turns a paragraph into `Paragraph` and `Block::Image { src, alt, link }` blocks.
  - `plain_text` writes an image as `[image: alt]` or `[image]` on its own line.
- **Session, pure.** `session/images/` holds four modules:
  - `resolve.rs`: `resolve(src, current) -> Result<Location, Refusal>`. It uses the 2a link rules through a shared `session/href.rs`, which takes `as_browsers_read`, `is_disk_shaped` and the boundary check out of `links/page.rs`. Any extension is allowed.
  - `planner.rs`: **one app-wide table** maps `Location` to `Waiting`, `Off`, `Queued`, `Loading`, `Shown(Picture)` or `Failed(Problem)`. It is the single source of truth.
    - When a page becomes ready, it resolves each `src` once and stores only `src → Location`, or `Refused`. The page holds no image state of its own. Layout's `ImageView` is derived from the page map plus the table, and mode changes act on the table entries that the current page references.
    - It keeps at most 4 locations `Loading`, across the app, and emits `Effect::LoadImage(Location)`.
    - On `Event::ImageLoaded { location, result }`, it updates the state and the cache, starts the next queued load, and bumps `images.generation` if the current page uses that location.
    - On navigation, it drops the queue; running loads stay.
    - It also handles `Event::LoadImage(Location)` (a click), `Event::LoadPageImages` (the shortcut) and `Event::ChooseImageLoading(mode)`.
  - `cache.rs`: a pure LRU by pixel bytes, with a budget the tests can inject. An image is touched when a page that uses it becomes ready, and when it loads. Images of the current page are pinned.
  - `limits.rs`: the constants.
  - `Picture { width, height, handle }`. `Handle` is a cheap, reference-counted iced value type, and it compares by id, so cloning `State` stays cheap. ARCHITECTURE.md's Session row and Import Direction say this explicitly.
  - `Problem` is `Unavailable`, `Unsupported`, `TooLarge` or `Refused`. Images don't reuse the page `ErrorCode`.
  - `ImageLoading` is `Automatic`, `OnClick` or `Off`, held in `Settings` with a serde default of `Automatic`.
- **Services.**
  - `services::web` splits into `fetch_bytes(url, want, limits) -> Result<(Url, Vec<u8>), FetchError>`, the shared redirect, deadline, cap, encoding and no-cookie loop, and `fetch`, which pages use and which adds `to_text`.
    - `FetchError` is neutral: `NotFound`, `Server`, `Unreachable`, `BadRedirect`, `TooLarge`, `WrongType` or `Undecodable`.
    - Two pure, unit-tested functions map it: pages get an `ErrorCode` exactly as 2a does today, and images get a `Problem`, where `BadRedirect` becomes `Refused`. `Want::{Page, Image}` chooses the `Accept` header and the type policy. ureq sends no `Referer`; a test pins that.
  - `services::files` gains `read_capped(path)`, which `read_markdown` and image loading share.
  - `services::images::load(location) -> Result<Picture, Problem>`.
  - `web.rs` becomes `services/web/{mod,fetch}.rs` to stay under the size limit.
- **Layout.** `layout(blocks, theme, width, images: &dyn Fn(&str) -> ImageView)`. `ImageView` is a layout-owned value type: `Shown { width, height }` or `Placeholder { note: Option<(Note, bool)> }`, where the `bool` says whether the note is clickable. The UI builds the lookup from session state, once per generation.
  - A **shown** image gets a `LaidImage { src, bounds, link }` for painting. It also gets a zero-paint `Fragment`, with text `[image: alt]`, its link and `Joint::Break`. Selection, copy, `link_at`, hover and the pointer cursor then work unchanged.
  - A **placeholder** is the description as text (link-styled if it is linked), then the note as a second, smaller line. The note gets a `Fragment` tagged as an image action.
  - `layout::anchor::keep(old_page, new_page, scroll) -> scroll` finds the first fragment at or below the top of the viewport, then returns the scroll that keeps it at the same screen position. It is pure and tested.
    - The fragment is identified by a stable key: its **block index** and its byte offset within the block, which every fragment carries. Block indices don't change when an image arrives, because the split happens in the model, before layout. Fragment positions in the list do change.
- **UI.**
  - The document widget draws `LaidImage`s with `image::Renderer::draw_image`, culled to the viewport.
  - A click on an action fragment sends `LoadImage`. A click on a linked image follows the link.
  - `Pages` treats a generation change like a width change: coalesced and deferred with the existing `SETTLE` timer, here at most one per 150 ms. Each relayout applies `anchor::keep`, and re-applies a pending fragment.
  - Ctrl/Cmd+Shift+I sends `LoadPageImages`.
  - The toolbar gains the Images `pick_list`.
  - The runner runs `LoadImage` through `services::images::load` on a background thread.

## Steps

### Step 1: ADR-004 and doctrine — done

**Complexity**: trivial
**GREEN**: Use `invest-adr` to record ADR-004, covering the design notes above.
- Rejected: lazy `from_bytes` decoding, all codecs, SVG, and full-resolution decoding.
- Recorded: the limits, the peak-memory estimate, and that the binary-size cost is measured and added in step 10.

In ARCHITECTURE.md:
- the stack table gains `image`
- the Services row gains `images`
- the Session row and Import Direction gain `images` and `iced::widget::image::Handle` as a value type
- the Layout row gains image boxes and `anchor`

**Files**: `vector/decisions/ADR-004-images.md`, `ARCHITECTURE.md`
**Commit**: `docs(vector): decide how images are decoded and drawn (ADR-004)`

### Step 2: Images become blocks

**Complexity**: standard
**RED**:
- Model unit tests:
  - an image alone in a paragraph
  - a leading, a middle and a trailing image
  - two images in a row
  - whitespace trimming and dropping empty parts
  - emphasis across the cut
  - `[see ![a](b.png)](url)`
  - an image directly after a heading
  - list items and quotes
  - headings and table cells, which flatten
  - code spans and code blocks, which stay literal
  - empty `src` and empty alt (`[image]`)
  - `plain_text`
- The two step-2 scenarios.
- Replace `images_show_alt_text`.
- **Before GREEN**, write the expected snapshot changes into the step's commit message. Find them with `grep -l '!\['` over `corpus/`. Any other diff is a failure.

**GREEN**: `Inline`, `flatten`, `split`, and `Block::Image` laid out as its description text for now.
**Files**: `src/model/{mod,blocks,inlines}.rs`, `src/layout/flow.rs`, `features/images.feature`, `tests/acceptance/steps/images.rs` (new), the snapshots
**Commit**: `feat(model): make images blocks of their own`

### Step 3: Where an image comes from

**Complexity**: complex (security-sensitive)
**RED**: `resolve` unit tests.
- **Bases:** file bases (relative, `../`, absolute, `~/`, `file://`, `http(s)`) and web bases (relative, root-relative, scheme-relative, query, percent-encoding, whitespace).
- **Credentials:** dropped.
- **Refused:** `file:`, drives and shares from a web page; `data:` and `javascript:`; empty. Also local-network sources from a public page: `127.0.0.1`, `2130706433`, `0x7f.1`, `[::1]`, `[::ffff:127.0.0.1]`, `localhost.`, `10.x`, `169.254.x`.
- **Property test:** for any web base and any source, `resolve` never returns `File`. For a public base, it never returns a local-network host.
- **2a regression:** the `links` and `links::page` tests stay green through the move to `href.rs`.

**GREEN**: `session/href.rs` (moved helpers) and `session/images/{mod,resolve,limits}.rs`.
**Files**: `src/session/{href,mod}.rs`, `src/session/images/*`, `src/session/links/{mod,page}.rs`
**Commit**: `feat(session): resolve image sources by the link rules`

### Step 4: The image setting

**Complexity**: standard
**RED**:
- `store` round-trips: a missing key, an unknown value, and each mode.
- The reducer persists `ChooseImageLoading`.
- The content test asserts the exact menu labels and every note string in AC 6.
- The step-4 scenarios.

**GREEN**: `ImageLoading`, the `Settings.images` field and its persistence, the toolbar `pick_list`, and the `content` strings.
**Files**: `src/session/mod.rs`, `src/services/store.rs`, `src/content.rs`, `src/ui/toolbar.rs`, `features/images.feature`, `tests/acceptance/steps/images.rs`
**Commit**: `feat(session): let the reader choose how web images load`

### Step 5: Read, fetch and decode images

**Complexity**: complex (I/O, untrusted input)
**RED**: `services::images` and `services::web` tests. Fixtures are generated in the test with the `image` crate. WebP is encoded losslessly by the same crate; if that encoder isn't available, a hand-written byte constant is used.
- **Formats:** PNG (RGB, RGBA, grey, indexed), GIF (first frame), JPEG and WebP decode to the right size.
- **Bytes decide:** untyped and mislabelled images decode.
- **Refused types and bytes:** `text/html` without decoding; SVG, AVIF and garbage bytes as `Unsupported`.
- **Size limits:**
  - 16,384×1 is accepted and 16,385×1 is `TooLarge`.
  - 25 MP is accepted and 25 MP plus one row is `TooLarge`. The refusal comes from header dimensions, with no pixel buffer: assert on the error kind, not on time.
  - Bodies of exactly 16 MiB and 16 MiB + 1, including a gzip body that inflates past the cap.
- **Downscale:** 4,000×100 becomes 2,048×51.
- **Fetching:**
  - a redirect is followed
  - a redirect to a local-network host from a public URL is `Unavailable` (unit level, through `next_hop`)
  - a truncated body is `Unavailable`
  - a deadline is `Unavailable`
  - `Accept` is the image header, and no cookie or `Referer` is sent
- **Local files:** `.txt` and `.svg` are `Unsupported` without reading.
- **Error mapping:** both pure `FetchError` mappings, one test per variant.
- **2a regression:** every page `fetch` test stays green.

**GREEN**:
- `Want`, `IMAGE_ACCEPT` and `image_policy` in `session::web`
- `fetch_bytes`
- `services/web/{mod,fetch}.rs`
- `files::read_capped`
- `services::images::load`, with the decode mutex
**Files**: `Cargo.toml`, `src/session/web/{mod,body}.rs`, `src/services/{files,images,mod,runner}.rs`, `src/services/web/*`
**Commit**: `feat(services): read, fetch and decode images within limits`

### Step 6: The image planner and cache

**Complexity**: complex (state machine)
**RED**: Reducer and planner tests, with concrete inputs.
- **Loading:**
  - a page with 6 images emits 4 `LoadImage`s, and each `ImageLoaded` emits the next
  - the cap holds across pages
  - a repeated location on a page or across pages is loaded once
- **Clicks:**
  - in on-click mode a page emits nothing until `LoadImage`
  - a second click while loading emits nothing
  - in off mode a page emits nothing, and a click on an off note is ignored
  - a click retries a failed image
- **Local images** load in every mode.
- **Transitions:** all six setting transitions from AC 7, including that nothing shown is hidden and nothing running is cancelled.
- **Shortcut:** `LoadPageImages` loads waiting, off and failed images and leaves `Settings` alone.
- **Leaving and returning:**
  - navigating away drops the queue, and running loads still finish into the cache
  - `ImageLoaded` for a location the current page doesn't use doesn't bump the generation
  - a failed image is retried on revisit and on reload
- **Refused** images make no effect.
- **Cache:** with a budget of 2 images and 3 loaded, the least recently used is evicted, a touched image survives, and an image on the current page is never evicted.
- **Property test:** over random event sequences, at most 4 loads are in flight, the in-flight count equals the number of `Loading` entries, and the cache is within budget except for pinned images.

**GREEN**: `session/images/{planner,cache}.rs`, plus `State.images`, events and effects. The reducer gets one delegating arm per event.
**Files**: `src/session/images/*`, `src/session/{mod,reducer,tests}.rs`
**Commit**: `feat(session): load a page's images, four at a time, with a cache`

### Step 7: Lay out images

**Complexity**: standard
**RED**: Layout tests:
- shown images at their own size, at the column width (exact and wider), 1×1 and 1×10,000
- rounding
- never scaled up
- placeholders with and without a clickable note
- linked descriptions
- `LaidImage` contents
- the zero-paint fragment, so select all and copy yields `[image: alt]` lines and `link_at` on a linked image returns its link
- `anchor::keep` with a block growing above, at and below the viewport top, and with a placeholder becoming a shown image both above and inside the viewport

Also:
- a golden snapshot of `corpus/images.md`, which holds placeholders only
- a corpus sweep: parse and lay out every corpus file with images, and check there is no panic and every image becomes a block

**GREEN**: `ImageView`, the image boxes, the notes, the fragments and `layout::anchor`.
**Files**: `src/layout/{mod,flow,select,anchor}.rs`, `corpus/images.md`
**Commit**: `feat(layout): lay out images and keep the reading position`

### Step 8: Draw images and wire them up

**Complexity**: standard
**RED**:
- Pure tests for hit-testing a point: follow a link, load an image, or nothing (an off note).
- Pure tests for the `Pages` cache: a generation bump relays out, deferred and coalesced; the same generation reuses the layout; a pending anchor is re-applied.
- The step-7 scenarios: drawn images, formats, can't-be-shown, wide, a local file showing web images, a web page refusing the disk, repeats, retry, reload, copy and HTML img. The world handles `LoadImage` through `services::images::load`, and its 127.0.0.1-only guard covers images. A unit test checks that the guard rejects any other host.

**GREEN**:
- the document widget draws and hit-tests images, with the pointer cursor
- `Pages` handles the generation
- the shortcut, the runner and the menu are wired up
**Files**: `src/ui/{document,pages,app,toolbar}.rs`, `src/services/runner.rs`, `tests/acceptance/**`, `features/images.feature`
**Commit**: `feat(ui): draw images in the page`

### Step 9: Setting, clicks, links and loading, end to end

**Complexity**: complex (cross-layer)
**RED**: The step-9 scenarios. World additions:
- per-route holding for image loads, reusing 2a's hold and release, with "every slow image finishes loading"
- an injectable cache budget
- the step definitions listed in the feature, in `steps/images.rs`

**What this step is.** Most of this behaviour was built and unit-tested in step 6, so these scenarios characterise it end to end and may pass at once. Any that fail are real wiring bugs.
**GREEN**: Whatever the scenarios expose.
**Files**: `features/images.feature`, `tests/acceptance/**`
**Commit**: `test(acceptance): image settings, clicks, links and caching`

### Step 10: Docs, doctrine and manual check

**Complexity**: trivial
**GREEN**:
- **README:** images, the Images menu, Ctrl/Cmd+Shift+I, and a privacy note (loading web images tells their hosts that a post was read).
- **VECTOR.md:**
  - 2b is built
  - the remote-images open question is resolved
  - "Image semantics" stays open for phase 3
  - stacked rather than floated leading images is the open item against the scripting.com ship criterion
  - Research Status
- **`features/README.md`:** `images.feature` and its generated fixtures.
- **ADR-004:** add the measured binary-size cost.
- **Manual checklist** (record the results in the PR):
  - markdownWeb examples 2 and 3: images drawn, text below them
  - a scripting.com archive post with a leading image: the image sits above its paragraph, and the reading position holds as it loads
  - an animated GIF: the first frame shows
  - a page with a broken image: the note shows, and a click retries
  - each Images mode, plus Ctrl/Cmd+Shift+I in "off"
  - a `#anchor` link into an image-heavy post lands on its heading
**Files**: `README.md`, `VECTOR.md`, `features/README.md`, `vector/decisions/ADR-004-images.md`
**Commit**: `docs: describe images`

## Complexity Classification

| Rating | Steps |
|---|---|
| trivial | 1, 10 |
| standard | 2, 4, 7, 8 |
| complex | 3, 5, 6, 9 |

## Pre-PR Quality Gate

- [ ] `cargo test` passes (unit and acceptance)
- [ ] `cargo build --release` passes, with the binary size recorded in ADR-004
- [ ] `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass
- [ ] `/code-review` passes, including `security-review` on steps 3 and 5 and `a11y-review` on the menu, the notes and the shortcut
- [ ] The step-10 manual checklist is done and recorded in the PR
- [ ] All six CI checks are green
- [ ] Documentation is updated

## Risks & Open Questions

- **Right-margin images.** scripting.com floats leading images to the right, with text beside them. 2b stacks them above the text instead. Floats are a layout-engine feature of their own. Revisit after the manual check; a maximum image height may also be needed.
- **Images in headings, table cells and link text inside headings** stay as descriptions. Rare in the archive.
- **Animated GIFs and WebPs** show their first frame.
- **Captions.** Alt text often reads as a caption ("Night on the El Train, 1918."). Like browsers, 2b shows it only in place of a missing image. A caption could be a phase 4 reader control.
- **Privacy.** Loading web images by default tells their hosts that a post was read. No cookies, `Referer` or credentials are sent, but the IP address and User-Agent are. "on click", "off" and the README note answer this.
- **HiDPI.** Images are drawn at logical pixels, so 1× images look soft on 2× screens, as in browsers.
- **Badges.** A row of README badges stacks vertically.
- **Keyboard and screen readers.** iced can't focus items inside the page, so Ctrl/Cmd+Shift+I is the keyboard route, and screen readers aren't told when an image arrives. Both are platform limits, tracked with 2a's keyboard note.
- **No image viewer.** Opening an image at full size, saving or copying it are follow-ups. One option: opening an unlinked image in the system viewer.
- **Large textures.** 16,384-pixel heights may exceed some GPUs' texture limits. Downscaling to 2,048 wide bounds most images; a very tall one is checked on CI in step 8.
- **Binary size.** The codecs add to the binary. It is measured in step 10.
- **Delivery.** One PR with 10 steps. If review gets heavy, steps 1–4 can ship first as their own PR, with no visible change beyond the menu.

## Plan Review Summary

Four personas reviewed revision 1. The three with blockers re-reviewed revision 2, and all four now approve. Revision 3 applies their remaining warnings, which were small edits. Two iterations is the limit, so there was no third run.

| Reviewer | Revision 1 | Final |
|---|---|---|
| Acceptance | needs-revision (7 blockers) | approve (2 warnings, both fixed) |
| Design | needs-revision (1 blocker, 7 warnings) | needs-revision on the 3-warning rule only, with no blockers; all 3 fixed in revision 3 |
| UX | needs-revision (2 blockers) | approve (2 warnings, both fixed) |
| Strategic | approve (3 warnings, all fixed) | — |

**What changed:**
- **Reading position:** arriving images are coalesced into one relayout per 150 ms, `anchor::keep` holds the position using a stable block key, and a pending `#anchor` is re-applied.
- **Copy, hover and link clicks on shown images:** each image also lays out a zero-paint fragment.
- **What the reader sees:** each state has its own note, and failed images retry on a click.
- **Keyboard:** Ctrl/Cmd+Shift+I loads this page's images without changing the setting. The "off" note names the shortcut.
- **The menu:** "Web images: automatic / on click / off", with all six transitions specified.
- **Concurrency:** a cap of 4 across the app; leaving a page drops its queue; repeated images are fetched once.
- **Cache:** an LRU with an injectable budget and the current page pinned.
- **Failures:** not cached, so a revisit or reload retries them.
- **Memory:** decodes run one at a time, images are downscaled to 2,048 pixels wide, and the limit is 25 MP.
- **Fetching:** one fetch loop returning a neutral `FetchError`, with pure mappings to page errors and image problems.
- **Planner state:** one app-wide state table.
- **Requests:** no `Referer`, no cookies and no credentials, all stated and tested.
- **Scenarios:** placed in the steps that implement them, with deterministic request counts.

**Warnings kept as risks or observations:**
- Shortcut feedback when there is nothing to load.
- The clickable note and the linked description share one link style.
- Two of the six setting transitions are unit-tested only.
- AC 11 is covered by unit tests and the manual checklist, not acceptance.
- Right-margin floats, captions, HiDPI, badges and the image viewer are under Risks.
- Large textures are checked on CI.
