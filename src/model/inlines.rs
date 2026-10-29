//! Inline content: flattens nested emphasis, links and code into styled spans.

use pulldown_cmark::{Event, Tag, TagEnd};

use super::blocks::Events;
use super::{Span, Style};

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

/// Appends text, merging with the previous span when style and link match.
fn push(spans: &mut Vec<Span>, text: &str, style: Style, link: Option<&String>) {
    match spans.last_mut() {
        Some(last) if last.style == style && last.link.as_ref() == link => last.text.push_str(text),
        _ => spans.push(Span { text: text.to_string(), style, link: link.cloned() }),
    }
}

/// Consumes a run of inline events, flattening nesting into styled spans.
pub fn inlines(events: &mut Events) -> Vec<Span> {
    let mut depth = Depth::default();
    let mut links: Vec<String> = Vec::new();
    let mut spans: Vec<Span> = Vec::new();
    while events.peek(0).is_some_and(is_inline) {
        let style = depth.style();
        let link = links.last();
        match events.next().expect("peeked") {
            // Inline HTML is literal text, never interpreted.
            Event::Text(t) | Event::InlineHtml(t) => push(&mut spans, &t, style, link),
            Event::Code(t) | Event::InlineMath(t) | Event::DisplayMath(t) => {
                push(&mut spans, &t, Style { code: true, ..style }, link)
            }
            Event::SoftBreak => push(&mut spans, " ", style, link),
            Event::HardBreak => push(&mut spans, "\n", style, link),
            Event::FootnoteReference(label) => push(&mut spans, &format!("[^{label}]"), style, link),
            Event::Start(Tag::Image { .. }) => {
                let alt: String = inlines(events).into_iter().map(|s| s.text).collect();
                events.next();
                push(&mut spans, &format!("[image: {alt}]"), Style { emphasis: true, ..style }, link)
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
