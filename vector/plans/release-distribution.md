> **Status: deferred (2026-10-28).** Pick this up once web browsing (phase 2) is implemented: 2a, remote documents, was built 2026-10-29; 2b, images, is next. Whether 2a alone is enough to start is the product owner's call. Stage 1 (cross-platform CI and the platform fixes in the README) went ahead separately on `ci/cross-platform`; skip whatever of it is already done.

# mdbrowser-native — Downloadable releases for Linux, Windows and macOS

## Context
Today mdbrowser can only be used by cloning the repo and running `cargo run`. It has only ever been built and tested on Linux (CI is Ubuntu-only), and the README's shortcuts are wrong on macOS. The goal is for non-technical people to download an app for Linux, Windows or macOS, double-click it, and start reading. People who want to build from source still can.

Decisions already made:
- **Unsigned first:** a download page with clear "how to open it" steps for each OS. Signing can be added later without redesigning the pipeline.
- **First run:** a built-in welcome page, and the app offered for opening `.md` files. Windows and Linux get file association now; macOS later (see the macOS gap below).
- **Targets:**
  - Windows x64
  - macOS as one universal app (Apple Silicon and Intel)
  - Linux x64 and ARM64

What I found:
- **Linux binary:** the release build links only glibc, libm and libgcc_s. Wayland, X11 and Vulkan are loaded at runtime, and the fonts are embedded. That makes it portable as long as it is built on an old enough glibc (Ubuntu 22.04 gives glibc 2.35).
- **Size:** about 30 MB, of which about 5 MB is fonts.
- **No-argument launch:** `src/ui/app.rs:95` just focuses the address bar, so the window is empty.
- **The macOS gap:** winit 0.30.13 (under iced 0.14) never passes on the file a Mac user double-clicked. macOS sends that as an Apple Event, not as a command-line argument. So a `.md` association on macOS would open the app *without* the file. Fixing it needs a small Objective-C hook (`unsafe`, which ARCHITECTURE.md forbids without an ADR) or a newer winit. **Mac file association is deferred** and gets its own follow-up and ADR.
- Windows builds currently open a console window next to the app (`main.rs` has no `windows_subsystem` attribute).
- **Licence notices:** the fonts are OFL and the crates are MIT/Apache. All of them require their notices to ship *with* the binary, and none are bundled today.

## Approach
Four stages. Each is its own branch and PR into the protected `main`. I commit per step on each branch, and pause before every push, PR and tag.

### Stage 1: Cross-platform CI and platform fixes (`ci/cross-platform`)
1. **CI matrix** in `.github/workflows/ci.yml`, fastest gates first:
   - Format and clippy run once, on Linux.
   - Test and release build run on `ubuntu-22.04`, `ubuntu-24.04-arm`, `macos-latest` and `windows-latest`. They are separate jobs, so the required check list changes; I'll update branch protection to require each one.
2. **Fix failures test-first.** I expect them in Windows path and link handling:
   - `session::location`: `parse_file_url` with `file:///C:/…`, plus `normalise` and `is_network_path`
   - `session::links::classify`: relative hrefs against a `C:\…` base
   - `services::files`: the non-Unix branch at `src/services/files.rs:45`
   - the cucumber world's temporary-home paths

   Every fix starts with a failing test that runs on that OS.
3. **No console window on Windows:** add `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` to `src/main.rs`.
4. **`--version` flag:** prints the version and exits. Release CI uses it to smoke-test each packaged binary. The parsing is a pure function in `services::files`, next to `from_command_line`.
5. **README per platform:**
   - shortcuts in a table with a Mac column (Cmd+R / Cmd+L / Cmd+H)
   - the data folder on each OS
   - prerequisites: MSVC Build Tools on Windows, Xcode Command Line Tools on macOS
   - how to set `MDBROWSER_DATA_DIR` in PowerShell

### Stage 2: Welcome page (`feat/welcome-page`)
1. **Acceptance scenarios** in `features/reading.feature`:
   - Starting without a file shows the welcome page, titled "Welcome to mdbrowser".
   - The welcome page isn't recorded in History.
   - Back from the first real document returns to the welcome page.
   - External links on the welcome page open in the system browser.
2. **Content:** `content/welcome.md`, embedded with `include_str!` and exposed from `src/content.rs`. It explains what the app is, how to open a file (double-click a `.md` file, use "Open with", or type or paste a path in the address bar), the shortcuts, and links to the Web of Markdown and the repo. It has no relative links, because there is no base location.
3. **Reducer:** `session::step` gets an `Event::Welcome(source)`. It shows a parsed document with no `Location`, records no visit, and sets `Status::Ready` with the welcome title. `Follow` from the welcome page treats only absolute `file://`, `http(s)` and `mailto` links as followable.
4. **Startup:** in `ui::app` `boot`, a launch with no argument dispatches `Event::Welcome`.

