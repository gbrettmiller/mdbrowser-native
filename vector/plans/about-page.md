# Plan: About page

**Created**: 2026-11-03
**Branch**: `feat/about-page`, from `main` (this plan and ADR-013 travel on `docs/about-page`)
**Status**: approved 2026-11-03

## Goal

Give the app the standard About content before it is released as a download: version and build, what the app is, licence and third-party credits, what it sends and stores, keyboard shortcuts, and a short FAQ. It is a built-in Markdown page (`Location::About`, address `about:mdbrowser`) opened by one event from a Settings button, F1, Ctrl/Cmd+? and a welcome-page link, not a menu, panel or toolbar button (ADR-013). The release plan (tabled) keeps packaging, signing and the licence-notices file; this plan builds only what the reader sees in the app.

Spec artifacts: none from `/specs`. The Gherkin below is the spec and lands in `features/about.feature`, each scenario added in the step that makes it pass.

**Facts checked while planning:**
- `Location::Welcome` is the precedent: blank address, no History entry (`loaded()` skips `visits::record`), `from_welcome` link policy (web and mail only), no images (`services/images.rs`), served by `services::runner` from `include_str!("../../assets/welcome.md")`. About reuses all of it.
- In-page anchors already resolve before location policy (`Link::Scroll`, `links/mod.rs`), so `[FAQ](#faq)` works on any page.
- F1 is unused in `keys::command`; Alt+Home is the nearest precedent for a chord that sends an event.
- `ReadingFont::ALL` lists four reading fonts; `assets/fonts/` also has JetBrains Mono (code). Licence files: OFL for Literata and JetBrains Mono, `LICENSE.md` for Source Serif 4 and iA Writer Quattro, `LICENSE.txt` for Inter. The page names them from the files, checked in step 2.
- `session/reducer.rs`, `session/mod.rs` and `ui/app.rs` are near the 300-line file guide: new code goes in new modules (`session/about.rs`, and `ui/` wiring as small as possible).
- `Cargo.toml` has `license = "MIT OR Apache-2.0"`, version `0.1.0`; `git remote` is `github.com/gbrettmiller/mdbrowser-native`.

## Decisions made in this draft (gBRETT may overturn at approval)

