//! Inline content: flattens nested emphasis, links and code into styled spans,
//! with images kept apart so a paragraph can be split around them.

use pulldown_cmark::{Event, Tag, TagEnd};

use super::blocks::Events;
use super::{Span, Style, html, image_description};

fn is_inline_tag(tag: &Tag) -> bool {
    matches!(
        tag,
        Tag::Emphasis
            | Tag::Strong
            | Tag::Strikethrough
            | Tag::Link { .. }
            | Tag::Image { .. }
            | Tag::Superscript
            | Tag::Subscript
    )
}

fn is_inline_end(tag: &TagEnd) -> bool {
    matches!(
        tag,
        TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Link
            | TagEnd::Image
            | TagEnd::Superscript
            | TagEnd::Subscript
    )
}

pub fn is_inline(event: &Event) -> bool {
    match event {
        Event::Start(tag) => is_inline_tag(tag),
        Event::End(tag) => is_inline_end(tag),
        Event::Html(_) | Event::Rule => false,
        _ => true,
    }
}

#[derive(Default)]
struct Depth {
    strong: u32,
    emphasis: u32,
    strike: u32,
}

impl Depth {
    fn style(&self) -> Style {
        Style { strong: self.strong > 0, emphasis: self.emphasis > 0, strike: self.strike > 0, code: false }
    }
}

/// Styled text, or an image with its description and the style around it.
#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Span(Span),
    Image { src: String, alt: String, link: Option<String>, style: Style },
}

/// Appends text, merging with the previous span when style and link match.
fn push(parts: &mut Vec<Inline>, text: &str, style: Style, link: Option<&String>) {
    match parts.last_mut() {
        Some(Inline::Span(last)) if last.style == style && last.link.as_ref() == link => last.text.push_str(text),
        _ => parts.push(Inline::Span(Span { text: text.to_string(), style, link: link.cloned() })),
    }
}

/// Inline content as spans only, each image written as its description:
/// for headings and table cells, which images never split.
pub fn inlines(events: &mut Events) -> Vec<Span> {
    flatten(parts(events))
}

pub fn flatten(parts: Vec<Inline>) -> Vec<Span> {
    parts
        .into_iter()
        .fold(Vec::new(), |mut spans: Vec<Inline>, part| {
            match part {
                Inline::Span(span) => push(&mut spans, &span.text, span.style, span.link.as_ref()),
                Inline::Image { alt, link, style, .. } => {
                    push(&mut spans, &image_description(&alt), Style { emphasis: true, ..style }, link.as_ref())
                }
            }
            spans
        })
        .into_iter()
        .filter_map(|part| match part {
            Inline::Span(span) => Some(span),
            Inline::Image { .. } => None,
        })
        .collect()
}

/// Consumes a run of inline events, flattening nesting into styled spans and images.
pub fn parts(events: &mut Events) -> Vec<Inline> {
    let mut depth = Depth::default();
    let mut links: Vec<String> = Vec::new();
    let mut spans: Vec<Inline> = Vec::new();
    while events.peek(0).is_some_and(is_inline) {
        let style = depth.style();
        let link = links.last();
        match events.next().expect("peeked") {
            Event::Text(t) => push(&mut spans, &t, style, link),
            // Raw HTML is stripped, never interpreted (ADR-002).
            Event::InlineHtml(t) => push(&mut spans, &html::strip(&t), style, link),
            Event::Code(t) | Event::InlineMath(t) | Event::DisplayMath(t) => {
                push(&mut spans, &t, Style { code: true, ..style }, link)
            }
            Event::SoftBreak => push(&mut spans, " ", style, link),
            Event::HardBreak => push(&mut spans, "\n", style, link),
            Event::FootnoteReference(label) => push(&mut spans, &format!("[^{label}]"), style, link),
            Event::Start(Tag::Image { dest_url, .. }) => {
                // The nested call consumes the image's End event.
                let alt: String = inlines(events).into_iter().map(|s| s.text).collect();
                spans.push(Inline::Image { src: dest_url.to_string(), alt, link: link.cloned(), style });
            }
            Event::Start(Tag::Strong) => depth.strong += 1,
            Event::End(TagEnd::Strong) => depth.strong -= 1,
            Event::Start(Tag::Emphasis) => depth.emphasis += 1,
            Event::End(TagEnd::Emphasis) => depth.emphasis -= 1,
            Event::Start(Tag::Strikethrough) => depth.strike += 1,
            Event::End(TagEnd::Strikethrough) => depth.strike -= 1,
            Event::Start(Tag::Link { dest_url, .. }) => links.push(dest_url.to_string()),
            Event::End(TagEnd::Link) => {
                links.pop();
            }
            Event::End(TagEnd::Image) => break,
            _ => {}
        }
    }
    spans
}
