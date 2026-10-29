# Tables and code

Back to [the index](index.md).

## Table

| Stage  | Web browser       | Markdown browser                       |
|:-------|:-----------------:|---------------------------------------:|
| Fetch  | HTTP, `text/html` | `file://`, `text/markdown`             |
| Model  | DOM               | CommonMark tree                        |
| Style  | Author CSS        | **Reader only**                        |
| Layout | CSS engine        | Block flow with wrapped inline runs, which is a much smaller problem |

## Code block

```rust
fn layout(blocks: &[Block], theme: &Theme, width: f32) -> Page {
    let content = (width - 2.0 * theme.gutter).min(theme.measure);
    // A very long line that will need to wrap inside the code block because it is wider than the measure of the page
    flow.blocks(blocks, x, content, ink, 0);
}
```

    Indented code block
    with two lines

<div>
  An HTML block is stripped to its text.
</div>
