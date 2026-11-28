# ADR-016: Toolbar and panel icons come from Font Awesome Free, bundled as its font

**Date:** 2026-11-27
**Status:** proposed
**Deciders:** gBRETT (product owner)

## Context

The toolbar is labelled in words (← → Reload Source Home … ☆ Bookmarks History Settings), and in a narrow window the words crowd the address bar. bonBon's charter says: "icons in the browser chrome. Icons come from Font Awesome." The app already ships its fonts inside the binary and evicts host copies of them (ADR-001), and every text-like thing it draws goes through iced's shared font system. Icons are a new asset, so how they are drawn and licensed needs a record.

## Decision Drivers

- The charter names Font Awesome. It is a reference, not a spec, but matching it costs nothing extra here.
- VECTOR constraint: fonts are bundled, not resolved from the host (spike finding 1, ADR-001).
- ADR-008: every control is reachable and named from the keyboard. Screen readers aren't supported (iced has no accessibility tree), so a control's only name is what's drawn or shown on hover.
- ADR-013 and the CI licence gate: every bundled asset's licence ships in THIRD-PARTY-NOTICES.html and is credited on the About page.
- No new Rust dependency or iced feature unless it earns its place (ARCHITECTURE.md stack).

## Options Considered

### Option A: Keep words (do nothing)

**Pros:**
- No new asset. Every control names itself, for mouse and keyboard alike.

**Cons:**
- A wide toolbar that squeezes the address bar at the 620 px minimum window.
- Departs from the charter.

### Option B: Font Awesome Free 7.3.1 desktop fonts (Solid 900 and Regular 400 OTFs)

Each icon is a character in a bundled font, drawn as text through the existing font system.

**Pros:**
- Matches the charter, and covers every icon we need, including an outline (Regular) and a filled (Solid) star for the bookmark toggle.
- The desktop fonts are SIL OFL 1.1, the same licence as the reading fonts. CC BY 4.0 covers only the SVG and JS packages.
- No new dependency or iced feature: loaded like the other fonts, drawn as `text`.

**Cons:**
- About 500 KB more in the binary (Solid 415 KB, Regular 87 KB).
- Reserved Font Name "Font Awesome": we ship the files unmodified, so no subsetting to the dozen icons we use.
- Its bold, filled style is heavier than our serif-first chrome.

### Option C: Lucide (ISC) or Phosphor (MIT) as an icon font

**Pros:**
- More permissive licences, and a lighter line style that may sit better beside Literata.

**Cons:**
- Not what the charter names.
- Same size and bundling work as B, for a matter of taste.

### Option D: SVG icons

**Pros:**
- Any icon set, crisp at any scale.

**Cons:**
- Needs iced's `svg` feature (resvg and its dependencies) and a second drawing path for chrome.
- SVG is a format ADR-004 keeps out of pages.
- More code and dependencies than a font for the same result.

## Decision

**We will bundle Font Awesome Free 7.3.1's Solid and Regular desktop fonts, unmodified, and draw the chrome's icons as text in them (Option B).** It matches the charter, fits the existing bundled-font path (ADR-001) without a new dependency, and its font licence is the OFL we already ship.

Icons replace the words on Back (arrow-left), Forward (arrow-right), Reload (arrow-rotate-right), Home (house), the bookmark toggle (star: Regular when the page isn't kept, Solid when it is), Bookmarks (bookmark), History (clock-rotate-left) and Settings (gear), and the Bookmarks panel's move up, move down and remove (arrow-up, arrow-down, xmark). **Source / Page stays in words**: it is a mode switch, and its label names the view you'll get. Every icon button has a tooltip naming its action and shortcut, as Source already does.

## Consequences

**Positive:**
- A narrower toolbar, so the address bar keeps its room at the minimum window.
- One place for icons. Later chrome, such as tabs, draws from the same font.

**Negative:**
- **Keyboard readers lose the visible name.** iced shows a tooltip only on hover, so a focused icon button shows the ring but not its name. Mitigation, part of this work: the status line shows the name of the focused icon button, as it shows a hovered link's address.
- About 500 KB more in the binary, and a font upgrade is a manual step (download, check the licence, replace both files).
- Icons carry no meaning for a reader who doesn't know them. The tooltips and the About page's shortcut table are where they are learned.

**Neutral:**
- The family name "Font Awesome 7 Free" joins the bundled families, so a host copy is evicted like the reading fonts.
- The About page's font credits and THIRD-PARTY-NOTICES gain a Font Awesome entry with its LICENSE.txt.
- The ★ ☆ ↑ ↓ × glyphs from Inter, used since bookmarks (a1facca), go.

## Related Decisions

- ADR-001: Native rendering with our own layout engine (bundled fonts, host copies evicted)
- ADR-004: Images (keeps SVG out, relevant to Option D)
- ADR-008: Keyboard access (every control named and reachable; no screen readers)
- ADR-013: About page (credits and THIRD-PARTY-NOTICES)
