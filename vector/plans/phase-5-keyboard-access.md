# Plan: Phase 5: keyboard access

**Created**: 2026-10-30
**Branch**: `feat/phase-5`, from `main` after phase 4 (#13)
**Status**: built (2026-10-30)

## Goal

Make the whole app usable without a mouse. The 2026-10-30 Impeccable audit scored the UI 17/20 and found that every real gap is about operability, not looks: links can only be followed with a click, the page has no keyboard scrolling, and the toolbar and Settings controls are probably not reachable with Tab. The audit also flagged chrome text that ignores the reader's text size, and no minimum window size. Phase 5 fixes those, ahead of the downloadable-binary plan (`release-distribution.md`), so the first thing a stranger downloads is usable by everyone. VECTOR principle 2 gives the reader the presentation; this phase gives them the controls.

Spec artifacts: none from `/specs`. As in earlier phases, the Gherkin below is the spec and lands in `features/keyboard-access.feature`.

**Facts checked while planning (iced 0.14.2 source):**
- Only `text_input` and `text_editor` implement `Focusable`. `button` and `pick_list` do not, so Tab cannot reach the toolbar or the Settings controls today.
- `scrollable` handles no keys (only the Shift modifier for sideways wheel scrolling). Nothing scrolls the page from the keyboard. The comment at `keys.rs:82` ("Home scrolls; it needs Alt") is wrong and gets corrected.
- `Document` handles `Key::Character` only (copy and select all). It has no focus, no link stops and no Enter.
- `WINDOW_SIZE` is set but `min_size` is not.

## Scope

In:
1. **Links by keyboard.** The page takes focus. Tab and Shift+Tab move through links (and image notes) in reading order; a ring shows where focus is; Enter follows; the focused link is scrolled into view.
2. **Page scrolling by keyboard.** PageUp/PageDown, Space/Shift+Space, arrows, Home, End.
3. **Controls by keyboard.** Every toolbar button and Settings control can be reached with Tab and used with Enter, Space or the arrows.
4. **Chrome text size.** Decide and record whether it follows the reader's size, and move the chrome's sizes into `theme`.
5. **Narrow windows.** A minimum window size, and panels that give way to the page.

Out: screen-reader support (iced 0.14 exposes no accessibility tree; recorded as a known limit in ADR-008), mouse-free text selection (caret browsing), find in page, vim-style keys, remappable shortcuts, touch targets (a desktop app), any change to how documents are parsed.

## Acceptance Criteria

1. [x] **Links.** On a page with links, Tab focuses the first link and each further Tab the next, in reading order; Shift+Tab goes back. Past the last link, Tab moves on to the next control; before the first, Shift+Tab moves to the one before. A page is itself a stop before its links: Tab from the page enters the first link, and a page with no links takes one Tab to pass through.
   - The focused link shows a ring in the link colour (at least 2 px, outside the glyphs), drawn over every line a wrapped link covers.
   - Enter follows it. Image notes with a click action are stops too, and Enter does the click.
   - Moving focus scrolls the link into view (centred if it was off screen, untouched if already visible).
2. [x] **Opening a page focuses it**, so the scroll keys work straight away. Typing in a text field (address, home, colours) never scrolls the page.
3. [x] **Scrolling.** PageDown and Space move down by the viewport minus a three-line overlap; PageUp and Shift+Space up; Down and Up by one line; Home and End jump to the ends. Scrolling stops at both ends. PageUp and PageDown work wherever focus is; the other keys only when the page has focus.
4. [x] **Toolbar and panels.** Tab reaches Back, Forward, Reload, Home, the address bar, History and Settings in that order, then the page. Disabled buttons are skipped. Enter and Space press a button. In Settings, Tab reaches every control; on a choice (font, images, appearance, spacing), Up and Down change it (one step each, stopping at the ends), and the change applies and saves like a click. Esc closes an open panel and returns focus to the page.
5. [x] **Visible focus everywhere.** Every focus stop shows the same ring, derived from the palette, and it meets 3:1 against the page for light, dark and every custom set that passes `check`. A unit test checks it.
6. [x] **Chrome text.** The decision (below) is recorded in DESIGN.md and ADR-008. All chrome text sizes are `theme` tokens; none is below 13 px.
7. [x] **Narrow windows.** The window cannot be made smaller than 480 x 360. When History and Settings are both open and the page column would fall under 320 px, opening one closes the other. Resizing does not close anything.
8. [x] **Nothing else moves.** Shortcuts from earlier phases still work. Golden snapshots do not change (focus is drawn by the widget, not laid out). Colours stay in `theme`, text in `content`.
9. [x] **CI and screenshots.** Test, build, clippy and fmt are green on all four platform jobs. Screenshots through `scripts/shoot.sh` (focus ring on a link in light, dark and a custom palette; Settings with focus on a control; narrowest window with a panel open) are read and checked.

## User-Facing Behavior

```gherkin
@keyboard-access
Feature: Keyboard access
  As a reader who doesn't use a mouse
  I want to follow links, scroll and change settings from the keyboard
  So that I can read the web of Markdown

  Scenario: Tab walks the links in reading order
    Given I am viewing "index.md" which has 3 links
    When I press Tab
    Then the focus is on link 1
    When I press Tab
    Then the focus is on link 2

  Scenario: Shift+Tab walks back
    Given I am viewing "index.md" which has 3 links
    And the focus is on link 2
    When I press Shift+Tab
    Then the focus is on link 1

  Scenario: Tab past the last link leaves the page
    Given I am viewing "index.md" which has 3 links
    And the focus is on link 3
    When I press Tab
    Then the focus is on the next control after the page

  Scenario: A page with no links is passed straight through
    Given I am viewing "plain.md" which has no links
    When I press Tab
    Then the focus is on the next control after the page

  Scenario: Enter follows the focused link
    Given I am viewing "index.md" which has 3 links
    And the focus is on link 2
    When I press Enter
    Then I am viewing the second link's target

  Scenario: A wrapped link has one stop
    Given I am viewing "wrapped.md" whose only link wraps over 3 lines
    When I press Tab
    Then the focus is on link 1
    And the ring covers 3 lines

  Scenario: Focus scrolls the link into view
    Given I am viewing "long.md" with its last link below the first screen
    When I press Shift+Tab
    Then the last link is in view

  Scenario: Opening a page focuses it
    Given I typed "index.md" in the address bar
    When I press Enter
    Then the focus is on the page

  Scenario Outline: Scroll keys move the page
    Given I am viewing "long.md" at the top, in a window 600 pixels tall
    When I press "<key>"
    Then the page is scrolled to <offset>
    Examples:
      | key        | offset |
      | PageDown   | 513.6  |
      | Space      | 513.6  |
      | Down       | 28.8   |
      | End        | the end |

  Scenario: Scrolling stops at the ends
    Given I am viewing "long.md" at the top
    When I press PageUp
    Then the page is scrolled to 0
    When I press End
    And I press PageDown
    Then the page is scrolled to the end

  Scenario: Typing in the address bar doesn't scroll the page
    Given I am viewing "long.md" at the top
    And the focus is on the address bar
    When I press Space
    Then the page is scrolled to 0

  Scenario: Tab reaches the toolbar in order
    Given I am viewing "index.md" which has no links
    And the focus is on the address bar
    When I press Tab
    Then the focus is on the "History" button
    When I press Tab
    Then the focus is on the "Settings" button

  Scenario: Disabled buttons are skipped
    Given I have not followed any link
    And the focus is on the "Home" button
    When I press Shift+Tab
    Then the focus is on the "Reload" button

  Scenario: A choice changes with the arrow keys
    Given the Settings panel is open
    And the focus is on the "Spacing" control
    When I press Down
    Then the spacing is "Spacious"
    When I press Down
    Then the spacing is "Spacious"

  Scenario: Esc closes the panel and returns to the page
    Given the Settings panel is open
    When I press Esc
    Then the Settings panel is closed
    And the focus is on the page

  Scenario: The window has a minimum size
    Then the smallest window is 480 by 360

  Scenario: A second panel gives way when the page would be squeezed
    Given the window is 700 pixels wide
    And the History panel is open
    When I open the Settings panel
    Then the History panel is closed
    And the Settings panel is open

  Scenario: Two panels stay open when there is room
    Given the window is 1200 pixels wide
    And the History panel is open
    When I open the Settings panel
    Then both panels are open
```

## Decisions (proposed, 2026-10-30)

- **The `Document` widget becomes `Focusable`.** One iced focus stop for the page, with the links as stops *inside* it. Tab is handled by the widget while it has a next or previous link, and falls through to `focus_next` / `focus_previous` at the ends. This reuses iced's own focus order (address bar, text fields) and needs no focus tracking in the session.
- **Stops are a pure function of the page.** `Page::stops()` returns links and image-note actions in reading order, each with its page-space rectangles. A stop is identified by its position in that list, so a relayout (resize, text size) keeps focus on the same link as long as the document is the same. A new document clears it.
- **Focus lives in the UI layer, activation in the session.** Enter publishes the same `Event::Follow` / `LoadImage` a click does, so link following has one path (ARCHITECTURE: no browsing logic in `ui`). The reach rule (next, previous, leave) is a pure function, so the acceptance steps can test it without a window.
- **A `Focusable` wrapper for the chrome.** iced's `button` and `pick_list` cannot take focus, so `ui/focus.rs` wraps an element, makes it a stop, draws the ring and turns Enter and Space into its message. For the four choices in Settings it also turns Up and Down into "previous / next option" events, so no pop-up menu is needed from the keyboard. The pick lists stay for the mouse.
- **The ring is the link colour**, 2 px, outside the shape, so it needs no new colour token. For custom palettes `check` already guarantees link on page at 4.5:1, which is above the 3:1 non-text minimum.
- **Opening a page focuses it** (`operation::focus` on the document id once a new page is ready), as browsers do. Scroll keys other than PageUp and PageDown act only when the page has focus, because text fields need Space, the arrows, Home and End.
- **Chrome text does not follow the reader's text size** (recommended), but its floor rises to 13 px and the sizes become `theme` tokens. The reader controls are for the *document*; the chrome is a fixed 300 px panel and a one-line toolbar that would break at 32 px. The reader who needs larger chrome can use the OS scale, which iced honours. Confirmed 2026-10-30.
- **Narrow windows:** minimum 480 x 360. The page column is at least 320 px. A pure `panels::toggled(open, panel, width)` (planned as `panels_for`) closes the older panel when both don't fit, so a resize never closes a panel the reader opened.
- **One ADR** (ADR-008: keyboard access, the focus model, the chrome-size decision and the screen-reader limit), written through `invest-adr` in step 1.

## Decisions taken (2026-10-30: 1 fixed, 2 arrow keys, 3 PageUp and PageDown anywhere)

1. **Chrome text: fixed (decided: fixed; ADR-008) or scaling** with the reader's size. Scaling means a panel that grows with the size, a toolbar that wraps, and a new set of golden screenshots.
2. **Arrow keys on choices** (above) versus opening the pick list's menu from the keyboard. iced 0.14 gives no way to open a pick list from code, so a menu route would mean replacing the pick lists with our own widget.
3. **Scroll keys when the page lacks focus.** Above: only PageUp and PageDown. The alternative is to scroll on every bare key unless a text field is focused, which needs an async focus query before each key.

## Steps

Each step is test first, red then green, with the order test → build → clippy → fmt before moving on. Each leaves `main`-ready code.

1. **Verify and decide.** With throwaway tests, confirm how iced delivers Tab, Shift+Tab, Esc, Space and PageDown to a widget and to the app subscription, and whether `text_input` captures Tab. Confirm `operation::focus_next` order against the toolbar. Check how `min_size` is set in `iced::application`. Write ADR-008 with `invest-adr`, and record the answers to the three questions above here.
   **Complexity**: standard
   **RED**: none (investigation); the ADR is the output
   **GREEN**: ADR-008 in `vector/decisions/`, the answers in this plan
   **REFACTOR**: None needed
   **Files**: `vector/decisions/ADR-008-keyboard-access.md`, this plan
   **Commit**: `docs: ADR-008 keyboard access`

2. **Link stops in the layout.** `Page::stops()` returns the links and image-note actions in reading order with their rectangles, and a pure `reach(stops, current, direction) -> Reach` (`Stop(i)` or `Leave`).
   **Complexity**: standard
   **RED**: reading order across paragraphs, lists, tables and quotes; a link that wraps over three lines is one stop with three rectangles; two links to the same address are two stops; an image note is a stop; `reach` walks forward and back, leaves at both ends, and passes a page with none; golden snapshots unchanged
   **GREEN**: `stops`, `Stop`, `reach`
   **REFACTOR**: if `link_at` and `action_at` repeat the same walk, share it
   **Files**: `src/layout/select.rs` (or a new `src/layout/stops.rs`), `src/layout/mod.rs`, tests
   **Commit**: `feat(layout): link stops in reading order`

3. **The page takes focus, and shows it.** `Document` implements `Focusable` and keeps the focused stop in its widget state. It draws the ring over the stop's rectangles. Tab and Shift+Tab move through stops using `reach`, and fall through at the ends. A new document clears the stop; a relayout keeps it.
   **Complexity**: complex (a custom widget gains an iced focus protocol)
   **RED**: widget-state tests (Tab moves, falls through at the end, Shift+Tab from the first leaves backwards, a new page clears focus, the same page at a new width keeps the stop); a theme test that the ring colour is the link colour and meets 3:1 on the page for light, dark and a spread of custom sets
   **GREEN**: the `Focusable` impl, the ring in `draw`, the app routing Tab to `focus_next` and `focus_previous` when the page lets it go
   **REFACTOR**: None needed
   **Files**: `src/ui/document.rs`, `src/ui/app.rs`, `src/ui/keys.rs`, `src/theme/mod.rs`
   **Commit**: `feat(ui): Tab through the links on a page`

4. **Follow and reveal.** Enter on a stop publishes `Follow` or `LoadImage`, as a click does. Moving focus asks the app to scroll the stop into view (nothing if already visible, centred otherwise). Opening a page focuses it.
   **Complexity**: standard
   **RED**: Enter on a link stop publishes `Follow` with its target; Enter on an image note publishes `LoadImage`; a reveal offset for a stop above, below and inside the viewport (pure, in `viewport`); a reducer or app-level test that a new page requests focus
   **GREEN**: the Enter arm, `Message::Reveal`, the focus task after `scroll_after` for a new page
   **REFACTOR**: share the follow path between click and Enter
   **Files**: `src/ui/document.rs`, `src/ui/viewport.rs`, `src/ui/app.rs`, `src/ui/app/scroll.rs`
   **Commit**: `feat(ui): follow a focused link and keep it in view`

5. **Scroll keys.** A pure `scroll_key(key, modifiers) -> Option<Scroll>` (`Page`, `Line`, `Home`, `End`, each up or down) and `Viewport::target(scroll, offset, viewport_height, content_height, line) -> f32`, clamped to the page. PageUp and PageDown are app shortcuts; the rest are handled by the focused `Document`. The line step follows the text size. Correct the comment in `keys.rs`.
   **Complexity**: standard
   **RED**: each key maps to its scroll (and Shift+Space to the reverse); targets are clamped at both ends; a short page that fits the window does not move; the line step grows with the text size; bare Space and arrows do nothing when the page lacks focus
   **GREEN**: the mapping, the target function, the wiring through `scroll_task`
   **REFACTOR**: None needed
   **Files**: `src/ui/keys.rs`, `src/ui/viewport.rs`, `src/ui/document.rs`, `src/ui/app.rs`
   **Commit**: `feat(ui): scroll the page from the keyboard`

6. **Controls take focus.** `ui/focus.rs`: a wrapper that makes any element a `Focusable` stop with the ring, activates on Enter and Space, and (for choices) turns Up and Down into previous and next option. The toolbar buttons, the Settings buttons and choices, the stepper and the colour and home fields are reached in order; disabled buttons are skipped. Esc closes a panel and focuses the page.
   **Complexity**: complex (a new widget, used across the toolbar and panels)
   **RED**: wrapper state tests (focus, unfocus, Enter and Space publish the message, a disabled wrapper is not focusable, Up and Down step a choice and stop at the ends); a pure `step_choice(all, current, direction)` test; Esc maps to closing a panel
   **GREEN**: `focus.rs`, the toolbar and settings using it, `Message::ClosePanel`
   **REFACTOR**: replace the `choice` helper's pick list wiring in `settings.rs` so both routes share one event
   **Files**: `src/ui/focus.rs`, `src/ui/toolbar.rs`, `src/ui/settings.rs`, `src/ui/history.rs`, `src/ui/body.rs`, `src/ui/keys.rs`, `src/ui/app.rs`
   **Commit**: `feat(ui): reach every control from the keyboard`

7. **Chrome text sizes.** The 12 to 20 px literals in `ui/` become tokens in `theme` (a small named scale: note, label, title), with a floor of 13 px.
   **Complexity**: standard
   **RED**: a test that no token is below 13 and the scale is strictly increasing; a grep-style test (as phase 4's) that `ui/` sets no literal size; the golden snapshots unchanged (the page, not the chrome)
   **GREEN**: the tokens and their uses
   **REFACTOR**: None needed
   **Files**: `src/theme/scale.rs`, `src/ui/*.rs`
   **Commit**: `refactor(theme): chrome text sizes as tokens`

8. **Narrow windows.** `min_size` on the window; the window size kept in the app; `panels_for(width, open)` decides which panels stay open when one is toggled.
   **Complexity**: standard
   **RED**: `panels_for` at widths under and over the limit, opening and closing each way, and a resize leaves both open; the window settings carry 480 x 360
   **GREEN**: the function, the size tracking from `window::resize_events`, the settings
   **REFACTOR**: None needed
   **Files**: `src/ui/app.rs`, `src/ui/app/view.rs`, `src/ui/panels.rs` (new, pure)
   **Commit**: `feat(ui): a minimum window size and panels that give way`

9. **Acceptance feature.** `features/keyboard-access.feature` from this plan, with steps in `tests/acceptance/steps/keyboard.rs`. They drive `stops`, `reach`, `scroll_key`, `target`, `step_choice` and `panels_for` through the world's real layout, since the focus rules are pure. Scenarios that need a live window are covered by steps 3 to 6 and the screenshots.
   **Complexity**: standard
   **RED**: the feature and steps first (fail to compile, then pass as the pieces are wired)
   **GREEN**: the steps and fixtures (`plain.md`, `wrapped.md`, `long.md`)
   **REFACTOR**: share the "viewing a page" steps with `reader.rs`
   **Files**: `features/keyboard-access.feature`, `tests/acceptance/steps/keyboard.rs`, `tests/acceptance/steps/mod.rs`, `tests/acceptance/fixtures.rs`
   **Commit**: `test: keyboard access acceptance scenarios`

10. **Screenshots and docs.** `scripts/shoot.sh` gains a way to focus the first link (via `wtype` Tab) and to open a panel in a narrow window. Read the shots (AC 9). Update README (a keyboard table), `assets/welcome.md`, DESIGN.md (focus ring, chrome sizes, the decision), `features/README.md`, `ARCHITECTURE.md` (`ui/focus.rs`, `ui/panels.rs`, `layout` stops), VECTOR.md (Direction: phase 5 built, release distribution next), and the phase 4 plan's last tick if still open.
    **Complexity**: trivial
    **RED**: a shell check that the new flags are rejected when malformed (by hand)
    **GREEN**: the flags, the screenshots, the docs
    **REFACTOR**: None needed
    **Files**: `scripts/shoot.sh`, `README.md`, `DESIGN.md`, `features/README.md`, `ARCHITECTURE.md`, `VECTOR.md`, `assets/welcome.md`
    **Commit**: `docs: phase 5 keyboard access`

11. **Pre-PR.** `/code-review` (with `a11y-review` named in the run), fix findings, an `/impeccable audit` re-run to confirm the score, then `/pr`.
    **Complexity**: standard
    **RED / GREEN / REFACTOR**: per the findings
    **Files**: as needed
    **Commit**: as needed

## Complexity Classification

| Step | Rating | Why |
|---|---|---|
| 1 | standard | Investigation and an ADR |
| 2 | standard | Pure functions within existing patterns |
| 3 | complex | A custom widget takes on iced's focus protocol |
| 4 | standard | Wiring within existing patterns |
| 5 | standard | Pure mapping plus wiring |
| 6 | complex | A new widget used across the toolbar and panels |
| 7 | standard | Constants into the theme |
| 8 | standard | A pure function plus window settings |
| 9 | standard | Acceptance within existing patterns |
| 10 | trivial | Docs and screenshots |
| 11 | standard | Review fixes |

## Pre-PR Quality Gate

- [x] `cargo test`
- [x] `cargo build`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo fmt --check`
- [x] Golden snapshots unchanged
- [x] Screenshots read (AC 9)
- [x] `/code-review` passes, `a11y-review` included
- [x] `/impeccable audit` re-run; no P1 left
- [x] README, DESIGN.md, ARCHITECTURE.md, VECTOR.md and `features/README.md` updated

## Risks

- **Tab may be consumed before the app sees it.** `text_input` or the scrollable could capture it. Step 1 checks, and step 3's routing depends on the answer.
- **The `Focusable` wrapper fights iced's event order.** A wrapped `button` also handles Enter's mouse equivalent and its own `on_press`. Step 6 keeps the wrapper's keys out of the child's event stream and tests both routes.
- **Focus lost on relayout.** Stops are indexed, so a changed document could point at the wrong link. A new document clears focus, and the index is checked against the stop count.
- **Scroll fights the held anchor.** A key scroll has to release the anchor that `Viewport::hold_anchor` keeps after a jump, or the next relayout snaps back. Step 5 tests this.
- **No screen-reader support.** iced 0.14 exposes no accessibility tree. This phase does not fix it; ADR-008 records it so it is a known limit, not a surprise.
- **The macOS job can queue for a while.** As in phase 4.

## Not decided here

Caret browsing and find in page, and whether to revisit screen-reader support when iced gains an accessibility layer.
