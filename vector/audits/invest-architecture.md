## Architecture Audit

**Scope:** whole project: `src/` (71 `.rs` files), with `tests/` checked for size only
**Doctrine source:** ARCHITECTURE.md (last updated 2026-10-31), VECTOR.md, CLAUDE.md
**Run:** 2026-10-31, on `main` at 79d0a12. The two low findings were fixed the same day (see below).
**Files scanned:** 71

### Violations

#### STATE — info
- `src/services/web/fetch.rs:54`, `:65`, `:134` — `Arc<dyn Resolver>` is a trait object. ARCHITECTURE.md allows none "without an ADR" for hierarchies, and no ADR names `Resolver`. It is one trait used as a test seam, not a hierarchy, so it likely needs a sentence in ADR-003 or ADR-004 rather than a change.

#### SIZE — info
- `src/ui/document.rs` — 451 non-test lines. Doctrine limit is about 300.
- `src/ui/focus.rs` — 347 non-test lines.
- `src/ui/app.rs` — 333 non-test lines.

### Fixed after the run
- `ui/app.rs` no longer reads `std::env::args()`; `services::files::launch_address` (pure, tested) and `launch_address_from_environment` do.
- The link underline's alpha is now `Theme::underline` in `theme`, not a literal in `layout/text.rs`. The nearby pixel offsets stay in `layout`: the doctrine treats them as geometry.

### Checked and clean
- **Import direction.** `model` and `theme` import nothing from other layers. `layout` imports only `model` and `theme`. `session` imports no `ui`, `services`, `layout` or `content`. `services` imports no `ui`, `layout` or `content`. `content` imports only `session` and `theme`. Only `ui` uses `content`.
- **Purity.** No iced runtime types (`Task`, `Element`, widgets, subscriptions) in `model`, `layout`, `session` or `theme`. The iced items used there are value types (`Color`, `Font`, `Rectangle`, `Point`, image `Handle`) plus font-system access for `layout` and `theme`, which the doctrine allows.
- **I/O.** `std::fs`, `std::net`, `std::process` and `std::thread` appear outside `services` only in tests (`layout/golden.rs`, `theme/chrome_text.rs`) and as value types (`session/network.rs`: `IpAddr` and friends).
- **No `unsafe`** anywhere in `src/` or `tests/`.
- **No HTML interpretation or webview dependency.**
- **Colours, fonts, sizes.** The only colour literals outside `theme` are test fixtures and `Color::TRANSPARENT` for quad fills. Font names outside `theme` are in `content` (display labels) and tests. The one `ui` use of a font family goes through `theme::ReadingFont`.
- **User-facing strings.** No inline strings in `ui` widgets; the only literal in `layout` that reads like one is inside a test.
- **No business logic in `ui`.** It doesn't match on `Link`, parse addresses or classify locations; it reads `State` and sends `Event`s.
- **Naming.** Every module and directory is `snake_case`, and Rust defaults are the only declared convention.
- **Structure.** Every module belongs to a declared layer, and the only loose files in `src/` are `lib.rs`, `main.rs` and `content.rs`.
- **Gates.** `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` pass. `cargo test` was not run for this audit; CI on `main` is green.

### Summary
- High: 0 | Medium: 0 | Low: 0 | Info: 4
- Architecture health: CLEAN. No layer, import or purity violations; the remaining findings are advisories.
