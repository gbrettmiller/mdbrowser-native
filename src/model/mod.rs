//! Markdown source → owned document tree. This tree *is* the document model;
//! nothing downstream ever sees HTML.

mod blocks;
mod frontmatter;
mod inlines;
pub mod slug;

use pulldown_cmark::{Event, MetadataBlockKind, Options, Parser, Tag};

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
    Heading { level: u8, spans: Vec<Span>, slug: String },
    Paragraph(Vec<Span>),
    Code { lang: Option<String>, text: String },
    Quote(Vec<Block>),
    List { start: Option<u64>, items: Vec<Item> },
    Table { aligns: Vec<Align>, rows: Vec<Vec<Vec<Span>>> },
    Rule,
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
        })
        .collect()
}

fn frontmatter_source(events: &[Event]) -> Option<String> {
    events.windows(2).find_map(|pair| match pair {
        [Event::Start(Tag::MetadataBlock(MetadataBlockKind::YamlStyle)), Event::Text(yaml)] => Some(yaml.to_string()),
        _ => None,
    })
}

pub fn parse(source: &str) -> Document {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS;
    let events: Vec<Event> = Parser::new_ext(source, options).collect();
    let frontmatter_title = frontmatter_source(&events).and_then(|yaml| frontmatter::title(&yaml));
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
    fn html_is_shown_as_literal_text_not_interpreted() {
        assert_eq!(paragraph_text("x <b>y</b>"), "x <b>y</b>");
        assert_eq!(
            blocks("<script>alert(1)</script>"),
            vec![Block::Code { lang: None, text: "<script>alert(1)</script>".into() }]
        );
    }

    #[test]
    fn images_show_alt_text() {
        assert_eq!(paragraph_text("![a cat](cat.png)"), "[image: a cat]");
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
