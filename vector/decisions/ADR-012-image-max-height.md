# ADR-012: Images stay stacked and are capped in height

**Date:** 2026-11-03
**Status:** accepted
**Deciders:** gBRETT (product owner)

## Context

VECTOR.md carried an open item, "Leading images and tall images", against the ship criterion "renders the scripting.com archive". Images are drawn stacked in the flow, at their own size, scaled down only to fit the column (ADR-004). Two questions followed: should a leading image float to the right of the text, as it does on scripting.com, and should a very tall image be allowed to fill the window above a post's first line? Plan: `vector/plans/image-max-height.md`.

Checking the archive on 2026-11-03 changed how the first question reads. On the live page (`scripting.com/2024/03/31.html`) the leading image is `<img class="imgRightMargin" style="float: right; …">`. In the Markdown (`corpus/markdownweb/2024/03/31/a161730.md`) it is `![](https://imgs.scripting.com/2023/11/30/goodhumortruck.png)` at the start of the paragraph's text, plus an `image` front matter field, and no float. The float is the site's HTML and CSS. Markdown encodes none. The live site also puts no maximum on image height.

## Decision Drivers

- VECTOR.md, design principle 2: the reader owns the presentation. Documents bring content and the browser brings the style.
- VECTOR.md: "Not an HTML browser". A float is a presentation choice made by one site's CSS.
- ADR-001: layout is our own engine, pure, and takes only a width. `Pages` caches a page by look (font, size, spacing, palette, colours) and width.
- ADR-007: reader-controlled sizes follow the reader's text size.
- ADR-004: images are stacked and scaled down only to the column; decoded sizes already have limits.
- The 640 by 480 images in the corpus and the Phase 3 goldens must not change.
- A window of 620 by 360 is the smallest the app allows (ADR-008).

## Options Considered

### Option A: Float leading images to the right

Wrap text beside the image, as scripting.com does.

**Pros:**
- Posts look like the source site.

**Cons:**
- The layout engine only stacks blocks, so floats are a feature of their own (wrapping, clearing, narrow columns).
- It puts one site's styling over the reader's choice, for something the Markdown does not say.

### Option B: A maximum height as a fraction of the window

Cap images at, say, 80% of the window height.

**Pros:**
- A tall image never exceeds the window on any window.

**Cons:**
- Layout takes only a width. Window height would enter the `Pages` key and every vertical resize would lay the page out again, which ADR-001's cache exists to avoid.

### Option C: A fixed pixel maximum that ignores text size

**Pros:**
- Simple.

**Cons:**
- A reader who enlarges text for low vision would get no larger images, against ADR-007.

### Option D: A maximum of 24 times the text size (432 px)

**Pros:**
- Closer to a small window.

**Cons:**
- It would scale 640 by 480 images, which the corpus, the goldens and most posts use.

### Option E: No maximum

**Pros:**
- Nothing to build.

**Cons:**
- A tall image (a screenshot, an infographic) can fill the window above the first line of text.

### Option F: Stacked, scaled to fit a maximum of 30 times the text size (chosen)

## Decision

**We will keep images stacked and scale a shown image down to fit both the column and a maximum image height of 30 times the body text size.** The image keeps its aspect ratio and is never cropped or scaled up. The maximum is 540 px at the default 18 px, 360 px at 12 and 960 px at 32.

Stacking reads the Markdown faithfully and leaves the presentation to the reader (Option A rejected). The maximum is a length derived from `body_size`, like line height, so it is a function of `Look`, which is already in the `Pages` key: no cache key, `ImageView` or planner change (Option B rejected). It follows text size (Option C rejected). At 30 times, a 640 by 480 image is not scaled (Option D rejected).

- **Implementation.** `Theme::image_max_height()` in `src/theme/mod.rs` and a pure `fit(own, max_width, max_height)` in `src/layout/image.rs`: one scale factor, the smaller of the column's and the maximum's, never above 1, each side rounded to whole pixels, at least 1.
- **Boxes.** The click area, the selection fragment (`[image: alt]`), the Tab ring and the painted picture all read the one scaled box.
- **Unchanged.** Placeholders and notes, the decode limits (a very large image still says "too large to show"), and the planner.

## Consequences

**Positive:**
- A tall image no longer pushes the first line of text far below the top of the page. A test checks the first line starts within one block gap of the maximum.
- No gap against the ship criterion "renders the scripting.com archive": the Markdown is rendered as written, and the float was never in it.
- Larger text gives larger images.

**Negative:**
- A tall thin image shrinks in width too (200 by 3000 becomes 36 by 540) and can become unreadable. There is no width floor: a floor would break either the aspect ratio or the maximum.
- The reader cannot open a capped image larger. The follow-up decision would be "open at full size", which needs ADR-010's consent rule for anything that leaves the app.
- At the 620 by 360 minimum window a 540 px image is still taller than the window, and at 32 px text the maximum is 960 px. It is bounded, and the text is one scroll away, but the stated aim holds for ordinary windows.
- How many archive posts have a leading or a tall image is unmeasured. The maximum is a safeguard, not a response to a counted problem.

**Neutral:**
- The 30 times multiple was chosen, not measured. It is one constant to change.
- Posts that float an image on scripting.com read differently here. That is the reader's presentation, not a defect.

## Related Decisions

- ADR-001: Native rendering with our own layout engine (layout takes a width; the cache is keyed by look).
- ADR-004: Images (the decode limits and the stacked layout this builds on).
- ADR-007: Reader controls (sizes follow the text size).
- ADR-008: Keyboard access (the minimum window, the Tab ring).
- ADR-010: Web links open in the app (the consent rule a future "open at full size" would need).
- ADR-011: The source view (shows the Markdown, which has no float).
