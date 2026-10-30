# ADR-008: Keyboard access: a focusable page with link stops, focusable controls, fixed chrome text

**Date:** 2026-10-30
**Status:** accepted
**Deciders:** gBRETT (product owner)

## Context

Phase 5 (`vector/plans/phase-5-keyboard-access.md`) follows the 2026-10-30 Impeccable audit, which found that every real gap in the UI is about operability. In iced 0.14.2 only `text_input` and `text_editor` implement `Focusable`, so Tab cannot reach any button or pick list. `scrollable` handles no keys, so the page cannot be scrolled from the keyboard. The `Document` widget handles `Key::Character` only, so links can only be followed with a click. A reader who cannot use a mouse cannot browse. The audit also flagged chrome text that ignores the reader's text size and a window with no minimum size.

## Decision Drivers

- The app is a browser: following a link is the main task, and it must work from the keyboard (WCAG 2.1.1).
- ARCHITECTURE.md: no browsing logic in `ui`; all values in `theme`, all text in `content`; pure layers are tested without a window.
- ADR-001: the app draws its own page, so links and focus on the page are ours to build. There is no DOM to lean on.
- ADR-007: the Settings panel and toolbar wear the reader's palette, and custom sets already guarantee link on page at 4.5:1.
- Phase 3's golden snapshots must not change: focus is drawn by the widget, not laid out.
- iced 0.14.2, verified in its source:
  - `button` and `pick_list` are not `Focusable`.
  - `text_input` does not handle Tab. The app subscription (`keyboard::listen`, which only sees uncaptured events) does.
  - `text_input` captures Escape and unfocuses, so a second Esc reaches the app.
  - `text_input` captures Space, arrows, Home and End while focused; PageUp and PageDown it does not.
  - `scrollable` handles no keys.
  - `min_size` is set through `window::Settings`.
  - iced exposes no accessibility tree.

## Options Considered

### Option A: Leave it pointer-first

**Pros:**
- No work.

**Cons:**
- A reader without a mouse cannot follow a link. The audit's P1 stays open, and the downloadable binary would ship it.

### Option B: Keep the page unfocusable and drive links from app-level state

The session or app holds a "focused link" index and handles Tab itself, bypassing iced's focus.

**Pros:**
- No custom widget protocol.

**Cons:**
- Two focus systems that disagree: Tab in the address bar and Tab in the page would need hand-built hand-offs.
- Puts focus bookkeeping in the reducer, which has no business knowing about widgets.

### Option C: A focusable page with link stops inside, and a focus wrapper for the chrome (chosen)

`Document` implements `Focusable` and keeps the focused stop in its widget state. `ui/focus.rs` wraps buttons and choices so they become stops too.

**Pros:**
- One focus order, iced's own: the address bar, the toolbar, the page, the panels.
- Stops are a pure function of the page, so the rules are tested without a window.
- A click and Enter share one path (`Event::Follow` / `LoadImage`).

**Cons:**
- A custom widget takes on iced's focus protocol, and a new wrapper widget is used across the toolbar and panels.
- The wrapper must keep its keys out of the child's event stream.

## Decision

**We will make the page and every control a keyboard stop, using option C.**

- **Page.** `Document` is `Focusable`. Tab and Shift+Tab move through the page's stops (links and image-note actions, in reading order) and fall through to `focus_next` and `focus_previous` at the ends. `Page::stops()` is a pure function; a stop is identified by its index, so a relayout keeps focus on the same link of the same document and a new document clears it. Enter publishes the same event a click does. Moving focus scrolls the stop into view.
- **Ring.** The focus ring is the link colour, 2 px, outside the shape, drawn over every line a wrapped link covers. It needs no new colour token and meets 3:1 on the page for light, dark and every custom set that passes `check`.
- **Controls.** `ui/focus.rs` wraps any element as a stop with the same ring. Enter and Space activate it. For the four choices in Settings (font, images, appearance, spacing), Up and Down step to the previous or next option, stopping at the ends. The pick lists stay for the mouse, because iced offers no way to open one from code.
- **Scroll keys.** PageUp, PageDown, Space and Shift+Space move by the viewport minus a three-line overlap; arrows move one line; Home and End jump to the ends. PageUp and PageDown work wherever focus is. Space, arrows, Home and End act only when the page has focus, because text fields need them. Opening a page focuses it, as browsers do.
- **Chrome text stays fixed.** It does not follow the reader's text size: the 300 px panel and the one-line toolbar would break at 32 px, and the reader's controls are for the document. Its sizes become `theme` tokens with a floor of 13 px (the audit found 12 px labels). A reader who needs larger chrome has the OS scale.
- **Narrow windows.** The minimum window size is 480 x 360. The page column is at least 320 px. When both panels are open and the page would fall under that, opening one closes the other. A resize never closes a panel.
- **Known limit: screen readers.** iced 0.14 exposes no accessibility tree, so a screen reader cannot read the page or the controls. This ADR does not fix it. Revisit when iced gains an accessibility layer.

## Consequences

**Positive:**
- Every task (browse, scroll, change a setting) can be done without a mouse.
- Link following has one path whether clicked or entered.
- The focus rules are pure and covered by acceptance scenarios.

**Negative:**
- A second custom widget (`focus.rs`) to maintain across iced upgrades; its event ordering with `button` has to be rechecked on each.
- Choices change on Up and Down with no pop-up menu from the keyboard, which differs from the mouse route.
- Chrome text will not grow for a reader who raises the text size. Readers who need larger chrome have only the OS scale.
- Screen-reader users still cannot use the app.

**Neutral:**
- Space, arrows, Home and End scroll only while the page has focus. After typing in the address bar, Tab or Esc moves back to the page.
- Esc in a text field first unfocuses the field (iced captures it), then a second Esc closes a panel.

## Related Decisions

- ADR-001: Native rendering with an own layout engine (the page is ours to focus).
- ADR-007: Reader controls (the palette guarantees the ring's contrast; the Settings panel gains keyboard controls; Ctrl+= / Ctrl+- / Ctrl+0 are unchanged).
