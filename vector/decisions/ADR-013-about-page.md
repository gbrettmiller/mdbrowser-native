# ADR-013: About is a built-in Markdown page, not a menu or a panel

**Date:** 2026-11-03
**Status:** accepted
**Deciders:** gBRETT (product owner)

## Context

Before the app is released as a download, a reader needs the standard "About" things in one place: the version, what the app is, its licence and third-party credits, what it sends over the network, the keyboard shortcuts, and a short FAQ. The app has no native menu bar: it is iced, with its own toolbar, and the 620 by 360 minimum window already truncates the address box (ADR-008). Plan: `vector/plans/about-page.md`. The release plan (`vector/plans/release-distribution.md`, tabled, ADR-009 reserved) already needs licence notices to ship with the binary; this decision covers what the reader sees inside the app.

## Decision Drivers

- ADR-001: the app draws its own pages; a new view should reuse the layout engine, not add a second renderer.
- VECTOR.md: the reader owns the presentation. About in the reader's font, size, spacing and palette is on-doctrine; a custom-styled dialog is not.
- ADR-008: one side panel at the minimum window width, and the toolbar has no room for another button. Every feature needs a keyboard route.
- The welcome page is the precedent: a bundled Markdown document (`assets/welcome.md`), the built-in `Location::Welcome`, no History entry, its own link and image policy.
- ADR-010: web links open in the app, mail leaves only after the reader agrees. About's links follow it.
- ADR-011: the source view works on any shown page, so About inherits it.
- Claims on the page (what the app sends, where settings live) must stay true. They are checked by tests against the code, not trusted.
- ARCHITECTURE.md: pure session logic, I/O only in `services`, user-facing strings in `content`, no `unsafe`.

## Options Considered

### Option A: A native application menu with an About item

**Pros:**
- The platform convention on macOS.

**Cons:**
- iced 0.14 has no menu bar; it needs a new dependency (`muda` or similar) with platform code, and probably `unsafe` on macOS. Three platforms, three behaviours, none testable in the acceptance world.

### Option B: A third side panel or a toolbar button

**Pros:**
- Discoverable.

**Cons:**
- ADR-008 allows one panel at the minimum window. A button does not fit the toolbar at 620 px. A panel is a small, fixed-width column: poor for a FAQ and a licence list, and it is a second renderer for long text.

### Option C: An About section inside the Settings panel

**Pros:**
- No new surface.

**Cons:**
- Settings holds choices that change the app. A FAQ and credits there make the panel taller than a small window already is (it scrolls once colour fields show), and give the reader no Back, Source or copy.

### Option D: A built-in Markdown page, `Location::About` (chosen)

Bundled `assets/about.md`, filled with the build's version, and opened like any page.

**Pros:**
- Reuses everything: layout, selection and copy, text size, font, palette, links, Back/Forward, Source view.
- Testable in the acceptance world like any page.
- Content is Markdown a contributor can edit without touching Rust.

**Cons:**
- Not where macOS users look first (the app menu); see consequences.

## Decision

**We will add a built-in About page: `Location::About`, served from `assets/about.md` with the build's facts filled in, opened by `Event::About`.**

- **Address.** `about:mdbrowser`, shown in the address bar and accepted when typed in any letter case; bare `about:` is an alias. Any other `about:` address typed is an address error, and as a link it is unsupported. It is not a file or a URL. Like Welcome it makes no History entry; it has a Back entry in the session like any page, and Reload re-serves it.
- **Entry points.** A button "About mdBrowser" at the top of the Settings panel, under its heading, so it needs no scrolling; F1 and Ctrl/Cmd+? (F1 needs Fn on many Mac keyboards, so the second chord carries them); a link on the welcome page. All send the one event, `Event::About`. Opening About closes the Settings and History panels, since two overlays compete with a page at 620 px, and F1 on About does nothing more. A future native menu (the release plan's packaging stage) would send the same event.
- **Content.** The app name and version, a build line (platform, and the commit when the build supplies one), a one-paragraph description, the licence (`MIT OR Apache-2.0`), the repository and issue links, the bundled fonts with their licences, a statement of what the app sends and stores, the keyboard shortcuts, and a short FAQ.
- **Build facts.** The version is `CARGO_PKG_VERSION`; the platform is `std::env::consts`; the commit is `option_env!("MDBROWSER_COMMIT")`, set by release CI and absent in a plain build. No build script. A pure function in `session` fills `{version}`, `{build}` placeholders in the template; the runner supplies the facts. The same version function will back `--version` in the release plan.
- **Links.** From About, and from the welcome page, links to `http`, `https` and `mailto` work as they do from Welcome, and so does `about:mdbrowser`, with in-page anchors (`#faq`) scrolling. Anything else is unsupported. Images are not loaded for built-in pages.
- **Third-party notices.** The page names each bundled font and its licence and says the full texts ship in `THIRD-PARTY-NOTICES.html` with each release (the release plan's `cargo-about` step). They are not embedded in the page, which would add hundreds of kilobytes to a document laid out in one piece. **Release gate:** the OFL fonts require their notice to accompany the binary, so no build is distributed before that file ships in the bundle; the release plan carries the gate.
- **Truth.** Tests check that the version on the page equals the Cargo version, that every bundled font and every keyboard shortcut named on the page exists, and that no placeholder is left unfilled. A test fails if `ureq` is used outside `services/web`, so "no analytics, no update check" breaks loudly if a call is added. The privacy and FAQ prose is checked by hand against ADR-003 and ADR-010 when written and again at release; it states only what the code does.

## Consequences

**Positive:**
- One new location and one event; no new widget, panel, dependency or `unsafe`.
- About follows the reader's look, is selectable and copyable (a reader can paste the version into a bug report), and works from the keyboard.
- The release plan's notices step and `--version` flag share the version source.

**Negative:**
- No entry in a native menu bar, so a macOS reader looking at the app menu will not find "About". The release plan can add an app-menu item later that sends `Event::About`. Until then the Settings button, F1 and Ctrl/Cmd+? are the routes.
- The About page has the same accessibility limit as every page: custom-drawn text, no structure for a screen reader (ADR-008).
- The page is Markdown text that can go stale. Tests cover the checkable facts (version, fonts, shortcuts); the prose (FAQ, privacy) needs a human check at each release.
- No in-app licence viewer: a reader offline sees the licence names, and the texts are in the installed notices file and the repository.

**Neutral:**
- The address `about:mdbrowser` is the first reserved internal address. If a second built-in page is added, `Location` is the place to generalise.
- The source view of About shows the filled page text, front matter and placeholders resolved.

## Related Decisions

- ADR-001: Native rendering with our own layout engine.
- ADR-003: HTTP client (what a fetch carries; the privacy statement cites it).
- ADR-007: Reader controls (About follows the reader's look).
- ADR-008: Keyboard access (F1, one panel at the minimum width).
- ADR-010: Web links open in the app (the link policy and mail confirmation).
- ADR-011: The source view (works on About).
- ADR-009 (reserved, release plan): packaging, notices and `--version`. This ADR depends only on `session::about::version()`, which `--version` will share.
