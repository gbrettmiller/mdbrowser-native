# Plan: Source view

**Created**: 2026-11-03
**Branch**: `feat/source-view`, from `main` (after #22)
**Status**: approved (2026-11-03)

## Goal

Let the reader see the Markdown source of the page they are on. The prompt: some web pages open without a byline or other front matter, and there was no way to tell whether the site's `.md` has none or the parser dropped it. The source view settles that in one keystroke, and it serves a project whose premise is that the source *is* the page: the reader can see exactly what the author wrote. The fetched text already reaches the app (`loaded()` in `src/session/reducer.rs`) and is thrown away after parsing; this plan keeps it and draws it through the existing layout engine as a single code block, in the reader's palette and size. No new widget.

Spec artifacts: none from `/specs`. As in earlier phases, the Gherkin below is the spec and lands in `features/source-view.feature`, each scenario added in the step that makes it pass, so every commit is green.

**Priority note.** This is a small debugging aid that also advances "exactly as the writer wrote it". It is not a VECTOR ship criterion, and the leading and tall image gap against the archive criterion is still open. It ships in two increments so the first is small; see below.

**Alternatives considered (for ADR-011).** (a) Hand the file to the system browser or editor, as Ctrl/Cmd+Shift+O does for originals. Rejected: it leaves the app, needs a temp file for local, fallback and welcome pages, and breaks the "one keystroke" answer. (b) A one-shot toggle with no scroll memory and no button. This is Increment 1.

**Facts checked while planning:**
- `loaded()` parses `Loaded.source` and drops it. `Shown` is `{ document, title, fallback_from }`.
- `Pages` caches layout by `Arc<Document>` pointer, so a source view needs its own stable `Arc<Document>`, built once per toggle and held.
- `Block::Code` already draws mono text, wrapped by glyph (`Wrap::Glyph`), on the code background; selection and copy work over code (`copying_rejoins_soft_wraps_but_keeps_hard_breaks`).
- `model::parse` strips front matter before building blocks; `assemble` adds a byline only from `front`. A synthetic document with one `Code` block and an empty `FrontMatter` shows the source as plain text and no byline.
- `Viewport` holds one `scroll_y`; `new_page` resets it. `scroll_after` decides "new page" by `Arc::ptr_eq` on `Shown`.
- Ctrl/Cmd+U is unused in `keys::command`. Esc is `Command::ClosePanel` today.
- Local files and fetches are capped at 16 MiB (`MAX_BYTES`); a file that is not valid UTF-8 is already refused with the "unreadable" error, so it never reaches the source view. `corpus/spec.md` (206 KB) and `corpus/long.md` (139 KB) are the large cases.
- iced 0.14 has no accessibility tree (ADR-008's known limit). The source view adds no screen-reader cue beyond its visible title line and button label, and the ADR says so.
- `session/reducer.rs` (290 lines), `session/mod.rs` (286) and `ui/app.rs` are near the 300-line file guide, so new state goes in a new `session/view.rs` and the Esc rule in `ui/panels.rs`.

## Decisions made in this draft (gBRETT may overturn at approval)

1. **Reload keeps the source view** (rebuilt from the freshly fetched text), because the likeliest reason to reload there is "did the site's `.md` change?". Every other load, Back, Forward, Home, a followed link or a typed address, returns to the rendered page.
2. **The welcome page has a live button.** It has real built-in source, and consistency beats a special case.
3. **A leading BOM is dropped** from the shown source; lone `\r` and `\r\n` become `\n`; an empty file is an empty source view (with its title line), not an error.
4. **Focus** stays on the page body after a toggle, so scroll keys and select-all work at once, the same as opening a page.

## Scope

In: keep the source on the page; a source view (mono, wrapped, selectable, copyable, in the reader's palette, size and spacing); Ctrl/Cmd+U and a toolbar button with a tooltip ("View Markdown source (Ctrl+U)", Cmd on macOS); Esc and navigation leave it; per-view scroll memory; works for local files, web pages, fallback finds and the welcome page.

Out: syntax highlighting, line numbers, find in page, editing, side-by-side, remembering the view across navigation or restarts, the HTML of a page that fell back to Markdown (the source is the Markdown found), horizontal scrolling, screen-reader announcements (iced limit).

## Increments

- **Increment 1 (the first PR, small):** steps 1 to 5b. Toggle by key and button, exact text, fallback source, Reload keeps it, navigation and Esc leave it, copy works, reader controls apply. Answers the original question.
- **Increment 2 (second PR, or the same PR if the timing is fine):** steps 6 and 7. Per-view scroll memory and the large-document budget. If step 7's timing shows the single block is fast enough, Increment 2 is only step 6.

## Acceptance Criteria

1. [ ] **Toggle.** On a loaded page, Ctrl+U (Cmd+U on macOS) or the toolbar button shows the raw Markdown; the same again shows the rendered page. The button reads "Source" on the page and "Page" in the source view, has the tooltip above, and is disabled while Idle, Loading or on an error. In the source view the title line reads "Source of <title>".
2. [ ] **Exact text.** The source view shows the file as fetched, front matter included, nothing parsed away. `\r\n` and lone `\r` show as `\n`; a leading BOM is dropped; an empty file shows an empty source view.
3. [ ] **Fallback finds.** After a fallback, the source view shows the Markdown found, and the address bar shows where it was found.
4. [ ] **Copy.** Select all and copy in the source view gives the source, with soft wraps rejoined, blank lines, leading spaces and tabs kept, and trailing newlines trimmed (as for any code block).
5. [ ] **Leaving.** Esc with no panel open returns to the rendered page; with a panel open, Esc closes the panel first and the source view stays; with neither, Esc only returns focus to the page. Back, Forward, Home, a followed link and a typed address return to the rendered page of whatever loads. Reload stays in the source view and shows the new text, from the top (a reload is a new load, so the offset resets); after a fallback find, Reload fetches the Markdown that was found (the entry settled on it) and stays in the source view of what it returns. A failed Reload shows the error and the Source button is disabled. The source view never appears in History, never enables Back, and never changes the settings file.
6. [ ] **Scroll.** *(Increment 2)* Toggling to the source and back returns the rendered page to where it was; the source keeps its own offset across toggles while the reader stays on the page; a new page always starts at the top of the rendered page.
7. [ ] **Reader controls apply.** The source view is laid out with the reader's text size, spacing and the code font (JetBrains Mono): a larger text step makes the laid-out page taller, and Spacious is taller than Compact, for the same source. In Light, Dark, all seven presets and the Custom fixture `#fdf6e3`/`#333333`/`#0b5fa5`, the code ink meets 4.5:1 on the code background (for any Custom set that passes `check`, `derive` already guarantees this, and its existing test covers a spread).
8. [ ] **Large documents.** *(Increment 2)* In a release build, the first layout of `corpus/spec.md` as a source document takes at most 250 ms (the VECTOR open-time bar), asserted by an `#[ignore]`d timing test in `src/layout/mod.rs` run with `cargo test --release -- --ignored`. Resizing stays smooth through the existing deferral: a layout costing more than `EXPENSIVE_LAYOUT` (32 ms, `src/ui/app.rs`) keeps its previous layout until the width settles, and a `Pages` test shows the source document is deferred like any expensive page. Selection and scroll cost on the single block are checked by hand in the step 8 screenshots session, and the plan accepts that sizes beyond `corpus/spec.md` (up to the 16 MiB cap) are not guaranteed fast. The ignored test is run by hand before the PR, not in CI. If the 250 ms line is missed, the source is chunked (N lines per `Code` block, N from the measurement) and the numbers go in ADR-011.
9. [ ] **Keyboard and focus.** In toolbar order the Source button is the stop directly after Reload (a pure focus-order test). After a toggle, focus is on the page body (the update emits the focus effect for `PAGE_ID`; a test checks it). PageDown, Space, Down, End and Home scroll the source view in the same direction and by the same step as they do a page (a scenario outline over the five keys, asserting direction and the end stops, with a unit test that `Viewport::key_scroll` gives the same delta in both views).
10. [ ] **Layout and nothing else moves.** At the 620 by 360 minimum window the toolbar does not overflow or truncate the new button (checked by screenshot). Golden snapshots, run as part of `cargo test`, do not change. Earlier chords are unchanged (a table test). Colours stay in `theme`, text in `content`; `arch-review` checks this in the PR.
11. [ ] **CI and screenshots.** Test, build, clippy and fmt are green on all four platform jobs. Screenshots through `scripts/shoot.sh` are read and must show: mono text on the code background, the "Source of" title, no byline, in light, in Night, after a fallback find, and at 620 by 360.

## User-Facing Behavior

```gherkin
@source-view
Feature: View the Markdown source
  As a reader
  I want to see the Markdown a page is made of
  So that I can tell what the author wrote and what the browser drew

  Scenario: The source shows the file as written, front matter included
    Given I am viewing "frontmatter/full.md"
    When I view the source
    Then I see the source view
    And the source starts with the line "---"
    And the source is the text of "frontmatter/full.md"
    And the title line says "Source of" the page title

  Scenario: Toggling again returns to the page
    Given I am viewing "index.md"
    And I viewed the source
    When I view the page
    Then I see the rendered page

  Scenario: A page without front matter shows none
    Given I am viewing "typography.md"
    When I view the source
    Then the source is the text of "typography.md"
    And the source has no line "---"

  Scenario: A page found by fallback shows the Markdown that was found
    Given the page "/post" is HTML and "/post.md" is Markdown
    And I am viewing "/post"
    When I view the source
    Then the source is the text of "/post.md"
    And the address is "/post.md"

  Scenario: Windows line endings, old Mac line endings and a BOM show as plain lines
    Given a file "crlf.md" with Windows line endings and a BOM
    And I am viewing "crlf.md"
    When I view the source
    Then the source has no carriage returns and no BOM

  Scenario: An empty file shows an empty source
    Given an empty file "empty.md"
    And I am viewing "empty.md"
    When I view the source
    Then I see the source view
    And the source is empty

  Scenario Outline: Leaving a page leaves the source view
    Given I am viewing "index.md"
    And I viewed the source
    When I <action>
    Then I see the rendered page
    Examples:
      | action                                             |
      | follow the link "Typography and inline styles"     |
      | go Home                                            |
      | enter the absolute path of "typography.md"         |

  Scenario: Back leaves the source view
    Given I am viewing "index.md"
    And I followed the link "Typography and inline styles"
    And I viewed the source
    When I go back
    Then I see the rendered page "index.md"

  Scenario: A failed reload leaves the source view
    Given I am viewing "editable.md"
    And I viewed the source
    When the file "editable.md" is deleted
    And I reload
    Then I see the error "notFound"
    And the Source button is disabled

  Scenario: Reload after a fallback find stays in the source view
    Given the page "/post" is HTML and "/post.md" is Markdown
    And I am viewing "/post"
    And I viewed the source
    When I reload
    Then I see the source view
    And the source is the text of "/post.md"

  Scenario: Reload keeps the source view and shows the new text
    Given I am viewing "editable.md"
    And I viewed the source
    When the file "editable.md" changes to "# Changed"
    And I reload
    Then I see the source view
    And the source is "# Changed"

  Scenario: Reload shows the source from the top
    Given I am viewing "long.md"
    And I viewed the source scrolled to 900
    When I reload
    Then I see the source view
    And the source is scrolled to 0

  Scenario: Esc leaves the source view when no panel is open
    Given I am viewing "index.md"
    And I viewed the source
    When I press Esc
    Then I see the rendered page

  Scenario: Esc closes a panel before it leaves the source view
    Given I am viewing "index.md"
    And I viewed the source
    And I opened the Settings panel
    When I press Esc
    Then the Settings panel is closed
    And I see the source view

  Scenario: Esc on a page does nothing but return focus to the page
    Given I am viewing "index.md"
    When I press Esc
    Then I see the rendered page
    And the page body has focus

  Scenario: The source view is not a History entry and changes no settings
    Given I am viewing "index.md"
    When I view the source
    Then the Back button is disabled
    And the History panel lists only "index.md"
    And the settings file is unchanged

  Scenario Outline: There is no source to view without a page
    Given <state>
    Then the Source button is disabled
    Examples:
      | state                                                         |
      | I entered the absolute path of "missing.md" and see the error "notFound" |
      | a page whose fetch is held and never resolves                 |
      | the app has just started with no page                         |

  Scenario: The welcome page has a source
    Given I am viewing the welcome page
    When I view the source
    Then I see the source view

  Scenario: Copying the source gives the source
    Given I am viewing "tables-and-code.md"
    And I viewed the source
    When I select all and copy
    Then the clipboard is the text of "tables-and-code.md" without trailing newlines

  Scenario Outline: Scroll keys work in the source view
    Given I am viewing "long.md"
    And I viewed the source scrolled to <start>
    When I press <key>
    Then the source is scrolled <result>
    Examples:
      | start | key      | result                     |
      | 0     | PageDown | to more than 0             |
      | 0     | Space    | to more than 0             |
      | 0     | Down     | to more than 0             |
      | 0     | End      | to the end                 |
      | 900   | Home     | to 0                       |

  Scenario: The Source button follows Reload in tab order
    Given I am viewing "index.md"
    Then the toolbar focus order has "Source" immediately after "Reload"

  Scenario: Focus returns to the page after a toggle
    Given I am viewing "index.md"
    When I view the source
    Then the page body has focus

  Scenario: The button names the other view
    Given I am viewing "index.md"
    Then the button reads "Source"
    When I view the source
    Then the button reads "Page"

  Scenario: Reader controls apply to the source view
    Given I am viewing "index.md"
    And I viewed the source
    When I make the text larger
    Then the source page is taller
    When I choose the spacing "Spacious"
    Then the source page is taller again

  # Increment 2
  Scenario: The rendered page keeps its place
    Given I am viewing "long.md" scrolled to 600
    When I view the source
    And I view the page
    Then the page is scrolled to 600

  Scenario: The source keeps its own place
    Given I am viewing "long.md"
    And I viewed the source scrolled to 900
    And I viewed the page
    When I view the source
    Then the source is scrolled to 900

  Scenario: A new page starts at the top
    Given I am viewing "long.md" scrolled to 600
    And I viewed the source scrolled to 900
    When I enter the absolute path of "index.md"
    Then I see the rendered page
    And the page is scrolled to 0
```

## Design decisions (recorded in ADR-011)

- **State, not setting.** `State.view: View`, an enum `Page | Source(Arc<Document>)`, in a new `src/session/view.rs`. The source document is the enum's payload, so it cannot outlive the view or belong to the wrong page. Not saved.
- **Source on `Shown`.** `Shown` gains `source: Arc<str>`. One allocation per page, no re-fetch. The synthetic document is built from it on toggle.
- **A synthetic document.** `model::source_document(&str)` is pure: one `Block::Code` with the normalised text and an empty `FrontMatter`.
- **Single block first, chunked only if measured** (criterion 8).
- **Esc order.** Settings, then History, then the source view, as a pure function in `ui/panels.rs` with a table test.
- **Scroll.** `Viewport` keeps the other view's offset in a second `Cell` and swaps on toggle, applied with the `snap_to` already used for new pages.
- **Reload.** `loaded()` keeps the view when the load is a reload of the location shown, and rebuilds the payload from the new source; any other load resets it to `Page`.

## Steps

Each step adds its own scenarios and step definitions to `features/source-view.feature`, red first, so every commit is green.

### Step 1: The source document

**Complexity**: standard
**RED**: `src/model/mod.rs` tests: `source_document("# Hi\n\ntext")` is one `Block::Code` with that exact text and an empty `FrontMatter`; `\r\n` and lone `\r` become `\n`; a leading BOM is dropped; front matter lines stay in the text; the empty string gives one empty code block; trailing newlines are kept in the block text.
**GREEN**: `pub fn source_document(source: &str) -> Document`.
**REFACTOR**: None needed.
**Files**: `src/model/mod.rs`
**Commit**: `feat(model): build a one-block document from Markdown source`

### Step 2: Keep the source on the loaded page

**Complexity**: standard
**RED**: `src/session/tests.rs`: after a local file loads, `Shown.source` equals the file text exactly; after a fallback find it is the Markdown found, not the HTML; the welcome page has its built-in source. Hand-built `Shown` values in existing tests are updated first.
**GREEN**: `Shown { source: Arc<str>, .. }`; `loaded()` stores it.
**REFACTOR**: None needed.
**Files**: `src/session/mod.rs`, `src/session/reducer.rs`, `src/session/tests.rs`
**Commit**: `feat(session): keep the fetched source on the shown page`

### Step 3: Toggle the view

**Complexity**: standard
**RED**: `src/session/tests.rs`: `Event::ToggleSource` on a ready page switches to `Source` (payload built from `Shown.source`) and back to `Page`; on Idle, Loading or Failed it does nothing; toggling produces no effects (nothing persisted, no History entry, no image loads, settings unchanged). Scenarios: "Toggling again returns to the page", "The source shows the file as written", "A page without front matter shows none", "An empty file shows an empty source", "The welcome page has a source".
**GREEN**: `src/session/view.rs` with `View` and `toggle`; `Event::ToggleSource`; steps `I view the source`, `I view the page`, `I see the source view`, `the source is the text of {string}` and the others the scenarios need.
**REFACTOR**: Extract anything pushing `reducer.rs` or `mod.rs` over 300 lines into `view.rs`.
**Files**: `src/session/view.rs`, `src/session/mod.rs`, `src/session/reducer.rs`, `src/session/tests.rs`, `features/source-view.feature`, `tests/acceptance/steps/source.rs`
**Commit**: `feat(session): toggle between the page and its source`

### Step 4: Leaving, and Reload

**Complexity**: standard
**RED**: `src/session/tests.rs`: a new load, Back, Forward, Home, a followed link and a typed address reset the view to `Page`; Reload keeps `Source` and rebuilds the payload from the new text, and after a fallback find it fetches the address found (the entry settled on it) and keeps `Source`; a failed reload shows the error with the view reset to `Page`. Scenarios: the "Leaving a page leaves the source view" outline, "Back leaves the source view", "Reload keeps the source view and shows the new text", "Reload after a fallback find stays in the source view", "The source view is not a History entry and changes no settings", "A page found by fallback shows the Markdown that was found", "Windows line endings … show as plain lines".
**GREEN**: The reset and keep rules in `loaded()` and `failed()` through `view.rs`.
**REFACTOR**: None needed.
**Files**: `src/session/view.rs`, `src/session/reducer.rs`, `src/session/tests.rs`, `features/source-view.feature`, `tests/acceptance/steps/source.rs`
**Commit**: `feat(session): navigation leaves the source view and reload keeps it`

### Step 5a: Chord, Esc rule, strings and button state

**Complexity**: standard
**RED**: `src/ui/keys.rs`: Ctrl+U (Cmd+U on macOS) maps to `Event::ToggleSource`; bare `u` and Alt+U do not; a table test that every earlier chord is unchanged. `src/ui/panels.rs`: the Esc table (Settings open closes it and keeps the source view; nothing open and the source view on leaves it; neither only returns focus). `content`: `SOURCE`, `PAGE`, `source_of(title)` and the tooltip string. `toolbar::source_button(state) -> (label, enabled)`: "Source" enabled on a ready page, "Page" enabled in the source view, disabled when Idle, Loading or Failed. A pure toolbar focus-order test: Source is the stop directly after Reload. Scenarios: "Esc leaves the source view…", "Esc closes a panel before…", "Esc on a page does nothing…", "There is no source to view without a page", "The button names the other view", "The Source button follows Reload in tab order", "A failed reload leaves the source view" (the session part, the error shown, was unit-tested in step 4; the disabled button is asserted here).
**GREEN**: The chord in `keys::command`; the Esc decision as a pure function in `ui/panels.rs` called from `close_panel`; the toolbar button next to Reload with its tooltip.
**REFACTOR**: Keep `app.rs` growth small by putting the new decisions in `panels.rs` and `toolbar.rs`.
**Files**: `src/ui/keys.rs`, `src/ui/panels.rs`, `src/ui/app.rs`, `src/ui/toolbar.rs`, `src/content.rs`, `features/source-view.feature`, `tests/acceptance/steps/source.rs`
**Commit**: `feat(ui): Ctrl+U and a toolbar button toggle the source view`

### Step 5b: Draw it, copy it, focus the page

**Complexity**: standard
**RED**: Layout (`src/layout/mod.rs`): the source document lays out with the reader's size, spacing and `mono_font` (`theme::reading(look)`); a larger text step and Spacious each make the page taller for the same source; a contrast table over Light, Dark, all seven presets and the Custom fixture checks `colours::contrast(code_ink, code_background) >= AA`; copy-all of the laid-out source page gives the source with trailing newlines trimmed, keeping blank lines, leading spaces and tabs. UI: a toggle emits the focus effect for `PAGE_ID`; the title line reads "Source of <title>". Scenarios: "Copying the source gives the source", "Scroll keys work in the source view", "Focus returns to the page after a toggle", "Reader controls apply to the source view".
**GREEN**: `document_view` draws the `Source` payload; the title line; focus to `PAGE_ID` after a toggle.
**REFACTOR**: None needed.
**Files**: `src/ui/app/view.rs`, `src/ui/app.rs`, `src/ui/body.rs`, `src/layout/mod.rs`, `features/source-view.feature`, `tests/acceptance/steps/source.rs`
**Commit**: `feat(ui): draw the Markdown source in the reader's palette and size`

*Increment 1 ends here: run the gates, `/code-review`, open the PR.*

### Step 6: Each view keeps its place

**Complexity**: standard
**RED**: `src/ui/viewport.rs` tests (pure, `Cell`-based): `swap_view()` stores the current offset and returns the other view's offset; a second swap returns the first; `new_page` clears both; a page that never saw the source returns 0 for it. `scroll_after` on a reload while in `Source` (a new `Shown`) resets to the top and stays in the source view. Scenarios: the three in "Increment 2" of the feature, and "Reload shows the source from the top" (moved here from step 4: the offset lives in the UI viewport, not the session).
**GREEN**: A second `Cell<f32>` in `Viewport`; `scroll_after` calls `swap_view` when only the view changed (same `Shown`, different `View`) and applies it with `snap_to`.
**REFACTOR**: Make "new page vs new view" one small enum if `scroll_after` gets hard to read.
**Files**: `src/ui/viewport.rs`, `src/ui/app/scroll.rs`, `features/source-view.feature`, `tests/acceptance/steps/source.rs`
**Commit**: `feat(ui): the page and its source each keep their scroll position`

### Step 7: Large documents

**Complexity**: complex
**RED**: An `#[ignore]`d timing test in `src/layout/mod.rs` (`cargo test --release -- --ignored`) lays out `corpus/spec.md` as a source document and asserts first layout ≤ 250 ms; a `Pages` test in `src/ui/pages.rs` shows a source document costing more than `EXPENSIVE_LAYOUT` is deferred during a resize like any page; a regular test checks the page height is within a range derived from the line count and that every source line is in the copied text.
**GREEN**: If the single block passes, nothing changes and the numbers go in ADR-011. If not, `source_document` splits into chunks of N lines (N from the measurement), and the tests above are kept green across the seams.
**REFACTOR**: None needed.
**Files**: `src/layout/mod.rs`, `src/ui/pages.rs`, `src/model/mod.rs`, `vector/decisions/ADR-011-source-view.md`
**Commit**: `perf(layout): keep the source view fast on large files`

### Step 8: Doctrine, docs and screenshots

**Complexity**: standard
**RED**: A scripted check that `scripts/shoot.sh` accepts the new `source` key (`bash -n` plus `shoot.sh -k source` reaching the app) and still rejects a bad key; a grep check that VECTOR, README, ARCHITECTURE and the ADR index mention the source view and Ctrl+U.
**GREEN**: ADR-011 through `invest-adr` (decisions above, alternatives considered, the measured numbers, the accessibility limit); a line in `VECTOR.md` § Direction; a sentence each in README and ARCHITECTURE; the `source` key in `shoot.sh` (Ctrl+U). Screenshots on an empty workspace (ask gBRETT first), read against criterion 11.
**REFACTOR**: None needed.
**Files**: `vector/decisions/ADR-011-source-view.md`, `VECTOR.md`, `README.md`, `ARCHITECTURE.md`, `scripts/shoot.sh`
**Commit**: `docs: ADR-011 and the source view in VECTOR, README and ARCHITECTURE`

## Complexity Classification

Steps 1 to 6 and 8 (including 5a and 5b) are `standard`. Step 7 is `complex`: it makes a performance decision that changes the document model if the single block is too slow.

## Pre-PR Quality Gate

- [ ] `cargo test` passes (units, scenarios and goldens), plus `cargo test --release -- --ignored` for the timing test
- [ ] `cargo build` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo fmt --check` passes
- [ ] `/code-review` and `arch-review` pass
- [ ] VECTOR, README, ARCHITECTURE and ADR-011 updated
- [ ] Screenshots read; CI green on all four platform jobs

## Risks & Open Questions

- **One giant code block** may be slow to lay out or hit-test. Mitigation: step 7 measures against a stated number, with chunking as the stated fallback.
- **Esc has three jobs now.** Mitigation: a pure function with a table test of every combination.
- **Ctrl+U in a text field.** Global chords already work while the address bar has focus (Ctrl+H, Ctrl+,); iced's `text_input` does not use Ctrl+U.
- **Tooltip support.** If `focus::button` can't carry a tooltip without a wrapper, step 5 adds the wrapper; if that proves large, fall back to the title line and the ADR as the discoverability and say so.
- **Reload-in-source rule** touches `loaded()`; its tests in step 4 are the guard.
- **Question for gBRETT:** should this wait behind the leading and tall image gap (the open archive ship criterion)? The plan assumes it is small enough to do first.

## Plan Review Summary

Four personas reviewed the plan; the acceptance critic reviewed it three times. Final verdicts: design approve, UX needs-revision (warnings only), strategic needs-revision (warnings only), acceptance no blockers.

**Applied from review**
- Strategic: split into Increment 1 (steps 1 to 5b, the first PR) and Increment 2 (scroll memory and the large-document budget); added alternatives considered and the priority note; the two open product questions are settled in the draft (see Decisions).
- Design: the view state is a `View` enum carrying the source document, in a new `session/view.rs`; the Esc rule is in `ui/panels.rs`; steps name the 300-line extraction.
- UX: Reload keeps the source view; tooltip with the shortcut; the 620 px layout check is a criterion; focus after a toggle; the iced no-accessibility-tree limit is stated.
- Acceptance: scenarios assigned to the step that makes each pass; AC8 has a number (250 ms, ignored release test) and the `EXPENSIVE_LAYOUT` (32 ms) deferral; AC9, AC7 and the reload cases have scenarios and RED tests; step 5 split in two.

**Left as observations**
- UX: the button names the destination, not the current state; the title line carries the state. Optionally show the button as pressed.
- Acceptance: sizes beyond `corpus/spec.md` up to the 16 MiB cap are not guaranteed fast; the ignored timing test runs by hand, not in CI.
- Strategic: the priority against the leading and tall image gap is gBRETT's call (see Risks).
