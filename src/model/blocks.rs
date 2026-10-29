//! Block structure: walks pulldown-cmark's event stream into owned blocks.

use pulldown_cmark::{Alignment, CodeBlockKind, Event, Tag, TagEnd};

use super::inlines::{Inline, inlines, is_inline, parts};
use super::slug::Slugger;
use super::{Align, Block, Item, Span, Style, html, text_of};

/// The parser's event stream with arbitrary lookahead (task markers can sit
/// one event deep inside a list item), plus the document-wide slug counter.
pub struct Events<'a> {
    events: Vec<Event<'a>>,
    pos: usize,
    slugger: Slugger,
}

impl<'a> Events<'a> {
    pub fn new(events: Vec<Event<'a>>) -> Self {
        Events { events, pos: 0, slugger: Slugger::default() }
    }

    pub fn peek(&self, ahead: usize) -> Option<&Event<'a>> {
        self.events.get(self.pos + ahead)
    }

    pub fn next(&mut self) -> Option<Event<'a>> {
        let event = self.events.get(self.pos).cloned();
        self.pos += 1;
        event
    }
}

/// Blocks up to (not including) the enclosing End event, or end of input.
pub fn blocks(events: &mut Events) -> Vec<Block> {
    let mut out = Vec::new();
    while let Some(event) = events.peek(0) {
        if matches!(event, Event::End(_)) {
            break;
        }
        if is_inline(event) {
            out.extend(paragraphs(parts(events)));
        } else if matches!(event, Event::Start(Tag::Paragraph)) {
            events.next();
            out.extend(paragraphs(parts(events)));
            events.next(); // End(Paragraph)
        } else {
            out.extend(block(events));
        }
    }
    out
}

fn paragraph(spans: Vec<Span>) -> Option<Block> {
    (!spans.is_empty()).then_some(Block::Paragraph(spans))
}

/// A paragraph split at each image: the text before it, the image, the text after.
/// Whitespace is trimmed only at the cuts; a paragraph without images is untouched.
fn paragraphs(parts: Vec<Inline>) -> Vec<Block> {
    let mut out = Vec::new();
    let mut text = Vec::new();
    let mut after_image = false;
    for part in parts {
        match part {
            Inline::Span(span) => text.push(span),
            Inline::Image { src, alt, link, .. } => {
                out.extend(paragraph(trimmed(std::mem::take(&mut text), after_image, true)));
                out.push(Block::Image { src, alt, link });
                after_image = true;
            }
        }
    }
    out.extend(paragraph(trimmed(text, after_image, false)));
    out
}

/// Trims whitespace at a cut (start, end or both), dropping spans left empty.
fn trimmed(mut spans: Vec<Span>, start: bool, end: bool) -> Vec<Span> {
    while let Some(first) = spans.first_mut().filter(|_| start) {
        first.text = first.text.trim_start().to_string();
        if !first.text.is_empty() {
            break;
        }
        spans.remove(0);
    }
    while let Some(last) = spans.last_mut().filter(|_| end) {
        last.text = last.text.trim_end().to_string();
        if !last.text.is_empty() {
            break;
        }
        spans.pop();
    }
    spans
}

fn block(events: &mut Events) -> Option<Block> {
    let block = match events.next()? {
        Event::Rule => return Some(Block::Rule),
        Event::Start(Tag::Heading { level, .. }) => {
            let spans = inlines(events);
            let slug = events.slugger.slug(&text_of(&spans));
            Some(Block::Heading { level: level as u8, spans, slug })
        }
        Event::Start(Tag::BlockQuote(_)) => Some(Block::Quote(blocks(events))),
        Event::Start(Tag::CodeBlock(kind)) => {
            let lang = match kind {
                CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.to_string()),
                _ => None,
            };
            Some(Block::Code { lang, text: text_until_end(events) })
        }
        Event::Start(Tag::HtmlBlock) => html_block(events),
        Event::Start(Tag::List(start)) => Some(Block::List { start, items: items(events) }),
        Event::Start(Tag::Table(aligns)) => return Some(table(events, aligns)),
        // Frontmatter, footnote definitions and anything else we don't show.
        Event::Start(_) => {
            skip_to_end(events);
            None
        }
        _ => return None,
    };
    events.next(); // the matching End
    block
}

/// Raw HTML is stripped, never interpreted (ADR-002); what text is left reads as a paragraph.
fn html_block(events: &mut Events) -> Option<Block> {
    let text = html::strip(&text_until_end(events)).split_whitespace().collect::<Vec<_>>().join(" ");
    let span = (!text.is_empty()).then(|| Span { text, style: Style::default(), link: None });
    paragraph(span.into_iter().collect())
}

fn skip_to_end(events: &mut Events) {
    let mut depth = 0usize;
    while let Some(event) = events.peek(0) {
        match event {
            Event::End(_) if depth == 0 => return,
            Event::End(_) => depth -= 1,
            Event::Start(_) => depth += 1,
            _ => {}
        }
        events.next();
    }
}

fn items(events: &mut Events) -> Vec<Item> {
    let mut items = Vec::new();
    while let Some(Event::Start(Tag::Item)) = events.peek(0) {
        events.next();
        let checked = [0, 1].iter().find_map(|&ahead| match events.peek(ahead) {
            Some(Event::TaskListMarker(checked)) => Some(*checked),
            _ => None,
        });
        items.push(Item { checked, blocks: blocks(events) });
        events.next(); // End(Item)
    }
    items
}

fn table(events: &mut Events, aligns: Vec<Alignment>) -> Block {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    while let Some(event) = events.next() {
        match event {
            Event::End(TagEnd::Table) => break,
            Event::Start(Tag::TableCell) => {
                row.push(inlines(events));
                events.next();
            }
            Event::End(TagEnd::TableHead | TagEnd::TableRow) => rows.push(std::mem::take(&mut row)),
            _ => {}
        }
    }
    let aligns = aligns
        .iter()
        .map(|align| match align {
            Alignment::Center => Align::Center,
            Alignment::Right => Align::Right,
            _ => Align::Left,
        })
        .collect();
    Block::Table { aligns, rows }
}

fn text_until_end(events: &mut Events) -> String {
    let mut text = String::new();
    while let Some(Event::Text(t) | Event::Html(t)) = events.peek(0) {
        text.push_str(t);
        events.next();
    }
    text
}
