# mdbrowser

A browser for the [Web of Markdown](https://github.com/scripting/markdownWeb). It reads `.md` files, parses them into a document tree and draws that tree with its own layout engine. No HTML, no DOM, no webview.

Writers own the words. Readers own the presentation. The document chooses nothing about how it looks; the reader picks the typeface, and raw HTML in a document is never interpreted.

> **Status:** phase 2a, local files and remote Markdown over `http(s)`. Images (phase 2b), front matter navigation and more reader controls are next. See [VECTOR.md](VECTOR.md) § Direction.

## What it does today

- Opens local Markdown by path, `~/` path or `file://` URL, and follows links between `.md` files.
- Opens Markdown on the web by `http://` or `https://` address. It asks the site for Markdown first, so some sites serve it even at addresses without `.md`: try `https://andrewshell.org/`. Redirects are followed, and the address bar shows where the page ended up.
- Follows links between `.md` pages across sites, and from local files to the web. A web page can never open files on your computer, or addresses on your local network.
- A web page that turns out not to be Markdown can be handed to your web browser with **Open in browser**.
- Renders CommonMark plus GitHub tables, task lists and strikethrough. In-page `#anchors` use GitHub heading slugs.
- Takes the window title from front matter `title`, then the first heading, then the file name.
- Back, forward, reload, and a History panel that persists across restarts.
- Four bundled reading faces: Literata (default), Source Serif 4, iA Writer Quattro and Inter. Code is set in JetBrains Mono.
- Select text with the mouse and copy it with Ctrl+C.
- Other `http` and `https` links, and `mailto` links, open in your system's handler. Other schemes are blocked.
- Raw HTML in a document is stripped, never interpreted: the tags go and the text between them stays.

## Build from source

mdbrowser builds and passes its tests on Linux (x64 and ARM64), macOS and Windows. CI checks all four on every change.

### Prerequisites

- **Rust 1.88 or newer**, installed with [rustup](https://rustup.rs).
- **Linux:** a C toolchain (`build-essential` on Debian and Ubuntu, `base-devel` on Arch). It runs under Wayland or X11.
- **macOS:** the Xcode Command Line Tools (`xcode-select --install`).
- **Windows:** the Visual Studio C++ Build Tools. rustup offers to install them.

### Build and run

```sh
git clone https://github.com/gbrettmiller/mdbrowser-native.git
cd mdbrowser-native
cargo run --release -- corpus/index.md
```

The first build takes a few minutes. The app ends up at `target/release/mdbrowser` (`mdbrowser.exe` on Windows). Pass it a path to a `.md` file or a web address (`target/release/mdbrowser https://andrewshell.org/`), or start it with none and type one in the address bar.

### Keyboard

| Action | Linux and Windows | macOS |
|---|---|---|
| Back / forward | Alt+← / Alt+→ | Option+← / Option+→ |
| Reload | Ctrl+R or F5 | Cmd+R or F5 |
| Focus the address bar | Ctrl+L | Cmd+L |
| Show or hide History | Ctrl+H | The History button (Cmd+H hides the app on macOS) |
| Select all / copy | Ctrl+A / Ctrl+C | Cmd+A / Cmd+C |

### Where your data lives

History and settings are kept in an `mdbrowser` folder in your user data directory:

| OS | Folder |
|---|---|
| Linux | `~/.local/share/mdbrowser` (or `$XDG_DATA_HOME/mdbrowser`) |
| macOS | `~/Library/Application Support/mdbrowser` |
| Windows | `%APPDATA%\mdbrowser` |

To use another folder, set `MDBROWSER_DATA_DIR`: `MDBROWSER_DATA_DIR=/tmp/mdb cargo run` in a Unix shell, or `$env:MDBROWSER_DATA_DIR = "C:\temp\mdb"` in PowerShell before running.

## Develop

Run the gates in this order before calling a change done:

```sh
cargo test                                  # unit, golden layout and acceptance tests
cargo build
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

CI runs the same gates on every pull request: formatting once, then lint, tests and a release build on Linux x64, Linux ARM64, macOS and Windows. Acceptance scenarios live in [`features/`](features/).

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
