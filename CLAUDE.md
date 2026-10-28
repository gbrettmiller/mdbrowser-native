# mdbrowser-native

A browser for the web of Markdown, written in Rust with iced. It parses `.md` into a tree and draws it with its own layout engine, with no HTML, DOM or webview.

## Doctrine: read in this order
1. **VECTOR.md** — why this project exists, who it serves, and its direction. (Owned by Investiture.)
2. **ARCHITECTURE.md** — layers, stack, conventions and import rules. This is the structural authority. (Owned by Investiture.)
3. This file — operational rules.

The stack ownership model and cadence rule live in `~/.claude/CLAUDE.md` (§ Stack composition): the doctrine here belongs to Investiture, while execution and per-PR review belong to agentic-dev-team. Run `/invest-architecture` for periodic audits. `arch-review` checks boundaries per PR. ADRs go through `invest-adr` into `vector/decisions/`.

## Essential Commands

Run them in this order before calling a change done (test → build → lint):

```
cargo test
cargo build
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

`cargo run --release -- <path.md>` runs the app.

## Language notes

The global JS style defaults (quotes, semicolons, pnpm) don't apply here. rustfmt and clippy are the style authority. The functional-style preference still applies; see ARCHITECTURE.md § Conventions.

## Prior art

- `~/projects/mdbrowser`: the predecessor (Tauri + Svelte + webview). Its acceptance behaviour is carried over in `features/`.
- `~/projects/mdbrowser-spike`: the throwaway spike that validated this stack. Findings are in `vector/research/2026-10-28-spike-findings.md`. Its tested `model`, `session`, `links` and `layout` modules are reference material: port them deliberately, test first, rather than copying them wholesale.
