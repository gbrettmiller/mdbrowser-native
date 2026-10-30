> **Status: ready to start (2026-10-30).** Phases 1–5 have merged (the last was keyboard access, PR #14), so this is the plan the distribution deferral was waiting on. Revised from the 2026-10-28 draft: Stage 1 (cross-platform CI and platform fixes) and Stage 2 (welcome page) are already done, and the ADR is now ADR-009.

# mdbrowser-native — Downloadable releases for Linux, Windows and macOS

## Context
Today mdbrowser can only be used by cloning the repo and running `cargo run`. The goal is for non-technical people to download an app for Linux, Windows or macOS, double-click it, and start reading. People who want to build from source still can.

Decisions already made:
- **Unsigned first:** a download page with clear "how to open it" steps for each OS. Signing can be added later without redesigning the pipeline.
- **First run:** the built-in welcome page (done), and the app offered for opening `.md` files. Windows and Linux get file association now; macOS later (see the macOS gap below).
- **Targets:**
  - Windows x64
  - macOS as one universal app (Apple Silicon and Intel)
  - Linux x64 and ARM64

## What is already done
- **CI** runs fmt once, then clippy → test → release build on `ubuntu-22.04`, `ubuntu-24.04-arm`, `macos-latest` and `windows-latest`, behind one required `ci` check. Ubuntu 22.04 keeps the glibc floor at 2.35.
- **No console window on Windows:** `src/main.rs` sets `windows_subsystem = "windows"` in release builds.
- **Welcome page and Home:** a launch with no argument opens the home page, or the built-in `assets/welcome.md` (Alt+Home, or the Home button).
- **README:** per-platform prerequisites, a keyboard table with a macOS column, and the data folders.
- **Line endings:** text is checked out as LF on every platform.

## What is still true from the first survey
- **Linux binary:** the release build links only glibc, libm and libgcc_s. Wayland, X11 and Vulkan are loaded at runtime, and the fonts are embedded. It is portable as long as it is built on an old enough glibc. About 30 MB, of which about 5 MB is fonts. Re-measure at the start of Stage 1, because phases 2–5 added dependencies (`ureq`, `image`).
- **The macOS gap:** winit 0.30.x (under iced 0.14) never passes on the file a Mac user double-clicked. macOS sends that as an Apple Event, not as a command-line argument. A `.md` association on macOS would open the app *without* the file. Fixing it needs a small Objective-C hook (`unsafe`, which ARCHITECTURE.md forbids without an ADR) or a newer winit. **Mac file association is deferred** and gets its own follow-up and ADR. Check the winit version iced resolves first; a newer iced may have closed this.
- **Licence notices:** the fonts are OFL and the crates are MIT/Apache. All of them require their notices to ship *with* the binary, and none are bundled today.
- **No way to open a file from inside the app:** there is no file dialog and no drag-and-drop. A user opens a file by passing it on launch ("Open with"), or by typing or pasting a path in the address bar. See Risks.

## Approach
Three stages. Each is its own branch and PR into the protected `main`. I commit per step on each branch, and pause before every push, PR and tag. Every code change starts with a failing test.

### Stage 1: Release-ready binary (`feat/release-ready`)
1. **`--version` flag:** prints the version and exits before any window opens. Release CI uses it to smoke-test each packaged binary. The parsing is a pure function in `services::files`, next to `from_command_line`, with unit tests for `--version`, `-V`, a path, a URL and no argument. On Windows a `windows_subsystem = "windows"` build has no console to print to, so the smoke test needs a check that works there: either attach to the parent console (`AttachConsole`, which needs `unsafe`, so no), or have the Windows smoke test run the debug-subsystem build. Decide this in the ADR; the likely answer is to smoke-test by launching with `--version` and checking the exit code only.
2. **Open-with arguments:** confirm that `mdbrowser %f` (Linux) and `mdbrowser.exe "%1"` (Windows) both reach `from_command_line` unchanged, including paths with spaces and non-ASCII names. Add acceptance scenarios if any step is missing.
3. **App identity:**
   - product name `mdbrowser`, identifier `com.gbrettmiller.mdbrowser`, version from `Cargo.toml`
   - an app icon: one source SVG in `assets/icon/`, rasterised to PNG, `.ico` and `.icns` by `scripts/icons.sh`, with the outputs committed. The design goes through `/impeccable`, and DESIGN.md documents it.
4. **Licence notices:** `cargo-about` generates `THIRD-PARTY-NOTICES.html` covering the crate licences and the five OFL font licences (`about.toml`, `about.hbs`). Every bundle ships it alongside `LICENSE*`. CI fails if a dependency's licence isn't on the allow-list.

### Stage 2: Packaging and release workflow (`build/release`)
1. **ADR-009** via `invest-adr`: the distribution approach.
   - packaging with cargo-packager
   - releases through GitHub Releases
   - unsigned for now, with signing as a later step
   - no macOS file association until the Apple Event gap is closed
   - how the Windows `--version` smoke test works (Stage 1, step 1)
2. **cargo-packager config** in `Cargo.toml` `[package.metadata.packager]`:
   - **Windows:** an NSIS installer that installs for the current user only, so no admin prompt. It adds a Start menu entry and registers `.md` and `.markdown` under "Open with", without taking over the default app.
   - **macOS:** a `.app` inside a `.dmg`. The universal binary is built for `aarch64` and `x86_64` and joined with `lipo` before packaging. No document types are declared yet.
   - **Linux:** a `.deb` that installs a `.desktop` file (`MimeType=text/markdown`, `Exec=mdbrowser %f`), plus an AppImage for other distributions. Both are built for x64 and ARM64.
3. **`.github/workflows/release.yml`**, triggered by a `v*` tag:
   - check that the tag matches the `Cargo.toml` version
   - run the full test suite on each OS
   - build and package
   - smoke-test each packaged binary with `--version`
   - generate `SHA256SUMS`
   - create a **draft** GitHub Release with generated notes, for you to review and publish. Tags containing `-rc` are marked pre-release.
   - Keep the build steps in sync with `ci.yml`, as CLAUDE.md requires.

### Stage 3: Download docs and first release (`docs/download`)
1. A **Download** section at the top of the README links to the latest release. It gives the steps for opening an unsigned app on each OS:
   - **macOS:** open it, see the warning, then approve it under System Settings → Privacy & Security → Open Anyway.
   - **Windows:** click "More info", then "Run anyway".
   - **Linux:** install the `.deb`, or make the AppImage executable.

   "Build from source" moves below it. The README also says how to open a file (pass it on launch, or paste a path).
2. **VECTOR.md:** add ship criteria for downloadable installers on three OSes and for signing (its own later criterion), and add Mac file association to the open questions. Update the Status line in the README.
3. **Release candidate:** tag `v0.1.0-rc.1` → a draft release → we check it. Then tag `v0.1.0`. **I ask before each tag.**

## Critical files
- `.github/workflows/ci.yml` (kept in sync) and a new `.github/workflows/release.yml`
- `src/main.rs`, `src/services/files.rs`, `src/ui/app.rs` (`boot`)
- `Cargo.toml` (packager metadata), `assets/icon/`, `scripts/icons.sh`, `about.toml` / `about.hbs`
- `tests/acceptance/steps.rs` and `features/` if open-with needs scenarios
- `README.md`, `VECTOR.md`, `DESIGN.md`, `vector/decisions/ADR-009-*.md`

## Reuse
- `files::from_command_line` stays the single way a path from the OS reaches the app, whether it comes from `argv`, Windows "Open with" or Linux `%f`.
- `assets/welcome.md` already covers first run; it only needs a line on opening files if the README's wording changes.

## Verification
- **CI:** green on all four OS jobs before each PR merges.
- **Release candidate:** `v0.1.0-rc.1` produces a draft pre-release with seven artifacts (Windows installer, macOS dmg, Linux x64 and ARM64 `.deb` and AppImage) plus `SHA256SUMS`. Each passes the `--version` smoke test in CI.
- **Local Linux check:** install the x64 `.deb` and run the AppImage. Check that the app launches, shows the welcome page, opens a `.md` file from the file manager's "Open with", and that the notices file is present.
- **Manual check by you** (I can't run macOS or Windows here):
  - Windows: install, then open a `.md` file via "Open with".
  - macOS: open the `.dmg`, drag the app to Applications, and approve it once in Privacy & Security. Check the welcome page and opening a file by typing its path.
  - Both: follow the README's download steps exactly as written.

## Risks
- **Universal macOS build:** cargo-packager's universal-binary support is unverified. The fallback is to `lipo` the binaries in the workflow and point the packager at the joined binary.
- **No open-file dialog or drag-and-drop:** without either, a Mac user's only way to open a file in this release is typing or pasting a path. Pasting a path in Finder (Option+Cmd+C, then paste) works. Drag-and-drop onto the window would be small to add (iced reports `FileDropped`) and could join Stage 1 if you want it; a file dialog needs a new dependency and an ADR.
- **Windows `--version` output:** a GUI-subsystem binary prints nothing to a console, so the smoke test can only check the exit code there.
- **Unsigned warnings will still lose some users.** The README steps reduce that but don't remove it.
- **glibc floor:** building on Ubuntu 22.04 sets the minimum at glibc 2.35. Older distributions can't run the AppImage.
- **Size growth:** the binary has grown since the 30 MB measurement; check it against the AppImage and installer sizes before the RC.
