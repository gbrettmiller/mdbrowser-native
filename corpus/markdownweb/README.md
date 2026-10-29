# Web of Markdown examples

Real posts from Dave Winer's [Scripting News](http://scripting.com/), copied unchanged from the Markdown archive in [scripting/markdownWeb](https://github.com/scripting/markdownWeb/tree/main/examples). They keep the archive's folder layout (`year/month/day/post.md`), so relative links between them work. For example, `urlNext` in [2024/03/31/a164241.md](2024/03/31/a164241.md) points to [a161730.md](2024/03/31/a161730.md). Most `urlPrev` and `urlNext` targets aren't included and will report a missing page.

## Posts

| Example | Post | Front matter it shows |
|---|---|---|
| 0 | [2024/03/31/a164241](2024/03/31/a164241.md) | Untitled; the base set of `author`, `pubDate`, `source`, `generator`, `urlPrev` and `urlNext` |
| 1 | [2024/03/05/200550](2024/03/05/200550.md) | `title`, `metaImage` |
| 2 | [2024/03/31/a161730](2024/03/31/a161730.md) | `image` (a right-margin image) |
| 3 | [2024/03/22/025201](2024/03/22/025201.md) | `title`, `metaImage`, two inline images |
| 4 | [2024/03/15/015719](2024/03/15/015719.md) | `title`, a numbered list |
| 5 | [2024/01/21/a194147](2024/01/21/a194147.md) | `enclosure`, `enclosureType`, `enclosureLength` (a podcast) |
| 6 | [2024/03/01/203935](2024/03/01/203935.md) | `urlPrev` reaching back into the previous month |
| 7 | [2024/03/12/120111](2024/03/12/120111.md) | `description`, a bulleted list |
| 8 | [2026/10/22/a160031](2026/10/22/a160031.md) | `urlPrev` with no `urlNext` (the newest post) |
| — | [2022/05/21/a020940](2022/05/21/a020940.md) | `inlineImage` (not in the numbered examples) |
| — | [2022/03/05/a163449](2022/03/05/a163449.md) | `urlvideo` (used in the archive, not in the readme's vocabulary) |
| — | [2024/03/17/a194939](2024/03/17/a194939.md) | `urltoot` (a Mastodon cross-post) |

Examples 0–8 match the screenshots on the markdownWeb examples page.

## Keys across the whole archive

Counts from the 16,869 `.md` files in `archive.zip`, surveyed 2026-10-28. 114 files, the monthly `links.md` pages, have no front matter.

| Key | Files |
|---|---|
| `author`, `pubDate`, `source`, `generator` | 16,755 |
| `urlPrev` / `urlNext` | 16,754 / 16,753 |
| `title` | 2,899 |
| `image` | 2,628 |
| `description` | 713 |
| `urlvideo` | 536 |
| `metaImage` | 450 |
| `enclosure` (+ `enclosureType`, `enclosureLength`) | 193 |
| `inlineImage` | 159 |
| `urltoot` | 9 |

## Licence

These files are from scripting/markdownWeb, used under its MIT License:

> MIT License
>
> Copyright (c) 2026 Dave Winer
>
> Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
>
> THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
