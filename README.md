# mdbrowser

A browser for the [Web of Markdown](https://github.com/scripting/markdownWeb). It reads `.md` files, parses them into a document tree and draws that tree with its own layout engine. No HTML, no DOM, no webview.

Writers own the words. Readers own the presentation. The document chooses nothing about how it looks; the reader picks the typeface, and raw HTML in a document is never interpreted.

> **Status:** phase 1, local files only. Remote (`http(s)`) browsing, front matter navigation and more reader controls are next. See [VECTOR.md](VECTOR.md) § Direction.

## What it does today

- Opens local Markdown by path, `~/` path or `file://` URL, and follows links between `.md` files.
- Renders CommonMark plus GitHub tables, task lists and strikethrough. In-page `#anchors` use GitHub heading slugs.
- Takes the window title from front matter `title`, then the first heading, then the file name.
- Back, forward, reload, and a History panel that persists across restarts.
- Four bundled reading faces: Literata (default), Source Serif 4, iA Writer Quattro and Inter. Code is set in JetBrains Mono.
- Select text with the mouse and copy it with Ctrl+C.
- `http`, `https` and `mailto` links open in your system's handler. Other schemes are blocked.

## Build and run

Needs Rust 1.88 or newer.

```sh
cargo run --release -- corpus/index.md
```

| Key | Action |
|---|---|
| Alt+← / Alt+→ | Back / forward |
| Ctrl+R or F5 | Reload |
| Ctrl+L | Focus the address bar |
| Ctrl+H | Show or hide History |

History and settings are stored in your data directory under `mdbrowser/` (for example `~/.local/share/mdbrowser` on Linux). Set `MDBROWSER_DATA_DIR` to use another location.

## Develop

Run the gates in this order before calling a change done:

```sh
cargo test                                  # unit, golden layout and acceptance tests
cargo build
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

CI runs the same gates on every pull request. Acceptance scenarios live in [`features/`](features/).

## Documentation

| File | What it covers |
|---|---|
| [VECTOR.md](VECTOR.md) | Why the project exists, who it serves, and where it's going |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Layers, stack, import rules and conventions |
| [DESIGN.md](DESIGN.md) | The reader stylesheet: typefaces and tokens |
| [vector/decisions/](vector/decisions/) | Architecture decision records |

## Acknowledgements

- [The Web of Markdown](https://github.com/scripting/markdownWeb) by Dave Winer: the idea this browser serves, writers owning the words and readers owning the presentation, and the shared front matter vocabulary we aim to interoperate with.
- [agentic-dev-team](https://github.com/bdfinst/agentic-dev-team) by Bryan Finster: the agentic workhorse of this project, from planning through test-driven builds, review and pull requests.
- [Investiture](https://github.com/erikaflowers/investiture) and [Zero Vector](https://zerovector.design/investiture) by Allison Flowers: my assistant in building out the project doctrine, `VECTOR.md`, `ARCHITECTURE.md` and the `vector/` research and decision records.

## Licence

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option. See [LICENSE](LICENSE).

The bundled fonts are not covered by that licence. They are under the SIL Open Font License, and each licence sits beside its font in `assets/fonts/`.
