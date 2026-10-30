//! What a post's front matter adds around its body: a byline, a description, an enclosure line
//! and a previous/next row (ADR-006). All of it is ordinary blocks, so the reader's typeface and
//! palette govern it, and nothing is added when the front matter has nothing to show.

use super::date::readable;
use super::{Block, Document, Span, Style, text_of};

/// The few words the browser itself adds. They live in `content`, which the model can't see.
#[derive(Debug, Clone, Copy)]
pub struct Labels<'a> {
    pub listen: &'a str,
    pub previous: &'a str,
    pub next: &'a str,
}

/// What a document's front matter has to show, as plain data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Frame {
    pub byline: Option<String>,
    pub description: Option<String>,
    pub enclosure: Option<EnclosureLine>,
    /// `urlPrev` and `urlNext`, as written: relative to the page they came from.
    pub previous: Option<String>,
    pub next: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnclosureLine {
    pub url: String,
    /// The file's name and what is known of it: `show.m4a (audio/mpeg, 6.1 MB)`.
    pub label: String,
}

pub fn frame(document: &Document) -> Frame {
    let front = &document.front;
    let date = front.pub_date.as_deref().map(|date| readable(date).unwrap_or_else(|| date.to_string()));
    let site = front.source.as_ref().and_then(|source| source.title.clone());
    let byline = [front.author.clone(), date, site].into_iter().flatten().collect::<Vec<_>>().join(" · ");
    Frame {
        byline: (!byline.is_empty()).then_some(byline),
        description: front.description.clone().filter(|text| !opens_with(document, text)),
        enclosure: front.enclosure.as_ref().map(|enclosure| EnclosureLine {
            url: enclosure.url.clone(),
            label: enclosure_label(&enclosure.url, enclosure.mime.as_deref(), enclosure.length),
        }),
        previous: front.url_prev.clone(),
        next: front.url_next.clone(),
    }
}

/// The post already says it: one of its first two blocks is the same text.
fn opens_with(document: &Document, text: &str) -> bool {
    document
        .blocks
        .iter()
        .take(2)
        .any(|block| matches!(block, Block::Paragraph(spans) if text_of(spans).trim() == text))
}

fn enclosure_label(url: &str, mime: Option<&str>, length: Option<u64>) -> String {
    let address = url.split(['?', '#']).next().unwrap_or(url);
    let address = address.split_once("://").map_or(address, |(_, rest)| rest);
    let (host, path) = address.split_once('/').unwrap_or((address, ""));
    let name = path.rsplit('/').find(|segment| !segment.is_empty()).unwrap_or(host);
    let details = [mime.map(str::to_string), length.map(size)].into_iter().flatten().collect::<Vec<_>>().join(", ");
    if details.is_empty() { name.to_string() } else { format!("{name} ({details})") }
}

fn size(bytes: u64) -> String {
    const MEGABYTE: u64 = 1024 * 1024;
    if bytes >= MEGABYTE {
        format!("{:.1} MB", bytes as f64 / MEGABYTE as f64)
    } else {
        format!("{} KB", bytes.div_ceil(1024))
    }
}

/// The document's blocks with its front matter around them: the byline, description and
/// enclosure under a leading heading (or first, with none), and the previous/next row last.
pub fn assemble(document: &Document, labels: &Labels) -> Vec<Block> {
    let frame = frame(document);
    let after = usize::from(matches!(document.blocks.first(), Some(Block::Heading { .. })));
    let quiet = |text: &String| Block::Paragraph(vec![span(text, Style { quiet: true, ..Style::default() }, None)]);
    let mut head: Vec<Block> = frame.byline.iter().chain(&frame.description).map(quiet).collect();
    if let Some(line) = &frame.enclosure {
        let listen = format!("{} ", labels.listen);
        let link = span(&line.label, Style::default(), Some(line.url.clone()));
        head.push(Block::Paragraph(vec![span(&listen, Style::default(), None), link]));
    }
    let link = |label: &str, href: &Option<String>| {
        href.as_ref().map(|href| span(label, Style::default(), Some(href.clone())))
    };
    let row = match (link(labels.previous, &frame.previous), link(labels.next, &frame.next)) {
        (Some(previous), Some(next)) => vec![previous, span(" · ", Style::default(), None), next],
        (one, other) => one.into_iter().chain(other).collect(),
    };
    let foot = if row.is_empty() { vec![] } else { vec![Block::Rule, Block::Paragraph(row)] };
    [&document.blocks[..after], &head, &document.blocks[after..], &foot].concat()
}

