## Doctrine Audit

**Files audited:** VECTOR.md, CLAUDE.md, ARCHITECTURE.md (cross-checked against `src/`, `tests/`, `Cargo.toml`, README.md, ADR-003, ADR-005, ADR-006 and the release plan)
**Project stage:** development
**Run:** 2026-10-31, on `main` at 43248d7 (after PR #17). CI on `main` is green.

### Findings

#### CONTRADICTION — high
- `VECTOR.md:What This Is Not` says "Non-Markdown links are handed off to the system", but since PR #17 every web link is tried in the app first and only `mailto:` leaves, after the reader agrees.

#### DRIFT — high
- `VECTOR.md:Direction 2d` and `ADR-005:Option B` say the reader is told which address was shown (a notice names the original address). The banner was removed in PR #17; the address bar is the signal and `Ctrl/Cmd+Shift+O` opens the original. ADR-005 needs an amendment note, as ADR-003 has for its changes.
- `README.md` still says "Other `http` and `https` links, and `mailto` links, open in your system's handler" and "a **Listen** line … opens in your system's player". Web links now open in the app, and a Listen link tries the app first and ends on the error page's **Open in browser**. The `Ctrl+Shift+O` row also omits that it opens the original of a fallback page. `VECTOR.md:Direction 3` and `ADR-006` repeat the enclosure claim.

#### DRIFT — medium
- `ADR-003:92` says local-network links "that aren't Markdown still go to the system browser". They are now refused outright (`LocalNetworkFromWeb`), and nothing but mail leaves.
- `ARCHITECTURE.md:Conventions` says non-test code stays under about 300 lines. Three files are over: `ui/document.rs` (451), `ui/focus.rs` (347) and `ui/app.rs` (333).

#### GAP — medium
- `VECTOR.md:Ship Criteria` leaves two boxes unticked that are done: "The reader can set typeface, text size, spacing and a light or dark palette, and the choices persist" (phases 2c and 4) and "Builds and passes CI on Linux. macOS and Windows build in CI" (four-OS matrix, green on `main`).
- `VECTOR.md:Ship Criteria` has two boxes that are genuinely open: "Renders the markdownWeb examples 0–8 and the scripting.com archive…" and "Opening a typical post takes under 250 ms… no stutter". The second has no measurement on record, and the first depends on the open "Leading images and tall images" question.

#### GAP — low
- `ARCHITECTURE.md:Layers` (Session) doesn't mention the leave-the-app confirmation (`State.leaving`, `ConfirmLeaving`, `StayHere`). `Last Updated` still reads 2026-10-30.
- `ARCHITECTURE.md:Layers` (Services) says `services` is the only layer touching `std::net`, but `session/network.rs` imports `IpAddr` and friends as value types. The rule should say "performing I/O". Carried over from the last audit.
- `Cargo.toml` names the package `mdbrowser`, while the repo, `VECTOR.md` and `CLAUDE.md` say `mdbrowser-native`. Carried over; the binary name matches the README, so this is a naming decision, not a bug.

#### PLACEHOLDER — info
- None. `layout/golden.rs` and `theme/chrome_text.rs` read `std::fs` inside tests only, which is fine in practice but not mentioned in the I/O rule.

### What checked out
- Every path in `ARCHITECTURE.md:Project Structure` exists, and `src/` holds exactly the declared layers. `tests/` and `DESIGN.md`, flagged last time, are now declared.
- Import direction holds: no `ui` or `services` imports in `model`, `layout`, `session` or `theme`, and only `ui` uses `content`.
- No `unsafe` in the source; the three grep hits are the word "unsafe" in the mail-link checks in `services/opener.rs`.
- No colour literals outside `theme` apart from one test fixture (`layout/stops.rs`).
- The stack table matches `Cargo.toml` (`open`, `libc` and `flate2` are now listed), and ADR-001 to ADR-008 exist and are referenced.
- `CLAUDE.md` matches `ci.yml` and does not contradict `ARCHITECTURE.md`.
- The previous run's high contradiction (the corpus is no longer "not yet in") is fixed.

### Summary
- Critical: 0 | High: 3 | Medium: 4 | Low: 3 | Info: 1
- Doctrine health: GAPS FOUND. Nothing blocks `invest-architecture`, but the doctrine describes the link behaviour from before PR #17.

### Recommended Actions
1. Update the link wording to match PR #17 in `VECTOR.md` (What This Is Not, Direction 2d, Direction 3), `README.md` (the two bullets and the `Ctrl+Shift+O` row) and `ADR-003:92`. Add an amendment note to `ADR-005` for the removed banner. Record the change as a short ADR via `invest-adr`: web links are tried in-app and only mail leaves, after confirmation.
2. Tick the two finished Ship Criteria, and decide how the open ones will be closed: read the markdownWeb screenshots, measure the 250 ms open, and settle the leading-image question.
3. Decide whether to split `ui/document.rs`, `ui/focus.rs` and `ui/app.rs`, or reword the 300-line rule.
