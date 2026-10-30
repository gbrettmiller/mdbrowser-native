# ADR-007: Reader controls: stepped text size, three spacings, derived custom colours

**Date:** 2026-10-30
**Status:** accepted
**Deciders:** gBRETT (product owner)

## Context

Phase 4 (`vector/plans/phase-4-reader-controls.md`) lets the reader set text size, spacing and colours. Today body size (18 px), line height (1.6), gaps and the colours are constants in `theme`, and the reader chooses only the typeface and light, dark or system. VECTOR Design Principle 2 says font, size, spacing and colour belong to the reader. The open questions are how to offer each control, how far a change reaches into the layout, and what to do about colour sets that can't be read.

## Decision Drivers

- VECTOR Design Principle 2: the reader owns the presentation, and documents bring none.
- 2c's contrast rule: ink, muted and link text meet WCAG AA (4.5:1) on the page and code backgrounds, checked by a unit test.
- The Settings panel and toolbar wear the palette (`ui::chrome`), so an unreadable palette also makes the controls to fix it unreadable.
- ARCHITECTURE.md: all values live in `theme`, all strings in `content`, pure layers do no I/O, and `Pages` caches laid-out pages by a hashable key.
- Phase 3's golden snapshots must not change at the defaults.
- `settings.json` from earlier phases must still load, and unknown values fall back to the defaults.

## Options Considered

### Option A: Free input (a number for size, sliders for spacing, a full palette of eight colours)

**Pros:**
- Maximum control.

**Cons:**
- Eight typed colours give many ways to break contrast, and each needs its own check.
- Free sizes make the cache key and the screenshots unbounded.
- A larger panel to build, label and test.

### Option B: Stepped size, three spacings, three typed colours with the rest derived (chosen)

Size is whole pixels from 12 to 32 in steps of 2 (default 18). Spacing is Compact, Comfortable or Spacious, setting line height, block gap and item gap together. Custom colours are page, text and link as hex, and muted, code, rule and selection are derived.

**Pros:**
- A small, finite set of looks: easy to cache, test and screenshot.
- Contrast is checkable: the derived colours are computed, so one check covers the set.
- Comfortable at 18 is today's layout exactly, so nothing changes for existing readers.

**Cons:**
- Readers can't pick a size between steps, tune spacing separately, or choose their own code colours.

### Option C: Do nothing

Keep the fixed size, spacing and palettes.

**Pros:**
- No new code.

**Cons:**
- Misses the phase 4 direction, and readers who need larger text can only use the OS magnifier.

## Decision

**We will offer stepped text size, three spacings and three typed custom colours with the rest derived, and refuse (not warn about) custom sets that fall below WCAG AA.**

- **Size:** whole pixels, 12 to 32 in steps of 2, default 18. The measure (40 em, 720 px at 18), gutter and indent scale with it, so characters per line stay about the same. A stored value outside the scale is ignored (the default), as with every other unknown setting.
- **Spacing:** line height 1.45 / 1.6 / 1.8, block gap 12 / 16 / 22 px, item gap 2 / 4 / 8 px. Spacing scales only these.
- **Colours:** muted, code background and rule are mixes of the typed page and text colours, code ink is the text colour, and selection is the link colour at 22% on light pages and 30% on dark (as now). A set is refused if text or link on page, or the derived muted text on page or code, is below 4.5:1. A stored custom set that fails on load falls back to light.

We refuse rather than warn because the panel that fixes a bad palette is drawn in it. This is the same rule 2c applies to the built-in palettes, now enforced for the reader's own.

## Consequences

**Positive:**
- The reader owns size, spacing and colour, finishing what Principle 2 promises.
- The cache key stays small and hashable (`Look` with colours as `[u8; 3]`), and the golden snapshots stay as they are at the defaults.
- Contrast is a single, shared function used by both the production check and the tests.

**Negative:**
- No size between the steps, no independent line height or gap, no typed muted, code or rule colours.
- Some pleasant but low-contrast palettes (for example a pastel text on a light page) are refused.
- Two small layout values don't scale with size (the 6 px gap after a heading and the 3 px quote bar in `layout/flow.rs`); this ADR leaves them fixed.
- `Palette` stops being a plain two-variant enum, so the chrome theme for a custom palette is built per change, not once.

**Neutral:**
- Column width stays out, and can follow as another control.
- Presets (named sets of size, spacing and colours) are left until the pieces exist.

## Related Decisions

- ADR-001: Native rendering with our own layout engine (the layout is ours, so size and spacing apply uniformly).
- ADR-006: Shared front matter (the byline and neighbour blocks are ordinary blocks, so they follow the controls).
