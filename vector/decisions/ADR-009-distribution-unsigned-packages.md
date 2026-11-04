# ADR-009: Releases are unsigned packages on GitHub Releases, built with cargo-packager

**Date:** 2026-11-04
**Status:** accepted
**Deciders:** gBRETT (product owner)

## Context

Today mdBrowser can only be run by cloning the repo and building it. VECTOR.md says downloadable apps for Linux, Windows and macOS come after phase 5, so that non-technical readers can install without Rust. Stage 1 of `vector/plans/release-distribution.md` made the binary release-ready: `--version`, the icon, `THIRD-PARTY-NOTICES.html` (ADR-013's release gate), and a welcome page that leads with the web. This ADR covers how that binary is packaged, signed, published and registered with each OS.

## Decision Drivers

- Non-technical readers must be able to double-click an installer. This is the VECTOR.md goal for the first public release.
- ADR-013 release gate: no build ships without `THIRD-PARTY-NOTICES.html` and the `LICENSE*` files in each bundle.
- ARCHITECTURE.md: no `unsafe`. This rules out attaching to the parent console on Windows to print `--version`.
- The Rust toolchain is the only build dependency. The project has no Node or Python build step, and `scripts/notices.sh` needs only `cargo-about` and python3.
- The release has one maintainer. The signing certificates (an Apple Developer ID, a Windows code-signing certificate) cost money and need account setup, so signing can't block v0.1.
- The app is a web browser, so a `.md` association must never take over the reader's default Markdown app.
- winit 0.30.x (under iced 0.14) doesn't deliver the macOS "open this file" Apple Event, so a macOS association would open the app without the file.
- CI already covers four targets: Linux x64 on Ubuntu 22.04 (glibc 2.35 floor), Linux ARM64, macOS and Windows.

## Options Considered

### Option A: cargo-packager, unsigned, GitHub Releases (chosen)

One tool, configured in `Cargo.toml` `[package.metadata.packager]`, produces an NSIS installer, a macOS `.app` in a `.dmg`, a Linux `.deb` and an AppImage. A `v*` tag triggers `release.yml`, which builds, smoke-tests and creates a draft GitHub Release.

**Pros:**
- The config sits in `Cargo.toml` next to the product name and icons already declared there.
- One tool covers all three OSes, and the metadata is declared once.
- Signing can be added later without changing the pipeline.

**Cons:**
- cargo-packager is younger than the alternatives. Its universal macOS binary support and its AppImage's `libfuse2` requirement are unverified, so the release candidate has to check both.
- Unsigned apps show OS warnings that will lose some readers.

### Option B: Per-OS native tooling (cargo-wix, cargo-bundle, cargo-deb, a hand-written AppImage script)

Each tool is mature for its own format: an MSI through WiX on Windows, a hand-built `.app` and `.dmg` on macOS, `cargo-deb` for the `.deb`, and `appimagetool` for the AppImage.

**Pros:**
- Each tool is better proven for its own format. `cargo-deb` in particular is likely a safer `.deb` path.
- Behaviour is explicit, with less uncertainty about what a packager generates.
- Failures are isolated, so one broken tool doesn't block the other platforms.

**Cons:**
- Four tools and four configs for one maintainer.
- The name, identifier, icons, version, MIME types and licence files would be declared in several places and could drift.
- The plist, dmg step and AppImage script would be our own untested glue, checked only by CI on a tag.
- Windows file-association behaviour would live in a hand-written `.wxs`, which is more to verify.
- More release-workflow surface to keep in sync with `ci.yml`.

### Option C: Sign and notarise from the first release

**Pros:**
- No OS warnings, and a better first impression.

**Cons:**
- It needs a paid Apple Developer account and a Windows certificate, plus secrets management in CI, before there is any evidence that anyone wants the download.

### Option D: Distribution through package managers only (AUR, Flatpak, Homebrew, winget)

**Pros:**
- Trusted channels and automatic updates.

**Cons:**
- It doesn't serve the non-technical reader who wants a download link. Each channel has its own review process and maintenance cost. These remain candidates for later.

## Decision

**We will package with cargo-packager and publish unsigned builds as GitHub Releases.**

- **Targets:**
  - a Windows x64 NSIS installer, installed for the current user only
  - a macOS universal `.dmg` (built for aarch64 and x86_64 and joined with `lipo`)
  - a Linux `.deb` and an AppImage, each for x64 and ARM64
- **Release workflow:** a `v*` tag runs `release.yml`.
  - It checks that the tag matches the `Cargo.toml` version and runs the full tests on each OS.
  - It builds and packages, smoke-tests each packaged binary with `--version`, and generates `SHA256SUMS`.
  - It creates a draft release. Tags containing `-rc` are marked pre-release, and the maintainer reviews and publishes the draft.
- **File association:** Windows and the Linux `.deb` only. Both register `.md` and `.markdown` as "Open with" (the `.desktop` file has `MimeType=text/markdown` and `Exec=mdbrowser %f`), silently at install and without taking the default. AppImages register nothing, because registering is up to the user's integration tool. macOS declares no document types until the Apple Event gap is closed.
- **Windows smoke test:** a `windows_subsystem = "windows"` binary has no console, and attaching to the parent console would need `unsafe`, which ARCHITECTURE.md forbids. On Windows the smoke test therefore checks the exit code only. Linux and macOS also check that stdout contains the version.
- **Notices:** every bundle ships `THIRD-PARTY-NOTICES.html` and `LICENSE*` (ADR-013).
- **Signing and notarisation:** a later step. The README's download steps cover the OS warnings in the meantime.
- **Fallback to per-platform tooling (Option B):** cargo-packager is the default for every platform, and the release candidate is its test. If it falls short on a platform, that platform alone switches to native tooling, for example `cargo-deb` for the `.deb`, or `lipo` plus `hdiutil` for the macOS `.dmg`. The other platforms stay on cargo-packager. Triggers: a universal macOS build that doesn't work, an NSIS script that sets the default app, an AppImage or `.deb` with missing files or wrong metadata, or an unfixable packager bug. A switch is recorded as an amendment to this ADR.

This is the only option that fits one maintainer and one config file, and it serves the download-and-double-click reader without waiting on certificates.

## Consequences

**Positive:**
- One config and one workflow cover three OSes, and everything lives in the repo.
- Signing can be added later by adding secrets and a step, without redesigning the pipeline.
- The default `.md` app is never changed.
- No platform is locked in to cargo-packager, because each can fall back to native tooling.

**Negative:**
- Unsigned apps trigger macOS Gatekeeper and Windows SmartScreen warnings.
- The Windows smoke test is weaker, because it can't read the version from the output.
- AppImage users, including Arch, don't get "Open with" unless they integrate the AppImage themselves.
- macOS users can only open a local file by typing or pasting a path.
- Ubuntu 22.04 sets the glibc 2.35 floor for the Linux builds.
- No other Linux packaging (an AUR package, Flatpak, RPM) exists yet.
- If the fallback is used, that platform's metadata is declared a second time and needs its own glue.

**Neutral:**
- Drag-and-drop onto the window, a file dialog and macOS association are separate follow-ups. The plan lists them under "Not in this release".
- A release is a draft until the maintainer publishes it, and the maintainer is asked before each tag.

## Related Decisions

- ADR-013: About page. It is the source of the notices release gate, and `--version` shares `session::about::version()`.
- ADR-001: the bundled fonts, whose OFL licences must ship with the binary.
- ADR-003: the HTTP client. The RC checks which TLS root store `ureq` uses on the packaged builds.
- ADR-010: web links open in the app. This is the reason the welcome page leads with live sites in the remote check.

## Amendment (2026-11-04): "Open with" comes from our templates, not the packager's file associations

Reading cargo-packager 0.11.8 before writing the config showed that its `file-associations` setting does the wrong thing for us. On Windows its NSIS macro writes the extension's default class (`HKCU\Software\Classes\.md`), which claims the default app. On macOS it adds `CFBundleDocumentTypes`, which would list the app in "Open with" but open it without the file (the Apple Event gap). On Linux it sets `Exec=… %F`. The setting is global, not per format.

We therefore do not use `file-associations`. The Windows installer uses a copy of the packager's NSIS template (`packaging/windows/installer.nsi`) that registers the app under `Applications\mdbrowser.exe` and lists it in `.md\OpenWithProgids` and `.markdown\OpenWithProgids`, and never writes an extension's default. The `.deb` uses its own desktop template (`packaging/linux/mdbrowser.desktop`) with `MimeType=text/markdown;` and `Exec=mdbrowser %f`. macOS gets no document types. `tests/packaging.rs` guards all of this.

This is the "NSIS script sets the default app" fallback trigger, handled by customising templates rather than switching tools. The cost is that we own a copy of the NSIS template and must re-diff it against upstream when cargo-packager is upgraded.
