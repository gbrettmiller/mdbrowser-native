# mdbrowser

A browser for the [Web of Markdown](https://github.com/scripting/markdownWeb). It reads `.md` files, parses them into a document tree and draws that tree with its own layout engine. No HTML, no DOM, no webview.

Writers own the words. Readers own the presentation. The document chooses nothing about how it looks; the reader picks the typeface, and raw HTML in a document is never interpreted.

> **Status:** phase 5 (keyboard access), local files and remote Markdown over `http(s)`, with images, a Settings panel (font, text size, spacing, Web images, light, dark, system or custom colours, home page), a built-in welcome page, a Markdown fallback when a page is HTML (its `Link` alternate, then the `.md` sibling), and an offer of the site's `llms.txt`. Shared front matter (byline, previous and next, description, enclosure) is read. Discovery (`llms.txt` as a site index, feeds) is next. See [VECTOR.md](VECTOR.md) § Direction.

## What it does today

- Opens local Markdown by path, `~/` path or `file://` URL, and follows links between `.md` files.
- Opens Markdown on the web by `http://` or `https://` address. It asks the site for Markdown first, so some sites serve it even at addresses without `.md`: try `https://andrewshell.org/`. Redirects are followed, and the address bar shows where the page ended up.
- Follows links between `.md` pages across sites, and from local files to the web. A web page can never open files on your computer, or addresses on your local network.
- A web page that turns out not to be Markdown can be handed to your web browser with **Open in browser**.
- Renders CommonMark plus GitHub tables, task lists and strikethrough. In-page `#anchors` use GitHub heading slugs.
- Takes the window title from front matter `title`, then the first heading, then the file name.
- Back, forward, reload, and a History panel that persists across restarts.
- Four bundled reading faces: Literata (default), Source Serif 4, iA Writer Quattro and Inter. Code is set in JetBrains Mono.
- A **Settings** panel (Ctrl+, or Cmd+,) holds the reading font, text size (12 to 32 px in steps of 2), spacing (Compact, Comfortable or Spacious), Web images, Appearance (System, Light, Dark or Custom) and a Home page, with a button that puts size and spacing back. System follows the operating system. Custom takes page, text and link colours as `#rgb` or `#rrggbb`, and refuses a set whose text or links fall below WCAG AA contrast (4.5:1) on the page. Home (the toolbar button or Alt+Home) opens the home page, or a built-in welcome page when none is set, and the app opens it on launch unless you give an address.
- Reads the shared Web of Markdown front matter: a byline under the title from `author`, `pubDate` and the `source` site name, a `description`, a **Listen** line for an `enclosure` (it opens in your system's player), and **Previous** and **Next** links from `urlPrev` and `urlNext`. Other keys, and the `image`, `inlineImage` and `metaImage` fields, are ignored.
- Select text with the mouse and copy it with Ctrl+C.
- Other `http` and `https` links, and `mailto` links, open in your system's handler. Other schemes are blocked.
- Raw HTML in a document is stripped, never interpreted: the tags go and the text between them stays.
- Draws Markdown images (`![description](src)`) from files and the web: PNG, JPEG, GIF (its first frame) and WebP. An image is never shown larger than it is, and is scaled down to fit the text. An image that can't be shown leaves its description and a note saying why; one that failed to load can be clicked to try again.
- The **Web images** setting (in Settings, Ctrl+, or Cmd+,) chooses whether images from the web load automatically, on click, or not at all. Images in local files always load. Ctrl+Shift+I (Cmd+Shift+I on macOS) loads the current page's images whatever the setting.
- Loading web images tells the sites that host them that you are reading the post, as in any browser, though mdbrowser sends no cookies or referrer. Choose "on click" or "off" if you'd rather not.
- A web page can't show images from your disk, and a public page can't show images from your local network.

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

The first build takes a few minutes. The app ends up at `target/release/mdbrowser` (`mdbrowser.exe` on Windows). Pass it a path to a `.md` file or a web address (`target/release/mdbrowser https://andrewshell.org/`), or start it with none to open your home page (the built-in welcome page until you set one) and type an address in the address bar.

### Keyboard

| Action | Linux and Windows | macOS |
|---|---|---|
| Back / forward | Alt+← / Alt+→ | Option+← / Option+→ |
| Reload | Ctrl+R or F5 | Cmd+R or F5 |
| Focus the address bar | Ctrl+L | Cmd+L |
| Show or hide History | Ctrl+H | The History button (Cmd+H hides the app on macOS) |
| Show or hide Settings | Ctrl+, | Cmd+, |
| Previous / next post | Ctrl+[ / Ctrl+] | Cmd+[ / Cmd+] |
| Open the home page | Alt+Home | Option+Home |
| Open a page that isn't Markdown in your browser | Ctrl+Shift+O | Cmd+Shift+O |
| Load this page's images | Ctrl+Shift+I | Cmd+Shift+I |
| Larger / smaller / default text size | Ctrl+= / Ctrl+- / Ctrl+0 | Cmd+= / Cmd+- / Cmd+0 |
| Select all / copy | Ctrl+A / Ctrl+C | Cmd+A / Cmd+C |
| Move to the next / previous link, button or field | Tab / Shift+Tab | Tab / Shift+Tab |
| Follow the focused link, press the focused button | Enter (Space also presses a button) | Enter (Space also presses a button) |
| Change a focused choice in Settings (font, images, appearance, spacing) | Up / Down | Up / Down |
| Scroll a page at a time | PageDown / PageUp, or Space / Shift+Space when the page has focus | the same |
| Scroll a line, or to the top or bottom | Up / Down, Home / End, when the page has focus | the same |
| Close a panel and return to the page | Esc | Esc |

Opening a page puts the keyboard on it, so the scroll keys work at once. While you type in a text field, only PageUp and PageDown scroll the page; Tab or Esc takes you out of the field. A ring in the link colour shows where focus is. The window can't be made smaller than 620 by 360, and if History and Settings are both open and there isn't room for the page beside them, opening one closes the other. Screen readers are not supported yet: iced has no accessibility tree.

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
| [DESIGN.md](DESIGN.md) | The reader stylesheet: typefaces, tokens, size, spacing and palettes |
| [vector/decisions/](vector/decisions/) | Architecture decision records |

## Acknowledgements

- [The Web of Markdown](https://github.com/scripting/markdownWeb) by Dave Winer: the idea this browser serves, writers owning the words and readers owning the presentation, and the shared front matter vocabulary we aim to interoperate with.
- [agentic-dev-team](https://github.com/bdfinst/agentic-dev-team) by Bryan Finster: the agentic workhorse of this project, from planning through test-driven builds, review and pull requests.
- [Investiture](https://github.com/erikaflowers/investiture) and [Zero Vector](https://zerovector.design/investiture) by Allison Flowers: my assistant in building out the project doctrine, `VECTOR.md`, `ARCHITECTURE.md` and the `vector/` research and decision records.

## Licence

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option. See [LICENSE](LICENSE).

The bundled fonts are not covered by that licence. They are under the SIL Open Font License, and each licence sits beside its font in `assets/fonts/`.