### Stage 3: Packaging and release workflow (`build/release`)
1. **ADR-002** via `invest-adr`: the distribution approach.
   - packaging with cargo-packager
   - releases through GitHub Releases
   - unsigned for now, with signing as a later step
   - no macOS file association until the Apple Event gap is closed
2. **App identity:**
   - product name `mdbrowser`, identifier `com.gbrettmiller.mdbrowser`, version from `Cargo.toml`
   - an app icon: one source SVG in `assets/icon/`, rasterised to PNG, `.ico` and `.icns` by a script (`scripts/icons.sh`), with the outputs committed. The design goes through `/impeccable`, and DESIGN.md documents it.
3. **Licence notices:** `cargo-about` generates `THIRD-PARTY-NOTICES.html` covering the crate licences and the five OFL font licences. Every bundle ships it alongside `LICENSE*`.
4. **cargo-packager config** in `Cargo.toml` `[package.metadata.packager]`:
   - **Windows:** an NSIS installer that installs for the current user only, so no admin prompt. It adds a Start menu entry and registers `.md` and `.markdown` under "Open with", without taking over the default app.
   - **macOS:** a `.app` inside a `.dmg`. The universal binary is built for `aarch64` and `x86_64` and joined with `lipo` before packaging. No document types are declared yet.
   - **Linux:** a `.deb` that installs a `.desktop` file (`MimeType=text/markdown`, `Exec=mdbrowser %f`), plus an AppImage for other distributions. Both are built for x64 and ARM64.
5. **`.github/workflows/release.yml`**, triggered by a `v*` tag:
   - check that the tag matches the `Cargo.toml` version
   - run the full test suite on each OS
   - build and package
   - smoke-test each packaged binary with `--version`
   - generate `SHA256SUMS`
   - create a **draft** GitHub Release with the generated notes, for you to review and publish

### Stage 4: Download docs and first release
1. A **Download** section at the top of the README links to the latest release. It gives the steps for opening an unsigned app on each OS:
   - **macOS:** open it, see the warning, then approve it under System Settings → Privacy & Security → Open Anyway.
   - **Windows:** click "More info", then "Run anyway".
   - **Linux:** install the `.deb`, or make the AppImage executable.

   "Build from source" moves below it.
2. **VECTOR.md:** update the ship criteria (signing becomes its own later criterion) and add Mac file association to the open questions.
3. **Release candidate:** tag `v0.1.0-rc.1` → a draft release → we check it. Then tag `v0.1.0`. **I ask before each tag.**

## Critical files
- `.github/workflows/ci.yml` and a new `.github/workflows/release.yml`
- `src/main.rs`, `src/ui/app.rs` (`boot`), `src/session/{mod,reducer,location,links}.rs`, `src/services/files.rs`, `src/content.rs`, plus a new `content/welcome.md`
- `Cargo.toml` (packager metadata), `assets/icon/`, `scripts/icons.sh`, `about.toml` / `about.hbs` for cargo-about
- `features/reading.feature`, `tests/acceptance/steps.rs`
- `README.md`, `VECTOR.md`, `DESIGN.md`, `vector/decisions/ADR-002-*.md`

## Reuse
- `files::from_command_line` stays the single way a path from the OS reaches the app, whether it comes from `argv`, Windows "Open with" or Linux `%f`.
- `session::step` and the existing effect runner handle the welcome page. No new service is needed.
- `content.rs` already holds all user-facing strings, and the welcome text joins it.

## Verification
- **CI:** green on all four OS jobs, with each job required by branch protection.
- **Tests:** unit and acceptance tests pass on every OS, including the new welcome scenarios.
- **Release candidate:** `v0.1.0-rc.1` produces a draft release with seven artifacts (Windows installer, macOS dmg, Linux x64 and ARM64 `.deb` and AppImage) plus `SHA256SUMS`. Each passes the `--version` smoke test in CI.
- **Local Linux check:** install the x64 `.deb` and run the AppImage. Check that the app launches, shows the welcome page, opens a `.md` file from the file manager's "Open with", and that the notices file is present.
- **Manual check by you** (I can't run macOS or Windows here):
  - Windows: install, then open a `.md` file via "Open with".
  - macOS: open the `.dmg`, drag the app to Applications, and approve it once in Privacy & Security. Check the welcome page and opening a file by typing its path.
  - Both: follow the README's download steps exactly as written.

## Risks
- **Universal macOS build:** cargo-packager's universal-binary support is unverified. The fallback is to `lipo` the binaries in the workflow and point the packager at the joined binary.
- **No open-file dialog or drag-and-drop:** without either, a Mac user's only way to open a file in this release is typing or pasting a path. Pasting a path in Finder (Option+Cmd+C, then paste) works. Drag-and-drop onto the window would be small to add (iced reports `FileDropped`) if you change your mind.
- **Unsigned warnings will still lose some users.** The README steps reduce that but don't remove it.
- **glibc floor:** building on Ubuntu 22.04 sets the minimum at glibc 2.35. Older distributions can't run the AppImage.