1. **Address `about:mdbrowser`**, case-insensitive; bare `about:` is an alias. Any other `about:` address typed in the bar is an address error; the same reference as a link is unsupported (the address parser and the link classifier differ today).
2. **No History entry**, like Welcome; Back and Forward work. F1 while About is showing does nothing (no duplicate Back entry).
3. **Two chords:** F1, and Ctrl+? / Cmd+? (Shift+/). F1 needs Fn on many Mac keyboards, so the second chord carries the Mac and laptop case. The Settings button's tooltip names both.
4. **Opening About closes the Settings and History panels.** About is a page: two overlays at 620 px compete with it. The reader reopens Settings from the toolbar.
5. **The Settings button is at the top of the panel**, under the "Settings" heading, so it is visible without scrolling.
6. **Licence texts are not embedded.** The page names each licence and says the full texts ship in `THIRD-PARTY-NOTICES.html` with each release (the release plan's `cargo-about` step) and are in the repository. **Release gate:** no build is distributed before that file ships in the bundle (OFL requires the notice to accompany the font). A line is added to the release plan's checklist.
7. **Commit hash is optional**: `option_env!("MDBROWSER_COMMIT")`; a plain build shows no commit. No build script. The platform line is `std::env::consts::OS` and `ARCH`, for example "linux x86_64".
8. **The welcome page links to About** with the literal text "About mdBrowser". About does not link to Welcome (it has a blank address). If release step 5 rewrites `assets/welcome.md`, that link moves with it.
9. **Smaller slice than first drafted.** Behaviour already covered by unit tests (case, mail confirmation, the reader's look, source view) is not repeated as acceptance scenarios. A "Report a problem" link with a prefilled issue body, a changelog link and a copy-version button are out.

## Scope

In: `Location::About` with predicates replacing `is_welcome`; its address, links, images and History rules; `assets/about.md`; a pure fill of the template; the runner serving it; `Event::About`; F1 and Ctrl/Cmd+?; the Settings button with its strings in `content`; the welcome link; scenarios; a test that restricts `ureq` to `services/web`; README, VECTOR, ARCHITECTURE notes; ADR-013 accepted; screenshots.

Out: a native menu (release plan), embedding the licence texts, an update check, a changelog page, prefilled issue links, `--version` (release plan, but it shares `session::about::version()` so the two cannot diverge), localisation, a second internal page, improving accessibility beyond ADR-008.

## Acceptance Criteria

1. [ ] **Reachable.** Settings → "About mdBrowser" (top of the panel), F1, Ctrl/Cmd+?, the welcome-page link, and typing `about:mdbrowser` or `about:` each show the About page. F1 works with focus in the address bar and in the Home page field.
2. [ ] **A normal page.** Title "About mdBrowser"; address bar `about:mdbrowser`; no History entry; Back returns to the previous page and Forward returns to About; Reload shows it again; Source (Ctrl/Cmd+U) shows its filled text; it follows the reader's font, size, spacing and palette; F1 on About is a no-op (no second Back entry); opening it closes Settings and History.
3. [ ] **True facts.** The page shows the Cargo version and the licence from `Cargo.toml` (tests compare), the platform as `OS ARCH`, and a commit only when supplied. No `{…}` placeholder is left. Every `ReadingFont` and JetBrains Mono is named with the licence its file in `assets/fonts/` states. Every shortcut the page lists is accepted by `keys::command` (the test lives in `ui` or the acceptance world, not `session`).
4. [ ] **Links.** From About: `http(s)` navigates in the app, `mailto:` waits for confirmation, `#faq` scrolls, `about:mdbrowser` stays, any other reference is unsupported. The welcome page may link to `about:mdbrowser` and to no other internal address.
5. [ ] **No network, no images.** Opening About emits one load effect served by the runner, no web request and no image load (a reducer test and the "site was not asked" scenario).
6. [ ] **Keyboard and focus.** The Settings button is a Tab stop with the ring and Enter activates it. After F1 from the address bar, focus is on the page so scroll keys work. The page scrolls with the usual keys.
7. [ ] **Nothing else moves.** Goldens and existing scenarios pass unchanged; the settings scenarios in `features/settings.feature` are checked for a control-list assertion (none found means none to update).
8. [ ] **Privacy cannot drift unnoticed.** A test fails if `ureq` is used outside `services/web` (so "no analytics, no update check" breaks loudly when someone adds a call). The prose in "What it sends and stores" cites its source per sentence (step 2); that check is **manual**, once, at release.
9. [ ] **Doctrine.** ADR-013 `accepted` via `invest-adr`; README (a features line, the shortcuts), VECTOR (a Direction line), ARCHITECTURE (the Session row names the built-in locations About and Welcome, Project Structure lists `assets/about.md`, and a sentence says `runner` supplies the build facts: compile-time constants, not I/O) updated; the release plan gets the licence-notices release gate and a "read the About prose" checklist line.
10. [ ] **CI and screenshots.** Test, build, clippy and fmt pass on the four platform jobs. Screenshots (`shoot.sh` with `-k` for F1, or a typed address) in the default look, at 32 px text, and at 620 by 360: pass means the title, version line and every section heading are fully inside the window width, wrapped not clipped, and legible. A failure fixes the page and amends the plan.

## User-Facing Behavior

Existing steps reused: `I press {string}`, `I pressed {string}`, `I enter …`, `I go back`, `I go forward`, `I reload`, `the focus is on the address bar`, `History lists …`, `the welcome page is not in History`, `I see the welcome page`, `I see the document titled {string}`, `I see the source view`, `the site was not asked for {string}`. New steps (in `steps/about.rs`): `I open Settings and choose About`, `I follow the welcome page's About link`, `the address bar shows {string}`, `the page shows the version from Cargo.toml`, `the page shows this platform`. Scenarios are added in the step that makes each pass (marked).

```gherkin
Feature: About

  Background:
    Given a document "post.md" containing "# Post"
    And I am viewing "post.md"

  # step 3
  Scenario: Open About from Settings
    When I open Settings and choose About
    Then I see the document titled "About mdBrowser"
    And the address bar shows "about:mdbrowser"
    And Settings is closed

  # step 3
  Scenario Outline: Open About from the keyboard
    When I press "<chord>"
    Then I see the document titled "About mdBrowser"
    Examples:
      | chord  |
      | F1     |
      | Ctrl+? |

  # step 3
  Scenario: F1 works from the address bar
    Given the focus is on the address bar
    When I press "F1"
    Then I see the document titled "About mdBrowser"

  # step 3
  Scenario: F1 on About does nothing more
    Given I pressed "F1"
    When I press "F1"
    And I go back
    Then I see the document titled "Post"

  # step 1
  Scenario: Type the address
    When I enter "about:mdbrowser"
    Then I see the document titled "About mdBrowser"

  # step 1
  Scenario: An unknown about address is an error
    When I enter "about:nothing"
    Then the error shows the address "about:nothing"

  # step 2
  Scenario: About shows the version and platform
    When I press "F1"
    Then the page shows the version from Cargo.toml
    And the page shows this platform

  # step 3
  Scenario: About is not in History, and Back and Forward work
    Given I pressed "F1"
    Then History does not list "About mdBrowser"
    When I go back
    Then I see the document titled "Post"
    When I go forward
    Then I see the document titled "About mdBrowser"

  # step 3
  Scenario: About starts no request
    When I press "F1"
    Then the site was not asked for "/"

  # step 4
  Scenario: Open About from the welcome page
    Given the app starts with no address given
    When I follow the welcome page's About link
    Then I see the document titled "About mdBrowser"
```

Unit-tested instead of repeated as scenarios (criteria 2 to 4): address case and the bare `about:`, link policy (web, mail, anchor, unknown, no link back to Welcome), reload, the reader's look and the source view on About, the Settings button's tab order, and F1 when Settings or the source view is open.

## Steps

Before step 1: count non-test lines in `session/reducer.rs`, `session/mod.rs` and `ui/app.rs` (each near 300) and plan any split first; new code goes in `session/about.rs` and `ui/settings/about.rs`.

### Step 1: The location (session, pure)

**Complexity**: standard
**RED**: tests in `session/location.rs`, `address.rs`, `links/mod.rs`, `images/resolve.rs`, `web/mod.rs`: `Location::About.address()` is `about:mdbrowser`, its label is "About mdBrowser", it has no host and is not a file, and its origin is `Origin::Public` like Welcome's (`session/web` test `the_welcome_page_is_a_public_origin`); `parse_address` accepts `about:mdbrowser` in any case and bare `about:`, and gives an address error for `about:other`; links from a built-in page: `about:mdbrowser` navigates, `about:other` is unsupported, web and mail as Welcome today, and no built-in links to Welcome; images from About are refused; predicates `Location::is_builtin()`, `records_history()`, `allows_images()` replace `is_welcome` at every site (delete `is_welcome`); `loaded()` records no visit for About. Scenarios "Type the address" and "An unknown about address is an error" go red then green.
**GREEN**: the `About` variant and the matches it forces; `from_welcome` becomes `from_builtin` with one `about` branch.
**REFACTOR**: None needed.
**Files**: `src/session/{location,address,links/mod,images/resolve,web/mod,reducer}.rs`, `src/services/images.rs`, `features/about.feature`, `tests/acceptance/steps/about.rs`
**Commit**: `feat(session): add the About location, its address and link rules`

### Step 2: The page (filler, asset, runner)

**Complexity**: standard
**RED**: `session/about.rs`: `fill(template, &Facts)` replaces `{version}`, `{build}`, `{licence}` and leaves no `{`; a missing commit leaves the platform only; `version()` equals `env!("CARGO_PKG_VERSION")`; the licence line equals `Cargo.toml`'s `license`; the asset has the title "About mdBrowser", a `# FAQ` heading (anchor `faq`), and names every `ReadingFont` and JetBrains Mono with the licence its file in `assets/fonts/` states (test in `theme` or `session`, reading the files). A services test: the runner serves `Location::About` filled, `fallback_from: None`, and a template with an unknown placeholder fails the unit test (never at runtime: the template is compiled in). A test fails when `ureq` appears outside `src/services/web`. Scenario "About shows the version and platform".
**GREEN**: `assets/about.md` (what it is; version and build; licence and credits with the notices file named; what it sends and stores; keyboard; FAQ; where to report a problem); `session::about::{Facts, fill, version}`; the runner supplies facts from `CARGO_PKG_VERSION`, `std::env::consts` and `option_env!("MDBROWSER_COMMIT")`.
**Prose check.** Each sentence in "What it sends and stores" is written with its source beside it in the commit message: fetches carry only `Accept`, no cookies, no `Referer` (ADR-003); only addresses the reader opens are fetched, plus images when the Images setting allows (ADR-004, ADR-007); mail links ask first (ADR-010); History and settings are on this computer, naming the directory `services::store` uses; no account, analytics or update check (the `ureq` test). This is a manual check; a reviewer repeats it at release.
**REFACTOR**: None needed.
**Files**: `assets/about.md`, `src/session/about.rs`, `src/session/mod.rs`, `src/services/runner.rs`, tests beside them
**Commit**: `feat(about): add the About page and fill in its build facts`

### Step 3: The event and the routes (session, ui, content)

**Complexity**: standard
**RED**: reducer tests: `Event::About` starts a load of `Location::About` (one arm in `reducer.rs` calling `about::open`), records no visit, closes Settings and History, leaves the source view, and is a no-op when About is already current (no new Back entry); no image or web effect. `keys` tests (in `ui`): F1 and Ctrl/Cmd+? map to `Event::About`; every shortcut the About page lists is accepted by `command`; typing in the address bar is unaffected. `ui` test: the Settings panel has the About button under the heading, a Tab stop with the ring. The scenarios marked step 3 go red then green.
**GREEN**: `Event::About`; `session::about::open`; the key mappings; `content::{ABOUT, ABOUT_TOOLTIP}` ("Version, licence and help (F1 or Ctrl+?)", Cmd on macOS); the button in `ui/settings.rs`; focus goes to the page after the event.
**REFACTOR**: if `settings.rs` passes 300 non-test lines, split into `ui/settings/about.rs`.
**Files**: `src/session/{mod,about,reducer}.rs`, `src/ui/{keys,settings,app}.rs`, `src/content.rs`, `features/about.feature`, `tests/acceptance/steps/about.rs`
**Commit**: `feat(about): open About from Settings, F1, Ctrl/Cmd+? and the address bar`

### Step 4: The welcome link, doctrine and screenshots

**Complexity**: trivial
**RED**: the welcome-page content test asserts a link with the text "About mdBrowser" to `about:mdbrowser`; the scenario "Open About from the welcome page".
**GREEN**: the welcome line; ADR-013 `accepted` (aliases, two chords, panels close, button at the top, release gate), README, VECTOR, ARCHITECTURE as in criterion 9; the release plan's gate and checklist lines; screenshots per criterion 10, read and judged.
**REFACTOR**: None needed.
**Files**: `assets/welcome.md`, `README.md`, `VECTOR.md`, `ARCHITECTURE.md`, `vector/decisions/ADR-013-about-page.md`, `vector/plans/release-distribution.md`
**Commit**: `docs: ADR-013 and the About page in VECTOR, README and ARCHITECTURE`

## Complexity Classification

| Step | Rating | Why |
|------|--------|-----|
| 1 | standard | A new variant in a closed enum touches several matches; pure and well tested. |
| 2 | standard | New module and content; the prose check is the risk, not the code. |
| 3 | standard | Wires session, keys and ui; follows the Home and source-view patterns. |
| 4 | trivial | Docs and a welcome link. |

## Pre-PR Quality Gate

- [ ] `cargo test` (unit and acceptance)
- [ ] `cargo build`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo fmt --check`
- [ ] `/code-review` and `arch-review`
- [ ] Docs updated (README, VECTOR, ARCHITECTURE, ADR-013 accepted)
- [ ] Screenshots read and judged (criterion 10)
- [ ] A human reads the About prose once (criterion 9)

## Risks & Open Questions

- **No native menu entry.** A macOS reader may look in the app menu first. Mitigation: F1, Ctrl/Cmd+?, the Settings button at the top of the panel and the welcome link; the release plan can add an app-menu item that sends `Event::About`. Owner: gBRETT, at packaging.
- **Prose goes stale.** Tests cover the version, fonts and shortcuts; the FAQ and privacy text need a human check at each release. Mitigation: criterion 9 and a line in the release checklist.
- **The repository URL.** The page links to `github.com/gbrettmiller/mdbrowser-native`; if the project moves, the page does. One place to change.
- **The licence of record.** The page says `MIT OR Apache-2.0` from `Cargo.toml`; a test compares the two so they cannot drift.
- **The notices file does not exist yet.** The page names `THIRD-PARTY-NOTICES.html` (release plan, `cargo-about`). The release gate in decision 6 stops a build from shipping before the file does; until then About describes what the release will contain, and nothing is distributed.
- **Name.** The release plan says product name `mdbrowser`; the app's name is mdBrowser. The page uses mdBrowser; the release plan's packaging metadata is reconciled when it resumes.
- **ADR-009 is unwritten.** ADR-013 depends only on `session::about::version()`, which `--version` will share.
- **Accessibility.** Same limit as every page (ADR-008); not improved here.

## Plan Review Summary

Four reviewers (acceptance, design, UX, strategic) returned needs-revision on the first draft. All blockers are addressed above.

- **Blockers fixed.** F1 alone fails on Macs (Fn): added Ctrl/Cmd+? and moved the Settings button to the top of the panel. Licence compliance: the release gate and the named `THIRD-PARTY-NOTICES.html`. Unobservable or missing scenarios: reworked, with real step names; F1 from the address bar, F1 on About, the no-request scenario and the unknown address are covered.
- **Warnings taken.** Panels close on About (no competing overlays); `about:` alias and a clear error; predicates replace `is_welcome`; one `about` branch in `from_builtin`; a one-line reducer arm; the key-table test lives in `ui`; scenarios split across the steps that make them pass; a test restricts `ureq` to `services/web`; Cargo licence and version tests; slice trimmed (duplicates moved to unit tests).
- **Not taken.** A prefilled issue link, a changelog link, a copy-version button (listed as out). `shoot.sh` is not changed; screenshots use `-k` keys or a typed address.
- **Observations kept.** Accessibility is the ADR-008 limit; manual prose check at release; the name mdBrowser vs the release plan's `mdbrowser`; welcome link may move when the release plan rewrites `assets/welcome.md`.
