# Plan: Phase 2c: settings and home

**Created**: 2026-10-29
**Branch**: `feat/phase-2c`, from `main` after phase 2b (#9)
**Status**: built (2026-10-29)

## Goal

Give the reader one place for their choices, a page to start from, and a dark palette. Today the font and Web images menus sit in the toolbar, the empty window shows a one-line welcome, and there is only a light palette. 2c moves the choices into a Settings panel, adds a home page, and adds **System / Light / Dark** appearance.

**Decisions already made (2026-10-29):**
- The palette setting is in 2c. Text size, spacing and custom colours stay in the later "Reader controls" phase.
- Markdown fallbacks (`.md` sibling, `llms.txt`) are phase 2d, not here.

## Scope

In:
1. **Settings panel**: font, Web images, palette, home page. Replaces the toolbar's two menus. Opens from a toolbar button and a shortcut, like History.
2. **Appearance**: System (default), Light, Dark. A dark palette in `theme`. System follows the OS through `iced::system::theme()` and `theme_changes()`, and falls back to light where the OS reports nothing.
3. **Home page**: a setting holding an address (local path or `http(s)` URL). Empty means the built-in welcome page. A Home button and a shortcut open it. It also opens on launch when no address is given on the command line.
4. **Built-in welcome page**: a real Markdown document bundled in the binary, rendered by our own engine. It replaces the `content::WELCOME` line, so first-run readers see something worth reading, and it has working links (a few starting points, how to set a home page).

Out: text size and spacing, custom palettes, per-site settings, any `llms.txt` or fallback behaviour, installers.

## Acceptance Criteria

1. [x] **Settings panel.** A toolbar **Settings** button toggles a side panel (as History does) with Font, Web images, Appearance and Home page. Choices apply at once and are saved. The toolbar no longer carries the font and image menus, so the narrow-window second row goes away.
2. [x] **Existing settings keep working.** `settings.json` files written by 2b still load unchanged. Unknown or missing `appearance` and `home` values fall back to the defaults.
3. [x] **Appearance.**
   - Options are System (default), Light and Dark, saved as `appearance`.
   - Light and Dark change every colour token in `Theme`: background, ink, muted, link, code ink, code background, rule and selection. No colour is hardcoded outside `theme`.
   - System follows the OS and updates when the OS changes while the app runs. If the OS reports no preference, it is light.
   - Contrast: ink, muted and link text on the page background, and on the code background, meet WCAG AA (4.5:1) in both palettes. A unit test computes this from the tokens.
   - The window chrome (toolbar, panels, pick lists, scrollbar) follows the palette, not only the page.
4. [x] **Appearance changes keep the reading position.** Switching palette relays out (layout bakes the colours into the page), repaints at once, and keeps the scroll position. Images and their placeholders and notes are legible in both palettes.
5. [x] **Home page.**
   - The Home page field accepts what the address bar accepts. An invalid value is refused with a message, and the old value stays.
   - **Home** (toolbar button, and Alt+Home) opens it as a normal navigation, so Back returns to where the reader was.
   - With no home set, Home opens the built-in welcome page.
   - On launch with no command-line address, the home page opens. A command-line address wins.
   - A home page that fails to load shows the usual error, with the toolbar and Settings still usable.
6. [x] **Welcome page.** Bundled Markdown, parsed and laid out like any document. Its title is "Welcome". Its address is shown as a built-in address (not a file or URL), it is not recorded in History (it has no address to return to), and relative links inside it are either absent or work. It can't be mistaken for a local file by Open in browser, and Reload reloads it from the binary.
7. [x] **Persistence.** Appearance and home survive a restart.
8. [x] **Strings and values.** All text is in `content`, all colours in `theme`.
9. [x] **CI.** Test, build, clippy and fmt are green on all four platform jobs. Screenshots through `scripts/shoot.sh` for both palettes are read and checked on every font.

## Decisions (2026-10-29)

- **Built-in address**: the address bar is blank for the welcome page. No new scheme, so no ADR.
- **Settings**: a side panel, like History.

## Still to verify

Checked by gBRETT after the build (2026-10-29): System following the OS and the dark values (AC 3), reading position on a palette change (AC 4), and screenshots on every font (AC 9).

From step 1:

- **System on Linux.** iced uses the XDG desktop portal. Verify on this machine (Omarchy) that `theme_changes()` fires. If it doesn't, ship System as "follows the OS where available" and record it.
- **Palette values.** The dark palette needs design, not just inversion. Use `/impeccable` on DESIGN.md, and update DESIGN.md's palette row.

## Steps

Each step is test first, red then green, with the order test → build → clippy → fmt before moving on.

1. **Verify.** Check `theme_changes()` on this machine and how a blank-address page fits `Location`/`Navigation`/History.
2. **Settings model.** Add `Appearance` and `home` to `session::Settings`, plus events `ChooseAppearance` and `ChooseHome`. Store round trip, old-file and unknown-value tests. Update the feature `reading.feature`.
3. **Theme.** Dark palette and `reading(font, palette)`. Contrast unit test. `Theme` stays data only.
4. **Resolve System.** A small pure function `resolve(appearance, os_mode) → Palette` in `theme`. The app subscribes to `theme_changes()` and fetches `theme()` at startup, holding the OS mode in UI state, not in `session`.
5. **Plumb the palette** through `Pages` (cache key), `view`, the `Document` widget and the iced `Theme` for chrome. Acceptance: reading position kept (AC 4).
6. **Settings panel.** UI and Message wiring, moving the two menus out of the toolbar. Remove the one/two-row toolbar logic.
7. **Home and welcome.** The built-in document, the Home button and shortcut, the launch behaviour, the invalid-value refusal. New `features/settings.feature` and updates to `browse.feature` for Home.
8. **Screenshots and docs.** `scripts/shoot.sh` for both palettes across fonts. Update README, DESIGN.md, `features/README.md` ("theme switching" is no longer waiting), and VECTOR.md (ship criterion: palette done, text size and spacing remain).
9. **Pre-PR.** `/code-review`, fix findings, then `/pr`.

## Risks

- **Hardcoded colours in `ui`** (iced's default theme, scrollbars, pick lists) will show up as light patches in dark mode. Step 5 starts with a grep and screenshots.
- **OS signal on Linux.** System may not update live everywhere. The fallback and manual choices keep the feature usable.
- **Built-in address** is the one design choice that could ripple into session and navigation. That is why it is settled first.

## Not decided here

Whether Settings should later include text size and spacing in the same panel (yes, presumably, in "Reader controls") and how 2d's fallback toggle appears (a likely Settings entry). The panel should be easy to extend.
