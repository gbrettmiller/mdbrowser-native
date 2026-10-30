# Examples to check

Every page in the corpus, by what it shows. Open a page, look for the thing described, and come back with Back. Open this page with `cargo run --release -- corpus/examples.md`.

## The reader

- [Typography and inline styles](typography.md): emphasis, code, links, mixed scripts, and the four reading fonts (change them in Settings).
- [Lists, quotes and task lists](lists.md): nesting, numbering, checkboxes.
- [Tables and code](tables-and-code.md): alignment, wide tables, long code lines.
- [Images](images.md): images alone, in links, at the start of a paragraph, missing and slow ones. Try the Web images setting.
- [A long document](long.md): scroll smoothness and resizing the window.
- [The CommonMark spec](spec.md): a 200 KB stress test, and anchors (the contents at the top).

## Shared front matter (phase 3)

What to look for is on each page.

- [A byline](front-matter/byline.md): author · date · site name, then the description, in the muted colour.
- [A repeated description](front-matter/description-repeated.md): the description appears once.
- [A date we can't read](front-matter/date-as-written.md): shown as written.
- [Wrong kinds of value](front-matter/wrong-kinds.md): bad values dropped one by one, no image drawn.
- [An enclosure](front-matter/enclosure.md): a Listen line that opens in the system.
- [An enclosure we refuse](front-matter/bad-enclosure.md): no Listen line.
- [A three-post blog](front-matter/blog/one.md): Previous and Next links, Ctrl+[ and Ctrl+], and a Next that isn't there.

## Front matter edge cases

- [Title over heading](front-matter/title-over-heading.md): the window title comes from `title`.
- [Malformed YAML](front-matter/malformed.md): the block is hidden, and the page still renders.
- [Unknown keys](front-matter/unknown-keys.md): ignored.
- [Empty block](front-matter/empty.md): `---` straight after `---` is still front matter.
- [Not at the top](front-matter/not-at-top.md): later `---` blocks are rules, never front matter.

## Real posts from the Web of Markdown

Dave Winer's Scripting News, with the front matter each one shows. The full table and the key counts are in the [examples readme](markdownweb/README.md).

| Example | What to look for |
|---|---|
| 0 · [2024/03/31/a164241](markdownweb/2024/03/31/a164241.md) | The base set: byline with no title, Previous and Next. |
| 1 · [2024/03/05/200550](markdownweb/2024/03/05/200550.md) | `title`, and a `metaImage` that is not drawn. |
| 2 · [2024/03/31/a161730](markdownweb/2024/03/31/a161730.md) | `image`: the picture in the body is the only one. It stacks above the text (the open question). |
| 3 · [2024/03/22/025201](markdownweb/2024/03/22/025201.md) | Two inline images. |
| 4 · [2024/03/15/015719](markdownweb/2024/03/15/015719.md) | A numbered list. |
| 5 · [2024/01/21/a194147](markdownweb/2024/01/21/a194147.md) | A podcast: the Listen line. |
| 6 · [2024/03/01/203935](markdownweb/2024/03/01/203935.md) | Previous reaching into the month before (not in the corpus, so "not found"). |
| 7 · [2024/03/12/120111](markdownweb/2024/03/12/120111.md) | `description` and a bulleted list. |
| 8 · [2026/10/22/a160031](markdownweb/2026/10/22/a160031.md) | The newest post: Previous only. |
| · | [2022/05/21/a020940](markdownweb/2022/05/21/a020940.md) has `inlineImage`. [2022/03/05/a163449](markdownweb/2022/03/05/a163449.md) has `urlvideo` and [2024/03/17/a194939](markdownweb/2024/03/17/a194939.md) has `urltoot`: both ignored. |

Posts link to each other by relative path, so Next from example 0 opens [a161730](markdownweb/2024/03/31/a161730.md) (example 2).

## Errors and the system

- [A missing page](nowhere.md): the "not found" screen, with the toolbar still usable.
- [An external link](https://commonmark.org): handed to your system's browser.
- [A web page that isn't Markdown](https://example.com): the "not Markdown" screen, with Open in browser.

## Back to the start

[The corpus home page](index.md).
