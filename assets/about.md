# About mdBrowser

mdBrowser is a browser for the web of Markdown. It reads Markdown from files and web addresses and draws it itself, with no HTML, no DOM and no web view. You choose how it looks: documents bring the words, and you bring the font, size, spacing and colours.

- [What it sends and stores](#what-it-sends-and-stores)
- [Keyboard shortcuts](#keyboard-shortcuts)
- [FAQ](#faq)

## Version

mdBrowser {version}, built for {build}.

Licensed under {licence}, at your option. The source is at [github.com/gbrettmiller/mdbrowser-native](https://github.com/gbrettmiller/mdbrowser-native).

## Licence and credits

The reading fonts are bundled in the app. Each is licensed under the SIL Open Font License 1.1, and the full texts, with those of the libraries the app is built from, ship with each release in THIRD-PARTY-NOTICES.html.

| Font | Used for | Licence |
|---|---|---|
| Literata | Reading font | SIL Open Font License 1.1 |
| Source Serif 4 | Reading font | SIL Open Font License 1.1 |
| iA Writer Quattro S | Reading font | SIL Open Font License 1.1 |
| Inter | Reading font | SIL Open Font License 1.1 |
| JetBrains Mono | Code | SIL Open Font License 1.1 |

## What it sends and stores

- **On the web:** it fetches only addresses you open or follow, and web images when the Images setting allows. Each request carries a type preference (`Accept`) and a `User-Agent` that names the app and its version. It sends no cookies, no referrer and no account. If your system sets a proxy, the app uses it.
- **Mail links:** `mailto:` links open your mail program, and only after you agree.
- **Nothing else:** there is no analytics and no update check. It makes no request you did not cause.
- **On this computer:** your History and settings are saved in an `mdbrowser` folder in your user data directory. The README says where that is on each system. Clear History in the History panel.

## Keyboard shortcuts

Use Cmd instead of Ctrl on macOS.

| Action | Shortcut |
|---|---|
| Show this page | F1 or Ctrl+? |
| Show or hide Settings | Ctrl+, |
| Show or hide History | Ctrl+H |
| Focus the address bar | Ctrl+L |
| Show a page's Markdown source, and back | Ctrl+U |
| Open the home page | Alt+Home |
| Larger, smaller, default text size | Ctrl+=, Ctrl+-, Ctrl+0 |
| Next or previous link, button or field | Tab or Shift+Tab |
| Close a panel | Esc |

## FAQ

### Why does it only show Markdown?

That is the idea. A page that is not Markdown is looked up in Markdown form (its `Link` alternate, or a `.md` sibling), and when there is none the app says so and offers to open it in your browser.

### A page looks different from its website. Why?

The website's styling is its own HTML and CSS. mdBrowser draws the Markdown the writer wrote, in your look. Press Ctrl+U to see exactly what the writer wrote.

### Why are some images smaller than they are?

An image is scaled down to fit the column and a height that follows your text size. It keeps its shape and is never cropped or enlarged.

### Where are my settings and History?

In an `mdbrowser` folder in your user data directory (see the README). Deleting that folder resets both.

### How do I change what the app looks like?

Open Settings: the reading font, text size, spacing, Web images, and a light, dark, system or custom palette.

## Report a problem

Open an issue at [github.com/gbrettmiller/mdbrowser-native/issues](https://github.com/gbrettmiller/mdbrowser-native/issues) and include the version and platform shown above.
