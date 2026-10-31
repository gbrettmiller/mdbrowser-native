> **Status: ready to start (2026-10-30).** Phases 1–5 have merged (the last was keyboard access, PR #14). Revised from the earlier 2026-10-30 draft to treat mdbrowser as a Web of Markdown browser, not a local file viewer: the web is the primary path and local files are secondary. Windows and Linux keep their `.md` association from the earlier draft; macOS association stays deferred. Stage 1 (cross-platform CI and platform fixes) and the welcome page already exist. The ADR is ADR-009.

# mdbrowser-native — Downloadable releases for Linux, Windows and macOS

## Context
Today mdbrowser can only be used by cloning the repo and running `cargo run`. The goal is for non-technical people to download an app for Linux, Windows or macOS, double-click it, and start reading the Web of Markdown. People who want to build from source still can.

mdbrowser is a browser for the web, so v0.1 is judged by how well it reads remote Markdown. Local files are supported: type or paste a path in the address bar, or use "Open with" on Windows and Linux. The app registers for `.md` but never takes over as the default.

Decisions already made:
- **Unsigned first:** a download page with clear "how to open it" steps for each OS. Signing can be added later without redesigning the pipeline.
- **First run:** the built-in welcome page, rewritten to lead with live Markdown sites (Stage 1, step 5).
- **File association:** Windows and Linux now ("Open with", not the default app); macOS later. See "Not in this release".
- **Targets:**
  - Windows x64
  - macOS as one universal app (Apple Silicon and Intel)
  - Linux x64 and ARM64

## What is already done
- **CI** runs fmt once, then clippy → test → release build on `ubuntu-22.04`, `ubuntu-24.04-arm`, `macos-latest` and `windows-latest`, behind one required `ci` check. Ubuntu 22.04 keeps the glibc floor at 2.35.
- **No console window on Windows:** `src/main.rs` sets `windows_subsystem = "windows"` in release builds.
- **Welcome page and Home:** a launch with no argument opens the home page, or the built-in `assets/welcome.md` (Alt+Home, or the Home button). It already links to the markdownWeb proposal and the scripting.com archive.
- **Command-line argument:** `mdbrowser <path-or-url>` already works through `files::from_command_line`. File association builds on it.
- **README:** per-platform prerequisites, a keyboard table with a macOS column, and the data folders.
- **Line endings:** text is checked out as LF on every platform.

## What is still true from the first survey
- **Linux binary:** the release build links only glibc, libm and libgcc_s. Wayland, X11 and Vulkan are loaded at runtime, and the fonts are embedded. It is portable as long as it is built on an old enough glibc. About 30 MB, of which about 5 MB is fonts. Re-measure at the start of Stage 1, because phases 2–5 added dependencies (`ureq`, `image`).
- **The macOS gap:** winit 0.30.x (under iced 0.14) never passes on the file a Mac user double-clicked. macOS sends it as an Apple Event, not a command-line argument, so a `.md` association would open the app *without* the file. Mac association is deferred (see "Not in this release").
- **Licence notices:** the fonts are OFL and the crates are MIT/Apache. All of them require their notices to ship *with* the binary, and none are bundled today.
- **Network is the main path:** the app fetches over `https` with no cookies. The packaged build must be checked against the network, not only against local files (see Verification).

## Approach
Three stages. Each is its own branch and PR into the protected `main`. I commit per step on each branch, and pause before every push, PR and tag. Every code change starts with a failing test.

### Stage 1: Release-ready binary (`feat/release-ready`)
1. **`--version` flag:** prints the version and exits before any window opens. Release CI uses it to smoke-test each packaged binary. The parsing is a pure function in `services::files`, next to `from_command_line`, with unit tests for `--version`, `-V`, a path, a URL and no argument. A `windows_subsystem = "windows"` build has no console to print to (attaching to the parent console needs `unsafe`, which ARCHITECTURE.md forbids without an ADR), so the Windows smoke test checks the exit code only. Record this in ADR-009.
2. **Open-with arguments:** confirm that `mdbrowser %f` (Linux) and `mdbrowser.exe "%1"` (Windows) both reach `from_command_line` unchanged, including paths with spaces and non-ASCII names. Add acceptance scenarios if any step is missing.
3. **App identity:**
   - product name `mdbrowser`, identifier `com.gbrettmiller.mdbrowser`, version from `Cargo.toml`
   - an app icon: one source SVG in `assets/icon/`, rasterised to PNG, `.ico` and `.icns` by `scripts/icons.sh`, with the outputs committed. The design goes through `/impeccable`, and DESIGN.md documents it.
4. **Licence notices:** `cargo-about` generates `THIRD-PARTY-NOTICES.html` covering the crate licences and the five OFL font licences (`about.toml`, `about.hbs`). Every bundle ships it alongside `LICENSE*`. CI fails if a dependency's licence isn't on the allow-list.
5. **Welcome page for the web:** rewrite `assets/welcome.md` so it opens with the Web of Markdown: what it is, and links to live Markdown sites (the markdownWeb proposal and examples, the scripting.com archive). Opening a local file gets one short line ("type or paste a path in the address bar"), not a section. The existing Home, keyboard and appearance sections stay. Strings stay in `content`. Update the acceptance scenario in `features/` that covers the welcome page if its wording is asserted.

### Stage 2: Packaging and release workflow (`build/release`)
1. **ADR-009** via `invest-adr`: the distribution approach.
   - packaging with cargo-packager
   - releases through GitHub Releases
   - unsigned for now, with signing as a later step
   - file association on Windows and the Linux `.deb` only, registered silently at install without taking the default; AppImages are not registered, and there is no macOS association until the Apple Event gap is closed
   - how the Windows `--version` smoke test works (Stage 1, step 1)
2. **cargo-packager config** in `Cargo.toml` `[package.metadata.packager]`:
   - **Windows:** an NSIS installer that installs for the current user only, so no admin prompt, with a Start menu entry. It registers `.md` and `.markdown` under "Open with", without taking over the default app.
   - **macOS:** a `.app` inside a `.dmg`. The universal binary is built for `aarch64` and `x86_64` and joined with `lipo` before packaging. No document types are declared yet.
   - **Linux:**
     - a `.deb` for Debian-family distributions (Debian, Ubuntu, Mint, Pop!_OS). It installs a `.desktop` file (`MimeType=text/markdown`, `Exec=mdbrowser %f`).
     - an AppImage for every other distribution, including Arch, Fedora and openSUSE. An AppImage registers nothing by itself, so "Open with" works only if the user integrates it (for example with AppImageLauncher). File association is a `.deb` and Windows feature.
     - both are built for x64 and ARM64.
3. **`.github/workflows/release.yml`**, triggered by a `v*` tag:
   - check that the tag matches the `Cargo.toml` version
   - run the full test suite on each OS
   - build and package
   - smoke-test each packaged binary with `--version`
   - generate `SHA256SUMS`
   - create a **draft** GitHub Release with generated notes, for you to review and publish. Tags containing `-rc` are marked pre-release.
   - Keep the build steps in sync with `ci.yml`, as CLAUDE.md requires.

### Stage 3: Download docs and first release (`docs/download`)
1. A **Download** section at the top of the README links to the latest release. It gives the steps for each OS:
   - **macOS:** open the `.dmg`, drag the app to Applications, open it, see the warning, then approve it under System Settings → Privacy & Security → Open Anyway.
   - **Windows:** run the installer, click "More info", then "Run anyway".
   - **Ubuntu, Debian and derivatives:** install the `.deb`.
   - **Arch, Fedora and everything else:** make the AppImage executable and run it. Note the `libfuse2` requirement if the RC check shows it is needed.

   "Build from source" moves below it. A short "Reading Markdown" note says the app opens web addresses, and that local files open with "Open with" (Windows, and Linux when installed from the `.deb`), by typing or pasting a path, or from the command line. The Arch/AppImage steps say the AppImage needs integration for "Open with".
2. **VECTOR.md:** add ship criteria for downloadable installers on three OSes and for signing (its own later criterion). Record the items under "Not in this release" as open questions. Update the Status line in the README.
3. **Release candidate:** tag `v0.1.0-rc.1` → a draft release → we check it. Then tag `v0.1.0`. **I ask before each tag.**

## Not in this release
Each of these can be added later without redesigning anything. The README and VECTOR.md list them so users and the roadmap know.
- **macOS file association:** winit 0.30.x never passes on the file a Mac user double-clicked (it arrives as an Apple Event). Fixing it needs a small Objective-C hook (`unsafe`, so an ADR) or a newer winit. Check the winit version iced resolves first.
- **Drag-and-drop onto the window:** small; iced reports `FileDropped`.
- **File dialog:** needs a new dependency and an ADR.
- **Code signing and notarisation.**
- **Other Linux packages:** an AUR package (`mdbrowser-bin`), Flatpak, RPM.

## Critical files
- `.github/workflows/ci.yml` (kept in sync) and a new `.github/workflows/release.yml`
- `src/main.rs`, `src/services/files.rs`, `src/ui/app.rs` (`boot`)
- `tests/acceptance/steps.rs` and `features/` if open-with needs scenarios
- `assets/welcome.md`, `src/content.rs`, `features/` (welcome scenario)
- `Cargo.toml` (packager metadata), `assets/icon/`, `scripts/icons.sh`, `about.toml` / `about.hbs`
- `README.md`, `VECTOR.md`, `DESIGN.md`, `vector/decisions/ADR-009-*.md`

## Reuse
- `files::from_command_line` stays the single way a path from the OS reaches the app, so `--version` parsing sits beside it, and so does open-with.
- The existing welcome page, Home button and links are the first-run experience; Stage 1 only rewrites its wording.

## Verification
- **CI:** green on all four OS jobs before each PR merges.
- **Release candidate:** `v0.1.0-rc.1` produces a draft pre-release with seven artifacts (Windows installer, macOS dmg, Linux x64 and ARM64 `.deb` and AppImage) plus `SHA256SUMS`. Each passes the `--version` smoke test in CI.
- **Remote check, on each platform:** install the packaged app, launch it, and from the welcome page follow a link to a live Markdown page (the markdownWeb proposal or the scripting.com archive). Check that it loads over `https`, renders with images, and that a link to a second page works. CI can't do this without a display, so it is a manual step for you on Windows and macOS. I do it on Linux.
- **Windows installer script (me, before the RC):** read the NSIS script cargo-packager generates and confirm it only adds the `.md` and `.markdown` handlers and does not set the app as the default for the extension.
- **Local Linux check (me):**
  - Install the x64 `.deb`, and check the launcher entry and that the notices file is present.
  - Run the x64 AppImage, here on Arch. Confirm it does not appear under "Open with" until integrated.
  - Run the AppImage on a clean Ubuntu environment and note whether `libfuse2` is needed.
  - In each, check the welcome page, the remote check above, opening a local file by pasting its path, and, for the `.deb`, opening a `.md` file from the file manager's "Open with" and checking that `xdg-mime query default text/markdown` is unchanged.
- **Manual check by you** (I can't run macOS or Windows here):
  - Windows: install, launch, run the remote check, paste a local path, and open a `.md` file via "Open with". Check that the app is listed there but the default app for `.md` hasn't changed (Settings → Apps → Default apps).
  - macOS: open the `.dmg`, drag the app to Applications, approve it once in Privacy & Security, run the remote check, paste a local path.
  - Both: follow the README's download steps exactly as written.

## Risks
- **Universal macOS build:** cargo-packager's universal-binary support is unverified. The fallback is to `lipo` the binaries in the workflow and point the packager at the joined binary.
- **AppImage has no file association:** unintegrated AppImages (the Arch route) don't appear in "Open with". Accepted for v0.1 and documented in the README; they open files by pasting a path or from the command line.
- **Windows default-app claim is unverified:** the installer is expected to register without taking over the default. The RC checks the NSIS script and a clean install to confirm.
- **Mac users can only type or paste a path:** with no association, no dialog and no drag-and-drop, that is their only way to open a file. Pasting from Finder (Option+Cmd+C, then paste) works. Drag-and-drop is the cheapest next step if users ask.
- **TLS roots on packaged builds:** the remote check exists to catch a packaged binary that can't verify certificates on some OS. Look at which root store `ureq` uses before the RC.
- **AppImage and FUSE:** Ubuntu 22.04 and later don't ship `libfuse2`, and it is unconfirmed whether cargo-packager's AppImage needs it. The RC check settles this.
- **Windows `--version` output:** a GUI-subsystem binary prints nothing to a console, so the smoke test can only check the exit code there.
- **Unsigned warnings will still lose some users.** The README steps reduce that but don't remove it.
- **glibc floor:** building on Ubuntu 22.04 sets the minimum at glibc 2.35. Older distributions can't run the AppImage.
- **Size growth:** the binary has grown since the 30 MB measurement; check it against the AppImage and installer sizes before the RC.