fn span(text: &str, style: Style, link: Option<String>) -> Span {
    Span { text: text.to_string(), style, link }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Block, parse, text_of};

    const LABELS: Labels<'static> = Labels { listen: "Listen:", previous: "← Previous", next: "Next →" };

    fn post(front: &str, body: &str) -> crate::model::Document {
        parse(&format!("---\n{front}\n---\n{body}"))
    }

    fn texts(blocks: &[Block]) -> Vec<String> {
        blocks
            .iter()
            .map(|block| match block {
                Block::Paragraph(spans) | Block::Heading { spans, .. } => text_of(spans),
                Block::Rule => "---".into(),
                other => format!("{other:?}"),
            })
            .collect()
    }

    #[test]
    fn the_byline_joins_what_is_there() {
        let all =
            post("author: Dave Winer\npubDate: Sun, 31 Mar 2024 16:17:30 GMT\nsource: {title: Scripting News}", "Text");
        assert_eq!(frame(&all).byline.as_deref(), Some("Dave Winer · 31 March 2024 · Scripting News"));
        assert_eq!(frame(&post("author: Dave Winer", "Text")).byline.as_deref(), Some("Dave Winer"));
        assert_eq!(frame(&post("pubDate: 1 Jan 2023", "Text")).byline.as_deref(), Some("1 January 2023"));
        assert_eq!(frame(&post("source: {title: S}", "Text")).byline.as_deref(), Some("S"));
    }

    #[test]
    fn a_date_that_is_not_rfc_822_is_shown_as_written() {
        let doc = post("author: A\npubDate: last Tuesday-ish", "Text");
        assert_eq!(frame(&doc).byline.as_deref(), Some("A · last Tuesday-ish"));
    }

    #[test]
    fn no_author_date_or_site_means_no_byline() {
        assert_eq!(frame(&post("title: Plain", "Text")).byline, None);
        assert_eq!(frame(&parse("no front matter")), Frame::default());
    }

    #[test]
    fn a_description_the_post_opens_with_is_not_repeated() {
        assert_eq!(frame(&post("description: A summary", "Text")).description.as_deref(), Some("A summary"));
        assert_eq!(frame(&post("description: Text", "Text")).description, None);
        assert_eq!(frame(&post("description: Text", "# Title\n\n  Text  ")).description, None);
        assert_eq!(frame(&post("description: Text", "# Title\n\nOther")).description.as_deref(), Some("Text"));
    }

    #[test]
    fn the_enclosure_line_names_the_file_and_what_is_known_of_it() {
        let line = |front: &str| frame(&post(front, "x")).enclosure.map(|e| (e.url, e.label));
        assert_eq!(
            line("enclosure: http://e.com/a/show.m4a\nenclosureType: audio/mpeg\nenclosureLength: 6374173"),
            Some(("http://e.com/a/show.m4a".into(), "show.m4a (audio/mpeg, 6.1 MB)".into()))
        );
        assert_eq!(line("enclosure: http://e.com/show.m4a").map(|e| e.1), Some("show.m4a".into()));
        assert_eq!(
            line("enclosure: http://e.com/s.mp3?x=1#t\nenclosureType: audio/mpeg").map(|e| e.1),
            Some("s.mp3 (audio/mpeg)".into())
        );
        assert_eq!(line("enclosure: https://e.com/\nenclosureLength: 2048").map(|e| e.1), Some("e.com (2 KB)".into()));
    }

    #[test]
    fn previous_and_next_are_kept_as_written() {
        let found = frame(&post("urlPrev: a.md\nurlNext: ../b.md", "x"));
        assert_eq!((found.previous.as_deref(), found.next.as_deref()), (Some("a.md"), Some("../b.md")));
    }

    #[test]
    fn nothing_is_added_when_there_is_nothing_to_show() {
        for source in ["# T\n\ntext", "---\ntitle: T\ngenerator: g\n---\n# T\n\ntext", "---\nauthor: [bad\n---\ntext"] {
            let doc = parse(source);
            assert_eq!(assemble(&doc, &LABELS), doc.blocks, "{source:?}");
        }
    }

    #[test]
    fn the_byline_sits_under_a_leading_heading_and_at_the_top_otherwise() {
        let under = assemble(&post("author: A", "# Title\n\nText"), &LABELS);
        assert_eq!(texts(&under), ["Title", "A", "Text"]);
        let top = assemble(&post("author: A", "Text\n\n# Later"), &LABELS);
        assert_eq!(texts(&top), ["A", "Text", "Later"]);
    }

    #[test]
    fn the_front_comes_in_reading_order_and_the_row_closes_the_page() {
        let doc = post(
            "author: A\ndescription: About\nenclosure: http://e.com/s.mp3\nurlPrev: p.md\nurlNext: n.md",
            "# Title\n\nBody",
        );
        let blocks = assemble(&doc, &LABELS);
        assert_eq!(texts(&blocks), ["Title", "A", "About", "Listen: s.mp3", "Body", "---", "← Previous · Next →"]);
    }

    #[test]
    fn the_row_links_to_what_exists_and_the_enclosure_to_its_address() {
        let blocks = assemble(&post("urlPrev: p.md\nenclosure: http://e.com/s.mp3", "Body"), &LABELS);
        let links = |block: &Block| match block {
            Block::Paragraph(spans) => spans.iter().filter_map(|s| s.link.clone()).collect::<Vec<_>>(),
            _ => vec![],
        };
        assert_eq!(links(&blocks[0]), ["http://e.com/s.mp3"]);
        assert_eq!(texts(&blocks)[3], "← Previous");
        assert_eq!(links(&blocks[3]), ["p.md"]);
    }

    #[test]
    fn the_byline_and_description_are_quiet_and_the_body_is_not() {
        let blocks = assemble(&post("author: A\ndescription: About", "Body"), &LABELS);
        let quiet = |block: &Block| matches!(block, Block::Paragraph(spans) if spans.iter().all(|s| s.style.quiet));
        assert!(quiet(&blocks[0]) && quiet(&blocks[1]) && !quiet(&blocks[2]));
    }
}
