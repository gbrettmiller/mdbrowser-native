# Architecture

**Last Updated:** 2026-10-29

This file is the technical specification: layers, stack, conventions, structure and import rules. For project intent and direction, see VECTOR.md. Read VECTOR.md first, then this file, before writing code. `/invest-architecture` audits the codebase against what is declared here.

## Stack

Rust (edition 2024). Rationale: ADR-001.

| Concern | Crate |
|---|---|
| UI shell; the document is our own widget | iced 0.14 |
| Text shaping and line breaking | cosmic-text, through iced's shared font system so that measuring and painting agree |
| Markdown | pulldown-cmark, with tables, strikethrough, task lists and YAML metadata |
| Frontmatter | yaml-rust2 |
| History and settings files | serde, serde_json |
| Per-user data directory | dirs |
| Unit tests | `cargo test` (built-in), insta for golden layout snapshots |
| Acceptance | Gherkin in `features/`, run by the `cucumber` crate (`tests/acceptance/`), with tempfile and futures |
| Lint | rustfmt (`rustfmt.toml`) and clippy with `-D warnings` |

Fonts are bundled: `assets/fonts/` holds static Literata, Source Serif 4, iA Writer Quattro, Inter and JetBrains Mono (each with its OFL licence), embedded at build time. `theme::fonts::install()` evicts host faces with the same family names and corrects variant metadata that upstream files get wrong.

## Rendering pipeline

`fetch → parse → lay out → paint`. Each arrow is a pure function except fetch.

```
bytes ──model::parse──▶ Document (tree) ──layout::layout(theme, width)──▶ Page (fragments + fills) ──ui paint──▶ pixels
```

## Layers

| Layer | Location | Rule |
|-------|----------|------|
| **Model** | `src/model/` | Markdown source → owned document tree. Pure. Depends only on pulldown-cmark. |
| **Layout** | `src/layout/` | Tree + theme + width → `Page`; hit-testing, carets, selection and copy text. Pure apart from reading the shared font database. |
| **Session** | `src/session/` | Address parsing, link classification, back/forward, History, and the reducer `step(state, event) → (state, effects)`. Pure. Effects are data. |
| **Theme** | `src/theme/` | The reader stylesheet: tokens (colour, type scale, spacing) and bundled fonts. Data only. |
| **Content** | `src/content.rs` | All user-facing strings. |
| **Services** | `src/services/` | All I/O: reading files, the external opener, the store (History and settings), and `runner`, which turns effects into iced tasks that report back as events. The only layer that touches `std::fs`, `std::net` or `std::process`. |
| **UI** | `src/ui/` | The iced application: dispatches events to the reducer, runs effects through `services::runner`, and draws state. Holds the document widget (paints a `Page`, turns input into messages) and the page cache. Owns no browsing logic. |

Every module belongs to exactly one layer. If you're unsure where something goes, and it can be pure, put it in `model`, `layout` or `session`.

## Import Direction

- `ui` may use every other layer.
- `services` may use `model` and `session` types, never `ui`.
- `layout` may use `model` and `theme`.
- `model` and `theme` use nothing else from this crate. `session` uses `model` (to parse loaded documents) and `theme::ReadingFont` (settings).
- Pure layers (`model`, `layout`, `session`, `theme`) never perform I/O and never use iced's runtime (`Task`, `Element`). Iced *value* types (`Color`, `Font`, `Rectangle`) are allowed.

## How to Add a Feature

1. **Acceptance scenario** in `features/` describing the behaviour.
2. **Pure logic first**, in `model`, `layout` or `session`, test first.
3. **Content and theme** additions as needed.
4. **Service**, if it needs I/O.
5. **UI last**: wire messages and paint.

## Conventions

- Rust defaults: `snake_case` modules and functions, `CamelCase` types, formatted by rustfmt.
- Functional style: functions take values and return new values. Mutation stays local to a function (builders, accumulators) and never leaks through shared state. No trait-object hierarchies without an ADR.
- No `unsafe`.
- Tests live beside the code in `#[cfg(test)] mod tests`. Layout gets golden tests over the corpus.
- Files stay under roughly 300 lines. Split by responsibility before that.

## What Not to Do

- **Never produce or interpret HTML.** Raw HTML in a document is shown as literal text.
- No I/O outside `services`.
- No hardcoded colours, sizes or fonts outside `theme`.
- No inline user-facing strings outside `content`.
- No business logic in `ui`.
- No dependency on host font metadata for the reading face. Bundle the fonts.
- No implementation before tests.

## Project Structure

- `src/` — the crate, one directory per layer; `lib.rs` exposes the layers, `main.rs` only starts `ui::app`
- `assets/fonts/` — bundled typefaces and their licences
- `features/` — Gherkin acceptance scenarios
- `corpus/` — sample Markdown documents for golden tests and manual checks
- `scripts/shoot.sh` — screenshots documents in each font (Hyprland + grim). It uses a scratch store via `MDBROWSER_DATA_DIR`, which redirects History and settings away from the real data directory.
- `vector/` — Investiture doctrine tree: `decisions/`, `research/`, `schemas/`
- `.github/workflows/` — CI: fmt → clippy → test → build
