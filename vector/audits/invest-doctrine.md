## Doctrine Audit

**Files audited:** VECTOR.md, CLAUDE.md, ARCHITECTURE.md (cross-checked against `src/`, `tests/`, `Cargo.toml`, `.github/workflows/ci.yml`)
**Project stage:** development
**Run:** 2026-10-30, on `feat/phase-3` at 4f35ffa

### Findings

#### CONTRADICTION — high
- `VECTOR.md:Research Status` says "Community examples: Not yet in the corpus", but `corpus/markdownweb/` holds examples 0–8 and three extras (the phase 3 plan surveys them).

#### STRUCTURE — medium
- `ARCHITECTURE.md:Project Structure` doesn't declare `tests/` (`acceptance/` and `support/`), which is a source directory. The Stack table mentions `tests/acceptance/`, but the tree doesn't.
- `ARCHITECTURE.md:Project Structure` doesn't declare `DESIGN.md` (the Impeccable visual system doc) or `corpus/examples.md`, the new examples index. `corpus/` is declared, so the index is covered; `DESIGN.md` is not.
- `vector/audits/` did not exist before this run and is not in the declared `vector/` tree (`decisions/`, `plans/`, `research/`, `schemas/`).

#### DRIFT — medium
- `ARCHITECTURE.md:Conventions` says files stay under roughly 300 lines. Ten files are over: `session/tests.rs` (651), `services/web/mod.rs` (498), `model/mod.rs` (441), `session/web/mod.rs` (380), `session/images/planner_tests.rs` (365), `session/images/resolve.rs` (351), `tests/acceptance/steps/local.rs` (346), `tests/acceptance/world.rs` (337), `session/images/planner.rs` (336) and `services/web/fetch.rs` (314). Most are test files or have inline tests. Either the rule should say "non-test code" or these should be split.
- `ARCHITECTURE.md:Stack` omits crates in `Cargo.toml`: `open` (behind `services::opener`), `libc` (unix only, in `services::files`) and the dev-dependency `flate2`.

#### GAP — low
- `VECTOR.md:Ship Criteria` has "Builds and passes CI on Linux. macOS and Windows build in CI" unchecked, but `ci.yml` already builds a four-OS matrix (Ubuntu x64 and ARM64, macOS, Windows). It can probably be ticked once the first run on `main` is green.
- `VECTOR.md:Ship Criteria` "Renders the markdownWeb examples 0–8 …" is unchecked. Phase 3 AC 8 (screenshots) is the open piece. This is accurate, so no change is needed until the screenshots are read.
- `ARCHITECTURE.md:Conventions` has no naming table (file types, casing, examples). It only says "Rust defaults". That's enough for Rust, but `invest-architecture` has no declared rule for feature files, ADRs, plans or corpus files.
- `ARCHITECTURE.md:Layers` says `services` is the only layer touching `std::net`, but `session/network.rs` imports `IpAddr`, `Ipv4Addr` and `Ipv6Addr`. These are value types used to refuse local-network targets, not I/O. The rule should say "performing I/O". `layout/golden.rs` reads `std::fs` inside tests, which is also fine in practice but unmentioned.
- `Cargo.toml` names the package `mdbrowser`, while the repo, `VECTOR.md` and the docs say `mdbrowser-native`. Worth a decision, but not blocking.

#### PLACEHOLDER — info
- None. No template text remains in any doctrine file.

### What checked out
- **VECTOR.md** is complete and specific: problem, audience, value proposition, direction (phases 1–5), "What This Is Not", five principles, constraints, Definition of Done, ship criteria, six assumptions, open questions. Phase 3 is recorded as built and ADR-006 is cited.
- **ARCHITECTURE.md** layer rules match the code. No `use crate::ui` or `use crate::services` appears in `model`, `layout`, `session` or `theme`. `model` has no `crate::` imports outside itself. Only `ui` uses `content`. I/O (`std::fs`, `std::process`, `std::thread`) is confined to `services`, apart from the items above.
- **Stack** matches `Cargo.toml` for every declared crate. ADR-001 to ADR-006 are all `accepted`, and every ADR referenced in ARCHITECTURE.md exists.
- **CLAUDE.md** points at real paths (`~/projects/mdbrowser`, `~/projects/mdbrowser-spike`, `vector/research/…`), its command list matches `ci.yml`, and it doesn't contradict ARCHITECTURE.md.
- **Persona vs. depth:** the global instruction is "default to concise" and ARCHITECTURE.md is a reference spec, not teaching text. No mismatch.
- **Constraints vs. stack:** "no HTML, DOM or webview" holds. The dependency list has none. HTTP is `ureq` without cookies, as ADR-003 requires.

### Summary
- Critical: 0 | High: 1 | Medium: 5 | Low: 5 | Info: 0
- Doctrine health: GAPS FOUND. Nothing blocks `invest-architecture`; the one high finding is a stale table row.

### Recommended Actions
1. `VECTOR.md:Research Status` — change "Community examples" from "Not yet in the corpus" to point at `corpus/markdownweb/` and `corpus/examples.md`.
2. `ARCHITECTURE.md:Project Structure` — add `tests/` (`acceptance/`, `support/`), `DESIGN.md` and `vector/audits/`, and add `open`, `libc` and `flate2` to the Stack table.
3. `ARCHITECTURE.md:Conventions` — decide the 300-line rule: scope it to non-test code, or split `services/web/mod.rs` and `model/mod.rs` first (the two largest with real logic).

### Resolved after this run
Actions 1 and 2 were applied the same day: the Research Status row, the project tree (`tests/`, `DESIGN.md`, `audits/`) and the Stack table (`open`, `libc`, `flate2`). Action 3 (the 300-line rule) is open for gBRETT.
