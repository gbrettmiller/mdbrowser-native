# Plan: Stacked images with a maximum height

**Created**: 2026-11-03
**Branch**: feat/image-max-height (to create from main)
**Status**: approved 2026-11-03 (30× the text size; follows text size)

## Goal

Settle the open item "Leading images and tall images" in `VECTOR.md`. It has two halves, and checking the archive (2026-11-03) changes how the first reads. **Float:** the float is Dave Winer's site presentation, not part of his Markdown. On the live page (`scripting.com/2024/03/31.html`) the leading image is `<img class="imgRightMargin" style="float: right; …">`; in the Markdown (`corpus/markdownweb/2024/03/31/a161730.md`) it is `![](…)` at the start of the paragraph's own text, plus an `image` front matter field, which is the only trace of it. Markdown encodes no float, so stacking the image above the text is a faithful reading of the file, and the reader chooses the presentation. That half is closed by decision, with no gap against the ship criterion "renders the scripting.com archive" (read as rendering its Markdown). **Tall images:** the live site puts no maximum on image height either, so a tall image is tall there too; this plan caps it, so a tall image cannot fill the window above a post's first line. That is what this plan builds. (How many archive posts have a leading or a tall image is unmeasured; the cap is a safeguard, not a response to a counted problem.) Layout takes only a width and is cached by look (ADR-001, `Pages`), so the cap is a theme-derived length, not a fraction of the window: a window-height cap would relayout on every vertical resize and put the window into the layout key.

## Decisions

