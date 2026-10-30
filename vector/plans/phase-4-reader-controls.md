# Plan: Phase 4: reader controls

**Created**: 2026-10-30
**Branch**: `feat/phase-4`, from `main` after phase 3 (#12)
**Status**: approved (2026-10-30), building

## Goal

Give the reader the rest of the presentation. After 2c the reader owns the typeface and light or dark, but body size, line and paragraph spacing, and colours are fixed (`theme::LIGHT`, `theme::DARK`). Phase 4 adds **text size**, **spacing** and **custom colours** to the Settings panel, so the reader owns font, size, spacing and colour, as VECTOR principle 2 says. Documents still carry no styling.

Spec artifacts: none from `/specs`. As in earlier phases, the Gherkin below is the spec, and it lands in `features/reader-controls.feature` in step 8.

**Decisions already made:**
- The panel is the Settings side panel from 2c. Controls apply at once, are saved, and keep the reading position, like appearance.
- Captions for images (phase 2b's note) and showing a link's destination before clicking are not text size, spacing or colour. They stay out.

## Scope

In:
1. **Text size**: a bounded scale around today's 18 px. Headings, code and image notes scale with it, because they are multiples of the body size. Shortcuts: Ctrl+= larger, Ctrl+- smaller, Ctrl+0 back to the default (Cmd on macOS).
2. **Spacing**: Compact, Comfortable (default, today's values) and Spacious. Each sets line height and the gaps between blocks and list items together.
3. **Custom colours**: a fourth appearance, **Custom**, with the reader's own page background, text and link colours as hex. The other tokens (muted, code ink and background, rule, selection) are derived from them, so the reader sets three values, not eight. A set that can't be read comfortably (text on page below WCAG AA, 4.5:1) is refused with a message and the old colours stay.
4. **Reset**: one button puts size and spacing back to the defaults. Custom colours are kept until the reader chooses another appearance.
5. **Persistence and old files**: `settings.json` from phase 3 still loads. New keys are optional, and unknown or out-of-range values fall back to the defaults.

Out: column width (the measure), per-site settings, saved themes or named presets, importing a colour scheme, a font beyond the four bundled, justification and hyphenation, any change to how documents are parsed, installers.

## Acceptance Criteria

1. [ ] **Text size.** Settings has a Text size control and Ctrl+=, Ctrl+- and Ctrl+0 work (Cmd on macOS). The scale has a smallest and largest step, and the control stops there. The default is today's 18 px.
   - Body, headings, code, image notes and the byline and front matter blocks all follow the size.
   - The text column's measure scales with the size, so the characters per line stay about the same instead of the column running out of room.
2. [ ] **Spacing.** Compact, Comfortable and Spacious change line height, block gap and list item gap together. Comfortable gives exactly the layout of phase 3, so the golden snapshots do not change.
3. [ ] **Custom colours.**
   - Choosing Custom with no saved colours starts from the current palette's page, text and link colours, so nothing jumps.
   - Page, Text and Link accept `#rgb` and `#rrggbb`, and the field says what it accepts. An invalid value is refused with a message, and the old colour stays. The field keeps what was typed, so the reader can correct it.
   - Text on page and link on page meet WCAG AA (4.5:1), and so does the derived muted text on page and on code. A unit test checks the derivation across a spread of inputs, and a custom set where text or link on page falls below 4.5:1 is refused with a message saying what failed.
   - Selection, rule and code colours are derived, never typed. No colour is hardcoded outside `theme`.
   - The window chrome (toolbar, panels, pick lists, scrollbar) wears the custom palette, as 2c's does for light and dark.
4. [ ] **The reader keeps their place.** Changing size, spacing or colours relays out, repaints at once, and keeps the reading position (the held anchor, as 2c's palette change does). Selection is cleared the way a width change clears it today.
5. [ ] **Persistence.** Size, spacing, appearance and custom colours survive a restart. Phase 3 files, unknown values (`"spacing":"tight"`, `"text_size":999`, `"custom":{"ink":"red"}`) and missing keys give the defaults.
6. [ ] **Reset.** The button restores size and spacing to the defaults and leaves font, appearance and colours alone.
7. [ ] **System and the others are unchanged.** System, Light and Dark behave as in 2c, and the System fallback to light when the OS reports nothing still holds.
8. [ ] **Strings and values.** All text is in `content`, all colours and sizes in `theme`.
9. [ ] **CI.** Test, build, clippy and fmt are green on all four platform jobs. Screenshots through `scripts/shoot.sh` for the smallest, default and largest sizes, three spacings and a custom palette are read and checked on every font.

## User-Facing Behavior

```gherkin
@reader-controls
Feature: Reader controls
  As a reader
  I want to set the size, spacing and colours of the text
  So that documents are comfortable for my eyes

  # Size and spacing names are as the Settings panel shows them.

  Scenario: Documents open at the default size and spacing
    Given I am viewing "index.md"
    Then the text size is "18"
    And the spacing is "Comfortable"

  Scenario: Larger text re-sets the open document
    Given I am viewing "index.md"
    When I make the text larger
    Then the document is set at a larger size
    And I am still viewing "index.md"

  Scenario: Text size stops at its limits
    Given I am viewing "index.md"
    When I make the text larger 40 times
    And I make the text larger
    Then the text size is the largest step
    When I make the text smaller 80 times
    Then the text size is the smallest step

  Scenario: Resetting the size returns to the default
    Given I made the text larger
    When I reset the text size
    Then the text size is "18"

  Scenario Outline: Choosing a spacing re-sets the open document
    Given I am viewing "index.md"
    When I choose the spacing "<spacing>"
    Then the spacing is "<spacing>"
    And I am still viewing "index.md"
    Examples:
      | spacing     |
      | Compact     |
      | Comfortable |
      | Spacious    |

  Scenario: Compact is shorter than Comfortable and Spacious is taller
    Given I am viewing "index.md"
    When I choose the spacing "Compact"
    Then the page is shorter than at "Comfortable"
    When I choose the spacing "Spacious"
    Then the page is taller than at "Comfortable"

  Scenario: Reset restores size and spacing only
    Given I chose the reading font "Inter"
    And I made the text larger
    And I chose the spacing "Spacious"
    When I reset the reader controls
    Then the text size is "18"
    And the spacing is "Comfortable"
    And the reading font is "Inter"

  Scenario: Custom colours start from the current palette
    Given I chose the appearance "Dark"
    When I choose the appearance "Custom"
    Then the custom page colour is the dark page colour

  Scenario Outline: A custom colour is accepted as hex
    Given I chose the appearance "Custom"
    When I set the custom "<which>" colour to "<value>"
    Then the custom "<which>" colour is "<value>"
    Examples:
      | which | value   |
      | Page  | #fdf6e3 |
      | Text  | #333    |
      | Link  | #0b5fa5 |

  Scenario Outline: A value that isn't a colour is refused
    Given I chose the appearance "Custom"
    When I set the custom "Text" colour to "<value>"
    Then I am told "That isn't a colour. Use a hex value such as #222 or #fdf6e3."
    And the custom "Text" colour is what it was
    Examples:
      | value |
      | red   |
      | #12   |
      | #ggg  |
      |       |

  Scenario: Text that can't be read on the page is refused
    Given I chose the appearance "Custom"
    And I set the custom "Page" colour to "#ffffff"
    When I set the custom "Text" colour to "#eeeeee"
    Then I am told that the text colour is too close to the page colour
    And the custom "Text" colour is what it was

  Scenario: Links that can't be read on the page are refused
    Given I chose the appearance "Custom"
    And I set the custom "Page" colour to "#ffffff"
    When I set the custom "Link" colour to "#ffff00"
    Then I am told that the link colour is too close to the page colour
    And the custom "Link" colour is what it was

  Scenario: Custom colours survive a restart
    Given I chose the appearance "Custom"
    And I set the custom "Page" colour to "#fdf6e3"
    When I restart the app
    Then the appearance is "Custom"
    And the custom "Page" colour is "#fdf6e3"

  Scenario: Choosing another appearance keeps the custom colours
    Given I chose the appearance "Custom"
    And I set the custom "Page" colour to "#fdf6e3"
    When I choose the appearance "Light"
    And I choose the appearance "Custom"
    Then the custom "Page" colour is "#fdf6e3"

  Scenario: Settings saved before reader controls existed still load
    Given the stored settings are "{\"font\":\"inter\",\"appearance\":\"dark\"}"
    When I restart the app
    Then the text size is "18"
    And the spacing is "Comfortable"
    And the appearance is "Dark"

  Scenario: A bad stored value doesn't discard its good neighbours
    Given the stored settings are "{\"font\":\"inter\",\"spacing\":\"tight\",\"appearance\":\"dark\"}"
    When I restart the app
    Then the reading font is "Inter"
    And the appearance is "Dark"
    And the spacing is "Comfortable"

  Scenario Outline: Unknown stored values fall back to the defaults
    Given the stored settings are "<stored>"
    When I restart the app
    Then the text size is "18"
    And the spacing is "Comfortable"
    And the custom "Page" colour is "#fbfaf7"
    Examples:
      | stored                                                             |
      | {\\"spacing\\":\\"tight\\"}                                        |
      | {\\"text_size\\":999}                                              |
      | {\\"text_size\\":\\"big\\"}                                        |

  Scenario: A stored custom set that can't be used falls back to the light colours
    Given the stored settings are "{\"appearance\":\"custom\",\"custom\":{\"ink\":\"red\"}}"
    When I restart the app
    Then the appearance is "Custom"
    And the custom "Page" colour is "#fbfaf7"
```

## Decisions (proposed, 2026-10-30)

- **Text size is a scale of steps, not free input.** Whole pixels from 12 to 32 in steps of 2, default 18. A stored out-of-range value is not clamped but ignored (the default), like every other unknown setting. The Settings control is a stepper (smaller, the size, larger) and the shortcuts use the same steps.
- **The measure scales with text size** (40 em, which is 720 px at 18), and so do the gutter and the indent. Spacing choices scale only line height and the gaps.
- **Spacing is a three-way choice**, not independent sliders. Line height 1.45 / 1.6 / 1.8, block gap 12 / 16 / 22 px, item gap 2 / 4 / 8 px. Comfortable is today's values exactly.
- **Custom colours derive the rest.** Muted is the text colour mixed toward the page, code background is the page nudged toward the text, code ink is the text, rule is a lighter mix, selection is the link at 22% on light pages and 30% on dark (as now). The mix amounts are tuned in step 6 so that every derived pair meets AA whenever the typed pairs do. If no mix gives AA muted text, muted falls back to the text colour, so a set that passes `check` always yields an all-AA palette.
- **Refuse, don't warn, below AA.** An unreadable custom palette is the one thing the reader can't see their way out of (the Settings panel itself wears the palette). The reader keeps ownership of everything else.
- **One ADR** (ADR-007: reader controls, the scale, the derivation and the AA refusal), written through `invest-adr` in step 1.

## Needs your call before building

1. **Refuse vs warn** below AA for custom colours (above: refuse).
2. **Measure scales with size** (above: yes), or keep 720 px and let the text wrap tighter?
3. Whether **column width** belongs in this phase. I left it out because VECTOR's wording is "text size, spacing and custom colours".

## Step 1 findings (2026-10-30)

- **Decisions:** gBRETT approved the plan with the proposed defaults: refuse below AA, the measure scales with size, column width stays out. ADR-007 records them.
- **Hardcoded sizes:** only two in `layout/`: the 6 px gap after a heading (`flow.rs:85`) and the 3 px quote bar (`flow.rs:106`). Both stay fixed (ADR-007). Nothing in `ui/` sets a text size apart from the panel's own labels.
- **Shortcuts:** iced hands `keys::command` the modified key (`iced_winit` `conversion.rs:275`), so Ctrl+Shift+= arrives as `+` and Ctrl+= as `=`. Step 5 matches `=`, `+` and `-` with the primary modifier, and the stepper is always there for layouts that differ. Still to check by hand on this machine in step 5.

## Steps

Each step is test first, red then green, with the order test → build → clippy → fmt before moving on. Each leaves `main`-ready code. Complexity sets review depth in `/build`.

1. **Verify and decide.** Check with a throwaway test that iced reports `=`, `+` and `-` with Ctrl on this keyboard layout, and that `Theme::column_width` and the front matter blocks use only `body_size` and the theme (grep for hardcoded sizes in `layout/` and `ui/`). Write ADR-007 with `invest-adr`.
   **Complexity**: standard
   **RED**: none (investigation); the ADR is the output
   **GREEN**: ADR-007 in `vector/decisions/`, and the answers to the three questions above recorded in this plan
   **REFACTOR**: None needed
   **Files**: `vector/decisions/ADR-007-reader-controls.md`, this plan
   **Commit**: `docs: ADR-007 reader controls`

2. **Settings model: size and spacing.** `TextSize` (a step, with `from_px`, `larger`, `smaller`, `default`) and `Spacing` (`Compact`, `Comfortable`, `Spacious`, with `key` and `from_key`) in `theme`. `Settings` gains `text_size` and `spacing`. Events `ChooseTextSize`, `StepTextSize(Larger | Smaller | Reset)`, `ChooseSpacing`, `ResetReaderControls`, each persisting like `ChooseAppearance`.
   **Complexity**: standard
   **RED**: theme tests (stepping stops at both ends, `from_px` rejects out-of-range, keys round-trip and `"tight"` is `None`); reducer tests (each event changes settings and emits `PersistSettings`; reset leaves font, images, appearance and home alone); store tests (round trip, a phase 3 file, unknown and out-of-range values give the defaults)
   **GREEN**: the types, the fields, the events, the store keys `text_size` and `spacing`
   **REFACTOR**: if `with_settings`' arms repeat, share them
   **Files**: `src/theme/mod.rs` (split to `src/theme/scale.rs` if it passes 300 lines), `src/session/mod.rs`, `src/session/reducer.rs`, `src/session/tests.rs`, `src/services/store.rs`
   **Commit**: `feat(session): settings for text size and spacing`

3. **Theme applies size and spacing.** A `Look { font, palette, size, spacing }` value replaces the `(ReadingFont, Palette)` tuple, and `theme::reading(look)` builds the `Theme`: body size from the step, measure, gutter and indent in proportion, line ratio and gaps from the spacing.
   **Complexity**: complex (the signature of `reading` is used across `layout`, `ui` and tests)
   **RED**: theme tests (size scales body size, headings, code size, and measure in proportion; Comfortable at 18 equals today's `LIGHT` and `DARK` exactly; Compact is tighter and Spacious looser on all three of line ratio, block gap and item gap); a layout test that Spacious at the same width gives a taller page than Comfortable; the golden snapshots must pass unchanged at the defaults
   **GREEN**: `Look`, `reading(look)`, the scaling functions
   **REFACTOR**: move the repeated `reading(ReadingFont::X, Palette::Y)` calls in tests to a `Look::default()` helper
   **Files**: `src/theme/mod.rs`, `src/layout/*.rs` tests, `src/ui/pages.rs`, `src/ui/viewport.rs`, `src/ui/app/scroll.rs`
   **Commit**: `refactor(theme): build the reader theme from a Look`

4. **Plumb size and spacing through the UI.** The page cache key becomes the `Look`. The app builds the `Look` from settings and the resolved palette. The scroll code asks for the gutter from the same `Look`.
   **Complexity**: standard
   **RED**: `Pages` tests (a different size or spacing misses the cache and the same one hits); a viewport test that the held anchor survives a size change (reuse the 2c palette-change test shape)
   **GREEN**: the key and the plumbing in `ui/app`
   **REFACTOR**: None needed
   **Files**: `src/ui/pages.rs`, `src/ui/app.rs`, `src/ui/app/scroll.rs`, `src/ui/app/view.rs`
   **Commit**: `feat(ui): set pages at the chosen size and spacing`

5. **Size and spacing controls.** A stepper and a Spacing pick list in the Settings panel (in `content`: labels, "Reset"), the three shortcuts in `keys.rs`, and the Reset button.
   **Complexity**: standard
   **RED**: `keys` tests (primary plus `=`, `+`, `-` and `0` map to the commands; bare keys do nothing); a `content` test that every spacing and size label is non-empty; a settings-panel test if the panel has any (it has none today, so cover it through the app message path)
   **GREEN**: the controls and chords
   **REFACTOR**: extract `settings::stepper` if the panel's `choice` helper can't carry it
   **Files**: `src/ui/settings.rs`, `src/ui/keys.rs`, `src/ui/app.rs`, `src/content.rs`
   **Commit**: `feat(ui): text size and spacing controls`

6. **Custom colours: the model.** `Colours { page, text, link }` stored as `[u8; 3]` (so it is `Hash` and `Eq`), `parse_hex` (`#rgb`, `#rrggbb`, case-insensitive), `derive(colours) → Theme` colour tokens, and a `check(colours) → Result<Colours, Unreadable>` against AA. `Appearance` gains `Custom`.
   **Complexity**: complex (new pure logic with a design constraint)
   **RED**: parsing (accepts both forms, rejects `red`, `#12`, `#ggg`, the empty string and a trailing space); derivation (for a spread of inputs including near-black on near-white, sepia and dark pairs, every derived pair meets 4.5:1 when the typed pairs do, and `check` refuses when text or link on page is below 4.5:1); `Appearance::Custom` key round trip and the System resolution still resolving to light or dark only
   **GREEN**: the parser, the derivation with its tuned mix amounts, the check
   **REFACTOR**: share the `contrast` helper that is a test function today, so the production check and the tests use one implementation
   **Files**: `src/theme/colours.rs`, `src/theme/mod.rs`
   **Commit**: `feat(theme): custom colours with derived tokens`

7. **Custom colours: session, store and palette plumbing.** `Settings.custom: Colours` (defaulting to the light palette's page, text and link), events `ChooseCustomColour(Which, String)` that parse, check and either persist or return a `Notice` with the message, and `Palette` becomes `Light | Dark | Custom(Colours)`. The chrome theme for a custom palette is built per change (the `OnceLock` stays for light and dark).
   **Complexity**: complex (touches the settings, the cache key, the chrome and the reducer's notices)
   **RED**: reducer tests (a valid colour persists, an invalid one leaves settings unchanged and returns the message, an unreadable one names what failed); a store round trip and the three bad-value cases; `Appearance::Custom.palette(_)` yields `Palette::Custom(settings.custom)`; the chrome test extended to a custom palette; a `Pages` key test (a changed colour misses the cache); a first-use test (choosing Custom the first time copies the current palette's colours)
   **GREEN**: the event, the notice, the palette variant, the chrome builder
   **REFACTOR**: None needed
   **Files**: `src/theme/mod.rs`, `src/session/mod.rs`, `src/session/reducer.rs`, `src/session/tests.rs`, `src/services/store.rs`, `src/ui/chrome.rs`, `src/ui/pages.rs`, `src/ui/app.rs`, `src/content.rs`
   **Commit**: `feat(session): custom colour settings`

8. **Custom colours: the panel, and the acceptance feature.** Three hex fields (Page, Text, Link) that show only when the appearance is Custom, each with a submit action (as the home page field has) and the notice. `features/reader-controls.feature` from this plan, with its steps in `tests/acceptance/steps/`.
   **Complexity**: standard
   **RED**: the feature file and its steps first (they fail to compile or run, then pass as the controls land); a draft-state test for the fields, as the home draft has
   **GREEN**: the fields, the messages and the steps
   **REFACTOR**: if the three fields share their code with the home page field, extract one `field` helper
   **Files**: `features/reader-controls.feature`, `tests/acceptance/steps/reader.rs`, `src/ui/settings.rs`, `src/ui/app.rs`, `src/content.rs`
   **Commit**: `feat(ui): custom colour fields and acceptance scenarios`

9. **Screenshots and docs.** `scripts/shoot.sh` for sizes 12, 18 and 32, the three spacings, and a custom palette (add a way to set text size, spacing and custom colours for the scratch store, as `-a` sets the appearance) across every font. Read the screenshots. Update README (status line), DESIGN.md (tokens, with the new scales and the derivation), `features/README.md`, `ARCHITECTURE.md` (theme and session rows), VECTOR.md (Direction: phase 4 built, phase 5 next), and the welcome page if it mentions what comes next. Use `/impeccable` to check the spacing values and the custom palette derivation.
   **Complexity**: trivial
   **RED**: a `scripts` smoke check that the new flags are rejected when malformed (shell, by hand)
   **GREEN**: the script flags, the screenshots, the docs
   **REFACTOR**: None needed
   **Files**: `scripts/shoot.sh`, `README.md`, `DESIGN.md`, `features/README.md`, `ARCHITECTURE.md`, `VECTOR.md`, `assets/welcome.md`
   **Commit**: `docs: phase 4 reader controls`

10. **Pre-PR.** `/code-review`, fix findings, then `/pr`. The macOS job can queue for a while.
    **Complexity**: standard
    **RED / GREEN / REFACTOR**: per the findings
    **Files**: as needed
    **Commit**: as needed

## Complexity Classification

| Step | Rating | Why |
|---|---|---|
| 1 | standard | Investigation and an ADR |
| 2 | standard | New types and events within existing patterns |
| 3 | complex | Changes a signature used across layers |
| 4 | standard | Plumbing within existing patterns |
| 5 | standard | UI and shortcuts within existing patterns |
| 6 | complex | New pure logic with an accessibility constraint |
| 7 | complex | Touches settings, cache key, chrome and notices |
| 8 | standard | UI and acceptance within existing patterns |
| 9 | trivial | Docs and screenshots |
| 10 | standard | Review fixes |

## Pre-PR Quality Gate

- [x] `cargo test`
- [x] `cargo build`
- [x] `cargo clippy --all-targets -- -D warnings`
- [x] `cargo fmt --check`
- [x] Golden snapshots unchanged at the default size, spacing and palettes
- [x] Screenshots read (AC 9)
- [x] `/code-review` passes
- [x] README, DESIGN.md, ARCHITECTURE.md, VECTOR.md and `features/README.md` updated

## Risks

- **`reading()` is used everywhere.** Step 3 is the widest change. It is a pure refactor with the goldens as the guard, and it lands before any new behaviour.
- **Layout cost at large sizes.** Every change relays out the open document. Check on the largest corpus page at 32 px that a step doesn't stutter (the page cache and image generation make repeated relayouts cheap, but not free).
- **The reader locks themselves out.** A palette with unreadable text would make the Settings panel unreadable too. The AA refusal in step 7 and the check in step 6 are the guard, and a stored palette that fails the check on load falls back to light.
- **Derived tokens for odd inputs.** A saturated page colour or a mid-grey page can make AA muted text impossible. The derivation falls back to the text colour for muted (and the page for the code background) rather than choosing an unreadable colour, so only `check` ever refuses a set.
- **Shortcut clashes.** Ctrl+= and Ctrl+- are the browser convention, but some layouts need Shift for `+`. Step 1 checks, and the stepper is always there.
- **Welcome page.** It is a normal document, so it follows the new controls with no special case.

## Not decided here

Whether column width and image size should join these controls later, and whether presets (a named set of size, spacing and colours) are worth having once the pieces exist.
