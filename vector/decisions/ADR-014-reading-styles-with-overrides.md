# ADR-014: Reading styles with reader overrides

**Date:** 2026-11-26
**Status:** proposed
**Deciders:** gBRETT (product owner)

## Context

Under ADR-007 and its 2026-11-03 amendment, the reader sets the look with four separate controls: typeface, Appearance (Light, Dark, System, seven colour presets, or Custom), text size and spacing. Each control is sound on its own, but nobody designed them as a whole. A reader puts four choices together and hopes they suit each other, and the presets cover only colour. ADR-007 left this for later: "Presets (named sets of size, spacing and colours) are left until the pieces exist." Those pieces now exist.

bonBon, the other Markdown browser in this community, ships five stylesheets (Basic, Book, Classic, Mono, Newsprint). Each one is a full, designed look that the reader picks by name and can change by editing the file. We can't use CSS files themselves, because we never produce HTML (ADR-001). We can still use the model behind them: a small set of complete, named looks, which the reader picks first and adjusts afterwards.

## Decision Drivers

- VECTOR Design Principle 2: the reader owns the presentation. Making a choice easier must not take control away.
- Text size is an accessibility need, not a matter of taste. A reader who needs 24 px needs it in every style, and Ctrl+= / Ctrl+- / Ctrl+0 must keep working as they do now.
- ADR-007's contrast rule: ink, muted and link text meet WCAG AA (4.5:1). We refuse a set that fails rather than warn about it, because the chrome is drawn in the same palette.
- ADR-001 and VECTOR: fonts are bundled, and there's no CSS engine. A style can only choose from the bundled fonts and the values the layout engine already understands.
- ARCHITECTURE.md: visual values live in `theme`, and `theme` is data only. `Look` is the cache key for `Pages`.
- Existing `settings.json` files must load and look the same as before. The Phase 3 golden snapshots must not change at the defaults.

## Options Considered

### Option A: Keep the separate settings (do nothing)

**Pros:**
- No new code, no migration and no new panel layout.
- The most direct control for readers who know what they want.

**Cons:**
- The reader has to design the look; the browser offers no starting point.
- The seven presets set only colour, so "Sepia" doesn't come with a typeface or spacing chosen to go with it.

### Option B: Styles replace the settings (bonBon's model)

A style fixes font, colours, size and spacing. To change a style, the reader picks a different one.

**Pros:**
- The simplest panel, and each look is fully designed.
- Closest to bonBon.

**Cons:**
- Text size would be tied to the style, so a reader who needs larger text would lose it by switching styles. This breaks the accessibility driver.
- Without a text editor and a style file format, which this ADR leaves out, the reader would have no way to adjust a style. That takes control away (Principle 2).

### Option C: Styles plus reader overrides (chosen)

A style is a designed default for every look setting. The reader's own settings sit on top as overrides. Some overrides apply in every style; others belong to one style.

**Pros:**
- Picking one style gives a complete look, and every current control is still there.
- Text size and spacing follow the reader from style to style.
- Adjusting a style's font or colours is a GUI version of bonBon's "Changing one": no file, no CSS, and Reset is easy to find.

**Cons:**
- Two kinds of override is a concept to explain in the panel.
- Settings get bigger: a style id, global overrides, and overrides for each style.

## Decision

**We will ship a small set of reading styles, each a complete designed look, and keep the reader's settings as overrides on top of the chosen style.**

- **A style** is fixed data in `theme`: a name, a bundled typeface, a light and a dark colour set (page, text and link, with the rest derived as in ADR-007), a default text size and a default spacing.
- **Mode stays separate.** Light, Dark or System chooses which of the style's two colour sets is shown. It's no longer an Appearance choice next to the presets.
- **Global overrides: text size and spacing.** Once the reader sets them, they apply in every style. Ctrl+= / Ctrl+- change the global size. Ctrl+0 and Reset clear the global overrides, so the style's defaults apply again.
- **Per-style overrides: typeface and colours.** Changing the font or typing a colour changes only the current style. Each style keeps its overrides when the reader switches away and comes back. "Reset style" clears them, returning the style to how it shipped.
- **Contrast:** every shipped style's light and dark sets pass ADR-007's AA check, with a unit test over every style and mode. Colour overrides are refused under the same rule as Custom today.
- **Migration:** today's defaults (Literata, 18 px, Comfortable, the current light and dark palettes) become the first style. An existing `settings.json` loads as that style. Its font, Appearance colours and Custom colours become that style's overrides, and its size and spacing become global overrides only if they differ from the defaults. Nothing on screen changes for an existing reader. The goldens stay the same.
- **Out of scope:** style files written by the reader (a style of their own, or a bonBon-compatible CSS file of variables or a subset of CSS). That needs its own ADR. Which styles ship, and their names and values, is left to the plan and to DESIGN.md (Impeccable).

## Consequences

**Positive:**
- A new reader picks one name and gets a coherent look. This finishes what ADR-007 set aside as presets.
- Text size works for accessibility in every style.
- It maps closely onto bonBon's stylesheets (pick one, then change it), so the two browsers are easy to explain side by side.
- `Look` stays a small, hashable, resolved value: style plus overrides, resolved to font, colours, size and spacing before layout. Caching and layout don't change.

**Negative:**
- The seven colour-only presets (Sepia, Paper green, Rose, Night, Solarized dark, Nord, High contrast) are replaced. Each becomes a full style with both a light and a dark set, or is dropped. A dark-only preset such as Night or Solarized dark needs a light counterpart designed for it.
- More states to test: styles × modes × overrides. The AA test grows to match.
- A reader who changes the font in one style and then switches won't see that font in the new style. The panel has to make clear which settings belong to the style.
- The Settings panel, `settings.json` schema and `features/reader-controls.feature` / `settings.feature` all change.

**Neutral:**
- Column width is still not a control. A style could carry one later, as a default the reader can't change.
- `scripts/shoot.sh` needs a style option next to its font and appearance options.

## Related Decisions

- ADR-007: Reader controls. Partly superseded: the Appearance choice and presets give way to styles and mode. Its stepped size, three spacings, derived colours and AA refusal still apply.
- ADR-001: Native rendering with our own layout engine. This is why styles are data in `theme` and not CSS.
- ADR-008: Keyboard access. The size shortcuts keep their behaviour.
- ADR-012: Image max height. It follows the resolved text size, so it is unaffected.