1. **Stacked, never floated.** No float layout. VECTOR's open question is answered "a maximum image height". "Float layout" goes in ADR-012 as a rejected option, for this reason: a float is a site's presentation choice (here scripting.com's CSS and inline style), and Markdown carries none, so floating would put the writer's site styling over the reader's choice.
2. **The maximum image height is `30 × body text size`**, a length derived from `body_size` the way line height is (`Theme::image_max_height()`, not a stored field). It is a function of `Look`, which is already in the `Pages` key, so no cache key changes. At the default 18 px that is 540 px, so the 640 × 480 images the goldens and corpus use are untouched. It follows the reader's text size (12 px: 360 px; 32 px: 960 px), as every other reader-controlled size does (ADR-007), so a reader who enlarges text for low vision also gets larger images. The values were chosen, not measured; they are in ADR-012 and can change in one place. **Confirmed by gBRETT (2026-11-03):** 30×, following text size.
3. **Scale down, never crop, never scale up.** A shown image is scaled by the smaller of "fit the column" and "fit the cap", so both limits hold and the aspect ratio is kept (whole pixels, at least 1 by 1). A tall thin image therefore also gets narrower. It stays left-aligned at the column edge, as now.
4. **Everything that rides on an image's box follows the scaled box**: the link click area, the selection and copy fragment (still `[image: alt]`), the Tab stop's ring, and the painted picture. They all read `LaidImage.bounds` today, so no new path is added.
5. **No expand or open-in-full control in this plan.** The reader sees the image at the capped size. If readers ask to see the whole image, that is its own decision (an image that opens in the system browser would need the same consent rule as other web links, ADR-010), and is listed under Risks.
6. **Unchanged:** placeholders and their notes, the decode limits (an image taller than the decoder allows still says "too large to show"), the image planner and the `ImageView` type (`Shown` keeps carrying the decoded size; layout does the scaling).

## Acceptance Criteria

1. [ ] **The maximum height.** An image whose natural height is over the maximum image height is laid out at exactly that height, with its width scaled by the same factor, rounded to whole pixels, at least 1. An image at the maximum (540 tall at default) or under it, and not wider than the column, is laid out at its own size; one pixel over (541) is 540. A thin image (1 × 10,000) is 1 × 540. Nothing is scaled up.
2. [ ] **Both limits.** An image both wider than the column and taller than the cap is scaled once, by the smaller factor, so it is within the column and within the maximum; tests assert the exact expected width and height (rounded from the factor), not a tolerance.
3. [ ] **The cap follows the reader.** At each text size from 12 to 32 px the maximum is 30 times the body size (360 to 960; body sizes are whole pixels), and a larger text step makes the same image taller. The default look's maximum is 540, so a 640 × 480 image is unchanged and the Phase 3 golden snapshots do not change.
4. [ ] **A tall image does not hide the text.** With a tall image above a paragraph at the default size, the paragraph's first line has its top at or above (image top + maximum height + one block gap) (a pure layout test, same wording in the scenario), so it is on the first screen of any window at least 700 px tall.
5. [ ] **Image behaviour holds on a capped image**, each with a named test in step 2: a click inside the scaled box follows the link and one outside it does not (`a_capped_image_is_clickable_only_inside_its_scaled_box`); Tab reaches it as one stop whose ring rectangle equals the scaled box (`a_capped_image_is_one_stop_with_the_scaled_ring`, built on the stop tests in `src/layout/stops.rs`); select all and copy give `[image: alt]` once (the existing selection test on a capped image); a very thin linked image (1 × 540) still has a one-pixel box, one stop and a link; and when a tall image arrives above the viewport, the held scroll position moves by the scaled height, not the natural one (a pure test on the existing keep-place arithmetic, located in step 2).
6. [ ] **Nothing else moves.** Placeholders, notes, decode limits and the `ImageView` contract are as before. Goldens, the existing image scenarios and the keyboard scenarios pass unchanged except the one test that pins the old unbounded height (`a_shown_image_keeps_its_size_when_it_fits`, a 1 × 10,000 image), which is updated to the new rule.
7. [ ] **Doctrine.** ADR-012 (via `invest-adr`, status accepted) records the decision and the rejected options; VECTOR's open question is replaced by the decision and a Direction line; README's images paragraph and ARCHITECTURE's theme and layout rows name the cap; ADR-004 gets a "Related" pointer.
8. [ ] **CI and screenshots.** Test, build, clippy and fmt are green on all four platform jobs. Screenshots through `scripts/shoot.sh` on a corpus page with a tall image are pass or fail: in the default look the image's bottom edge is at or above the maximum and the first line of text is visible without scrolling; at 32 px text the same image is taller and the first line is within one PageDown; at the 620 by 360 minimum window the first line is within one PageDown. A failure moves the multiple (24) or fixes the cap, and the plan is amended.

## User-Facing Behavior

```gherkin
@images
Feature: Tall images are capped, not floated

  # Images are stacked in the flow at their own size, never wider than the column
  # and never taller than the maximum image height, which follows the text size.

  Scenario: A tall image is scaled down to the maximum image height, aspect ratio kept
    Given an image file "skinny.png" of 200 by 3000 pixels
    And a file "post.md" containing "![skinny](skinny.png)\n\nThe first line of text."
    When I view "post.md"
    Then the image "skinny" is shown at 36 by 540

  Scenario Outline: The maximum image height is exact at the boundary
    Given an image file "edge.png" of 100 by <natural> pixels
    And a file "post.md" containing "![edge](edge.png)"
    When I view "post.md"
    Then the image "edge" is shown at <width> by <height>
    Examples:
      | natural | width | height |
      | 540     | 100   | 540    |
      | 541     | 100   | 540    |
      | 10000   | 5     | 540    |

  Scenario: An image under the maximum is not changed
    Given an image file "photo.png" of 640 by 480 pixels
    And a file "post.md" containing "![photo](photo.png)"
    When I view "post.md"
    Then the image "photo" is shown at 640 by 480

  Scenario: An image that is both wide and tall fits both limits
    Given an image file "big.png" of 4000 by 6000 pixels
    And a file "post.md" containing "![big](big.png)"
    When I view "post.md"
    Then the image "big" is within the column and within the maximum image height

  Scenario: The first line of text is not pushed out of the window
    Given an image file "skinny.png" of 200 by 3000 pixels
    And a file "post.md" containing "![skinny](skinny.png)\n\nThe first line of text."
    When I view "post.md"
    Then the first line of text has its top at or above the image top plus the maximum image height plus one block gap

  Scenario: Larger text raises the maximum image height
    Given an image file "skinny.png" of 200 by 3000 pixels
    And a file "post.md" containing "![skinny](skinny.png)"
    And I viewed "post.md"
    Then the image "skinny" is shown at 36 by 540
    When I make the text larger 3 times
    Then the image "skinny" is shown at 48 by 720

  Scenario: A capped image is still one stop and one line of copied text
    Given an image file "skinny.png" of 200 by 3000 pixels
    And a file "post.md" containing "[![skinny](skinny.png)](other.md)"
    When I view "post.md"
    Then there is 1 link stop
    And the document text is "[image: skinny]"
```

(The click, ring and later-arrival checks of criterion 5 are the named pure tests in step 2; the feature keeps the behavioural contract. The `shown at W by H` expectations use the default look and 720 px column; widths are 200 × 540 / 3000 = 36 and 100 × 540 / 10000 = 5.4 → 5, at 24 px text 200 × 720 / 3000 = 48. The two-limit scenario's exact numbers are asserted by its step from the laid-out column, since the decoder may downscale a 4000-pixel-wide image to 2,048 first.)

## Steps

### Step 1: The maximum image height is a derived theme length

**Complexity**: standard
**RED**: `src/theme/mod.rs` tests: `image_max_height()` is `30 × body_size` at every text step; the default look gives 540; it is at least 480 so the 640 × 480 images in the corpus and goldens are not affected; a custom or preset palette does not change it (it is geometry, not colour).
**GREEN**: An `IMAGE_MAX_HEIGHT_RATIO` constant and `Theme::image_max_height()` (derived from `body_size`, like line height). ARCHITECTURE's theme row names it as a derived length.
**REFACTOR**: None needed.
**Files**: `src/theme/mod.rs`
**Commit**: `feat(theme): add the maximum image height, derived from the text size`

### Step 2: Layout scales an image to fit, with its scenarios

**Complexity**: standard
**RED**: First list every reader of an image's decoded size and of `LaidImage.bounds` (`grep -rn "bounds\|ImageView::Shown" src/ui src/layout`) to confirm that the click area, selection, ring and painter all use `bounds` and nothing uses the decoded size directly. Then, in `src/layout/image.rs`: a table test for a pure `fit(own, max_width, max_height) -> (w, h)` (never scales up; the smaller factor; whole pixels; at least 1 by 1; 200 × 3000 → 36 × 540; 100 × 541 → 100 × 540; 100 × 540 unchanged; 100 × 10,000 → 5 × 540; 1 × 10,000 → 1 × 540; 4000 × 6000 inside both limits; 640 × 480 unchanged); and the named tests of criterion 5 (`a_capped_image_is_clickable_only_inside_its_scaled_box`, `a_capped_image_is_one_stop_with_the_scaled_ring`, the selection test on a capped image, a thin linked image, the later-arrival keep-place test; the last is in `src/ui` beside the existing keep-place tests). The paragraph after a capped image starts one block gap below the scaled box. Update `a_shown_image_keeps_its_size_when_it_fits` (the 1 × 10,000 case). Also the scenarios above in `features/images.feature` with steps in `tests/acceptance/steps/images.rs` (`is within the column and within the maximum image height`, `the first line of text has its top at or above …`), built on the existing "an image file … of … pixels" fixture, `shown at W by H`, `I make the text larger N times` and `there is N link stop`. Confirm before writing the scenario that the harness's laid-out page exposes the first text fragment's top.
**GREEN**: `fit` as a pure function and `picture` using it with the column width and `theme.image_max_height()`. Steps and scenarios and code land in this one commit, so no commit is red.
**REFACTOR**: None needed.
**Files**: `src/layout/image.rs`, `src/ui/` (the keep-place test), `features/images.feature`, `tests/acceptance/steps/images.rs`
**Commit**: `feat(layout): scale an image down to the maximum height, aspect kept`

### Step 3: Doctrine and screenshots

**Complexity**: standard
**RED**: A manual checklist (not a test): VECTOR (open question gone, decision in § Direction, the float is recorded as the site's presentation, not a gap in the ship criterion), README, ARCHITECTURE and ADR-004 "Related" mention the maximum image height; ADR-012 exists. `scripts/shoot.sh` already handles the screenshots.
**GREEN**: ADR-012 through `invest-adr`: stacked not floated (the float is scripting.com's CSS and inline style on its HTML page; its Markdown has `![](…)` in the paragraph and an `image` front matter field, and no float; cite `corpus/markdownweb/2024/03/31/a161730.md` and the live page); the multiple and its value; text size scaling; scale-not-crop; no way to open a capped image larger, and the named follow-up (open it at full size, which needs ADR-010's consent rule); the rejected float; the rejected window-height cap, because `Pages` is keyed by look and layout takes only a width; the cap is a function of `Look`, so no cache key changes. VECTOR, README and ARCHITECTURE updated; a tall image in `corpus/` for the screenshots (a small generated PNG and a post with a leading tall image and text under it). Screenshots on workspace 3 (ask gBRETT for an empty workspace first), judged by criterion 8.
**REFACTOR**: None needed.
**Files**: `vector/decisions/ADR-012-image-max-height.md`, `VECTOR.md`, `README.md`, `ARCHITECTURE.md`, `vector/decisions/ADR-004-images.md`, `corpus/` (one post, one image)
**Commit**: `docs: ADR-012 and the maximum image height in VECTOR, README and ARCHITECTURE`

## Complexity Classification

All steps are `standard`: one derived length, one pure function and one change in one layout function, tests and docs, inside existing patterns (the layout stays pure and cached by look).

## Pre-PR Quality Gate

- [ ] `cargo test` passes (units, scenarios and goldens, with no golden changed)
- [ ] `cargo build` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] `/code-review` and `arch-review` pass
- [ ] ADR-012, VECTOR, README, ARCHITECTURE updated
- [ ] Screenshots read (light, 32 px text, 620 × 360)
- [ ] Ask before pushing or opening the PR

## Risks & Open Questions

- **The maximum is not the window.** Layout cannot see the window height, so at the 620 × 360 minimum window a 540 px image is still taller than the window. It is no longer unbounded and the text is one scroll away, but the stated goal ("not the whole screen") holds for normal windows, not the smallest. Mitigation: the value is one constant; a smaller multiple (24, 432 px) is the alternative if the screenshots at 620 × 360 look wrong. A window-height cap would need the window height in the `Pages` key and a relayout on every vertical resize, which ADR-001's cache design avoids.
- **The cap follows text size.** A reader at 32 px text gets images up to 960 px tall, which is more than a typical window. This is consistent with ADR-007 but may surprise. Alternative: a fixed 540 px at every size. gBRETT to confirm or ask for the fixed value.
- **No way to see the whole image.** A capped image cannot be opened larger. A "click to open the image in the browser" control is a separate decision (consent, ADR-010) and is not in this plan.
- **Tall thin images shrink in width too** (200 × 3,000 becomes 36 × 540). A tall image that matters (a screenshot, an infographic) can become unreadable, and nothing tells the reader it was shrunk. The follow-up (open at full size) is named in ADR-012; no width floor is added, because a floor would have to break either the aspect ratio or the maximum. That is what keeping the aspect ratio means, and such images are rare; screenshots will show whether the result is useful.
- **Rounding.** Scaling in whole pixels with a floor of 1 means very thin images (1 × 10,000) become 1 × 540 and distort slightly. The step 2 tests pin the rule.
- **Archive posts.** On scripting.com a leading image floats right with text beside it, but that is the site's HTML and CSS; the Markdown has no float, so stacked is a faithful reading. Not a gap. The archive's leading-image and tall-image frequency is unmeasured.

## Plan Review Summary

Four critics reviewed the first draft. Acceptance returned needs-revision (one blocker); UX needs-revision (warnings only); design and strategic approve with warnings. All are folded in above.

**Blocker (fixed):** criterion 5 promised click, ring and later-arrival behaviour with no test behind it. It now names each test, and step 2 locates the existing keep-place and stop tests.

**Warnings (fixed):** steps 2 and 3 merged so no commit is red; `fit` is a required pure function with a table test; the term is "maximum image height" throughout; criterion 4's wording matches its scenario; exact expected sizes replace tolerances; boundary (540, 541) and thin (1 × 10,000) cases added; `tall.png` renamed `skinny.png` (the old name is another fixture); the larger-text scenario asserts absolute sizes; criterion 8 is pass or fail; the cap is described as derived from `body_size`; a grep of the readers of `bounds` and the decoded size runs first in step 2; the float half is now stated as the site's presentation, not part of the Markdown, so it is closed by decision with no gap against the ship criterion (after checking the corpus samples and a live post).

**Accepted, not changed:** no way to open a capped image larger, and no width floor for thin images (named as a follow-up in ADR-012); at 620 × 360 a 540 px image is still taller than the window (screenshots decide whether to move to 24); whether tall images occur in the archive is unmeasured.

**Decided by gBRETT (2026-11-03):** 30× the text size, and the maximum follows text size.
