//! Markdown source → owned document tree. This tree *is* the document model;
//! nothing downstream ever sees HTML.

mod blocks;
mod frontmatter;
mod html;
mod inlines;
pub mod slug;

#[cfg(test)]
mod corpus;

use pulldown_cmark::{Event, Options, Parser};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Style {
    pub strong: bool,
    pub emphasis: bool,
    pub strike: bool,
    pub code: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: Style,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub checked: Option<bool>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Block {
    Heading {
        level: u8,
        spans: Vec<Span>,
        slug: String,
    },
    Paragraph(Vec<Span>),
    Code {
        lang: Option<String>,
        text: String,
    },
    Quote(Vec<Block>),
    List {
        start: Option<u64>,
        items: Vec<Item>,
    },
    Table {
        aligns: Vec<Align>,
        rows: Vec<Vec<Vec<Span>>>,
    },
    Rule,
    /// A Markdown image, split out of its paragraph. `link` is the enclosing link, if any.
    Image {
        src: String,
        alt: String,
        link: Option<String>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    pub blocks: Vec<Block>,
    pub frontmatter_title: Option<String>,
}

pub fn text_of(spans: &[Span]) -> String {
    spans.iter().map(|span| span.text.as_str()).collect()
}

impl Document {
    /// All visible text, blocks separated by newlines (for tests and search).
    /// Every image block's source, in reading order, however deeply nested.
    pub fn image_sources(&self) -> Vec<String> {
        fn collect(blocks: &[Block], out: &mut Vec<String>) {
            for block in blocks {
                match block {
                    Block::Image { src, .. } => out.push(src.clone()),
                    Block::Quote(inner) => collect(inner, out),
                    Block::List { items, .. } => items.iter().for_each(|item| collect(&item.blocks, out)),
                    _ => {}
                }
            }
        }
        let mut sources = Vec::new();
        collect(&self.blocks, &mut sources);
        sources
    }

    pub fn plain_text(&self) -> String {
        blocks_text(&self.blocks).join("\n")
    }

    /// Frontmatter `title`, else the first heading, else the caller's fallback (the file name).
    pub fn title(&self, fallback: &str) -> String {
        let first_heading = || {
            self.blocks.iter().find_map(|block| match block {
                Block::Heading { spans, .. } => Some(text_of(spans)),
                _ => None,
            })
        };
        self.frontmatter_title.clone().or_else(first_heading).unwrap_or_else(|| fallback.to_string())
    }
}

/// What stands in for an image in text: `[image: alt]`, or `[image]` with no alt.
pub fn image_description(alt: &str) -> String {
    if alt.is_empty() { "[image]".to_string() } else { format!("[image: {alt}]") }
}

fn blocks_text(blocks: &[Block]) -> Vec<String> {
    blocks
        .iter()
        .flat_map(|block| match block {
            Block::Heading { spans, .. } | Block::Paragraph(spans) => vec![text_of(spans)],
            Block::Code { text, .. } => vec![text.trim_end_matches('\n').to_string()],
            Block::Quote(inner) => blocks_text(inner),
            Block::List { items, .. } => items.iter().flat_map(|item| blocks_text(&item.blocks)).collect(),
            Block::Table { rows, .. } => {
                rows.iter().map(|row| row.iter().map(|cell| text_of(cell)).collect::<Vec<_>>().join(" ")).collect()
            }
            Block::Rule => vec![],
            Block::Image { alt, .. } => vec![image_description(alt)],
        })
        .collect()
}

pub fn parse(source: &str) -> Document {
    let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let (yaml, body) = frontmatter::split(source);
    let events: Vec<Event> = Parser::new_ext(body, options).collect();
    let frontmatter_title = yaml.and_then(frontmatter::title);
    let blocks = blocks::blocks(&mut blocks::Events::new(events));
    Document { blocks, frontmatter_title }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(text: &str) -> Span {
        Span { text: text.into(), style: Style::default(), link: None }
    }

    fn heading(level: u8, text: &str, slug: &str) -> Block {
        Block::Heading { level, spans: vec![plain(text)], slug: slug.into() }
    }

    fn blocks(source: &str) -> Vec<Block> {
        parse(source).blocks
    }

    fn paragraph_text(source: &str) -> String {
        match &blocks(source)[0] {
            Block::Paragraph(spans) => text_of(spans),
            other => panic!("expected a paragraph, got {other:?}"),
        }
    }

    #[test]
    fn heading_and_paragraph() {
        assert_eq!(
            blocks("# Title\n\nHello"),
            vec![heading(1, "Title", "title"), Block::Paragraph(vec![plain("Hello")])]
        );
    }

    #[test]
    fn inline_styles_and_links() {
        let Block::Paragraph(spans) = &blocks("a **b *c*** `d` [e](x.md)")[0] else { panic!() };
        let styled: Vec<_> = spans.iter().map(|s| (s.text.as_str(), s.style, s.link.as_deref())).collect();
        let strong = Style { strong: true, ..Style::default() };
        let strong_em = Style { strong: true, emphasis: true, ..Style::default() };
        let code = Style { code: true, ..Style::default() };
        assert_eq!(
            styled,
            vec![
                ("a ", Style::default(), None),
                ("b ", strong, None),
                ("c", strong_em, None),
                (" ", Style::default(), None),
                ("d", code, None),
                (" ", Style::default(), None),
                ("e", Style::default(), Some("x.md")),
            ]
        );
    }

    #[test]
    fn soft_break_is_space_hard_break_is_newline() {
        assert_eq!(paragraph_text("a\nb  \nc"), "a b\nc");
    }

    #[test]
    fn tight_list_items_become_paragraphs() {
        assert_eq!(
            blocks("- one\n- [x] two"),
            vec![Block::List {
                start: None,
                items: vec![
                    Item { checked: None, blocks: vec![Block::Paragraph(vec![plain("one")])] },
                    Item { checked: Some(true), blocks: vec![Block::Paragraph(vec![plain("two")])] },
                ],
            }]
        );
    }

    #[test]
    fn nested_quote_list_and_code() {
        assert_eq!(
            blocks("> 3. x\n>    - y\n\n```rs\nfn a() {}\n```\n\n---"),
            vec![
                Block::Quote(vec![Block::List {
                    start: Some(3),
                    items: vec![Item {
                        checked: None,
                        blocks: vec![
                            Block::Paragraph(vec![plain("x")]),
                            Block::List {
                                start: None,
                                items: vec![Item { checked: None, blocks: vec![Block::Paragraph(vec![plain("y")])] }],
                            },
                        ],
                    }],
                }]),
                Block::Code { lang: Some("rs".into()), text: "fn a() {}\n".into() },
                Block::Rule,
            ]
        );
    }

    #[test]
    fn table_rows_include_header_first() {
        assert_eq!(
            blocks("| a | b |\n|:-|-:|\n| 1 | 2 |"),
            vec![Block::Table {
                aligns: vec![Align::Left, Align::Right],
                rows: vec![vec![vec![plain("a")], vec![plain("b")]], vec![vec![plain("1")], vec![plain("2")]]],
            }]
        );
    }

    #[test]
    fn inline_html_is_stripped_and_its_text_kept() {
        assert_eq!(paragraph_text("x <b>y</b>"), "x y");
        assert_eq!(paragraph_text("[<i>x</i>](u)"), "x");
        assert_eq!(paragraph_text("a <a title='p>q'>b</a>"), "a b");
    }

    #[test]
    fn html_blocks_become_their_text_or_nothing() {
        assert_eq!(blocks("<script>alert(1)</script>"), vec![Block::Paragraph(vec![plain("alert(1)")])]);
        assert_eq!(blocks("<div>\nhello\nthere\n</div>"), vec![Block::Paragraph(vec![plain("hello there")])]);
        assert_eq!(blocks("<!-- note -->\n\npara"), vec![Block::Paragraph(vec![plain("para")])]);
        assert_eq!(blocks("<div></div>"), vec![]);
        // An unclosed comment's HTML block runs to the end of the document, so it hides what follows.
        assert_eq!(blocks("a\n\n<!-- unclosed\n\nb"), vec![Block::Paragraph(vec![plain("a")])]);
    }

    #[test]
    fn html_blocks_inside_quotes_and_lists_are_stripped() {
        assert_eq!(parse("> <div>q</div>").plain_text(), "q");
        assert_eq!(parse("- <div>item</div>").plain_text(), "item");
    }

    #[test]
    fn html_in_headings_lists_and_tables_is_stripped() {
        let doc = parse("# <b>Title</b>\n\n- <i>item</i>\n\n| <u>cell</u> |\n|-|\n| y |");
        assert_eq!(doc.blocks[0], heading(1, "Title", "title"));
        assert_eq!(doc.title("t.md"), "Title");
        assert_eq!(doc.plain_text(), "Title\nitem\ncell\ny");
    }

    #[test]
    fn code_is_never_stripped() {
        assert_eq!(paragraph_text("`<b>code</b>` stays"), "<b>code</b> stays");
        assert_eq!(blocks("    <b>indented</b>\n"), vec![Block::Code { lang: None, text: "<b>indented</b>\n".into() }]);
        assert_eq!(blocks("```\n<b>x</b>\n```"), vec![Block::Code { lang: None, text: "<b>x</b>\n".into() }]);
    }

    fn image(src: &str, alt: &str) -> Block {
        Block::Image { src: src.into(), alt: alt.into(), link: None }
    }

    fn para(text: &str) -> Block {
        Block::Paragraph(vec![plain(text)])
    }

    #[test]
    fn an_image_alone_is_an_image_block() {
        assert_eq!(blocks("![a cat](cat.png)"), [image("cat.png", "a cat")]);
        assert_eq!(blocks("![a]()"), [image("", "a")]);
    }

    /// Phase 1 skipped the event after an image, losing the text that followed it.
    #[test]
    fn text_after_an_image_is_kept() {
        assert_eq!(parse("![x](y.png)text\n\nnext").plain_text(), "[image: x]\ntext\nnext");
    }

    #[test]
    fn paragraphs_split_around_their_images() {
        assert_eq!(blocks("![](c.png)I gotta say"), [image("c.png", ""), para("I gotta say")]);
        assert_eq!(blocks("Before ![b](y.png) after"), [para("Before"), image("y.png", "b"), para("after")]);
        assert_eq!(blocks("Text ![t](t.png)"), [para("Text"), image("t.png", "t")]);
        assert_eq!(blocks("![a](a.png)![b](b.png)"), [image("a.png", "a"), image("b.png", "b")]);
        assert_eq!(blocks("![a](a.png) ![b](b.png)"), [image("a.png", "a"), image("b.png", "b")]);
    }

    #[test]
    fn whitespace_at_a_cut_is_trimmed_and_breaks_inside_stay() {
        assert_eq!(blocks("a\nb ![i](i.png)\nc"), [para("a b"), image("i.png", "i"), para("c")]);
        assert_eq!(blocks("a  \nb ![i](i.png)"), [para("a\nb"), image("i.png", "i")]);
    }

    #[test]
    fn paragraphs_without_images_keep_their_whitespace() {
        assert_eq!(paragraph_text("`  a  `"), " a ");
    }

    #[test]
    fn emphasis_and_links_carry_across_a_cut() {
        let strong = Style { strong: true, ..Style::default() };
        assert_eq!(
            blocks("**a ![i](i.png) b**"),
            [
                Block::Paragraph(vec![Span { text: "a".into(), style: strong, link: None }]),
                image("i.png", "i"),
                Block::Paragraph(vec![Span { text: "b".into(), style: strong, link: None }]),
            ]
        );
        let link = Some("https://x.org/".to_string());
        assert_eq!(
            blocks("[see ![a](b.png)](https://x.org/)"),
            [
                Block::Paragraph(vec![Span { text: "see".into(), style: Style::default(), link: link.clone() }]),
                Block::Image { src: "b.png".into(), alt: "a".into(), link },
            ]
        );
    }

    #[test]
    fn images_split_inside_lists_and_quotes() {
        assert_eq!(blocks("> x ![q](q.png)"), [Block::Quote(vec![para("x"), image("q.png", "q")])]);
        assert_eq!(
            blocks("- ![l](l.png) y"),
            [Block::List {
                start: None,
                items: vec![Item { checked: None, blocks: vec![image("l.png", "l"), para("y")] }]
            }]
        );
    }

    #[test]
    fn images_in_headings_and_table_cells_stay_descriptions() {
        let emph = Style { emphasis: true, ..Style::default() };
        assert_eq!(
            blocks("# Title ![logo](l.png)"),
            [Block::Heading {
                level: 1,
                spans: vec![plain("Title "), Span { text: "[image: logo]".into(), style: emph, link: None }],
                slug: "title-image-logo".into(),
            }]
        );
        let doc = parse("| ![c](c.png) |\n|-|\n| ![](d.png) |");
        assert_eq!(doc.plain_text(), "[image: c]\n[image]");
    }

    #[test]
    fn images_in_code_stay_literal() {
        assert_eq!(paragraph_text("`![a](b.png)`"), "![a](b.png)");
        assert_eq!(blocks("    ![a](b.png)\n"), [Block::Code { lang: None, text: "![a](b.png)\n".into() }]);
    }

    #[test]
    fn an_image_right_after_a_heading_is_its_own_block() {
        assert_eq!(blocks("# H\n![a](a.png)"), [heading(1, "H", "h"), image("a.png", "a")]);
    }

    #[test]
    fn image_sources_are_listed_in_reading_order_however_deep() {
        let doc = parse("![a](1.png)\n\n> ![b](2.png)\n\n- x ![c](3.png)\n\n# ![d](h.png)");
        assert_eq!(doc.image_sources(), ["1.png", "2.png", "3.png"]);
    }

    #[test]
    fn plain_text_writes_images_as_descriptions() {
        assert_eq!(
            parse("Before ![a box](p.png) after\n\n![](q.png)").plain_text(),
            "Before\n[image: a box]\nafter\n[image]"
        );
    }

    #[test]
    fn plain_text_covers_nested_blocks() {
        let doc = parse("# T\n\n> - *a*\n\n```\n<b>c</b>\n```\n\n| x |\n|-|\n| y |");
        assert_eq!(doc.plain_text(), "T\na\n<b>c</b>\nx\ny");
    }

    #[test]
    fn frontmatter_is_not_content_and_titles_the_document() {
        let doc = parse("---\ntitle: Home\n---\n# Welcome");
        assert_eq!(doc.blocks, vec![heading(1, "Welcome", "welcome")]);
        assert_eq!(doc.title("index.md"), "Home");
    }

    #[test]
    fn title_falls_back_to_first_heading_then_file_name() {
        assert_eq!(parse("# Heading").title("t.md"), "Heading");
        assert_eq!(parse("intro\n\n## The `x` api").title("t.md"), "The x api");
        assert_eq!(parse("plain text").title("t.md"), "t.md");
        assert_eq!(parse("---\ntitle: [bad\n---\n# Fallback").title("t.md"), "Fallback");
    }

    #[test]
    fn headings_get_unique_slugs_document_wide() {
        let doc = parse("# Intro\n\n## Intro\n\n> # Quoted *head*");
        let Block::Quote(inner) = &doc.blocks[2] else { panic!() };
        assert_eq!(&doc.blocks[..2], &[heading(1, "Intro", "intro"), heading(2, "Intro", "intro-1")]);
        assert!(matches!(&inner[0], Block::Heading { slug, .. } if slug == "quoted-head"));
    }
}
