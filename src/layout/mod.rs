//! The rendering engine: document tree + theme + width → a positioned `Page`.
//! Text shaping and line breaking are borrowed (cosmic-text, through iced's
//! shared font system); block flow, selection and hit-testing are ours.

pub mod anchor;
mod flow;
mod image;
mod select;
mod text;

#[cfg(test)]
mod golden;

use std::sync::Arc;

use iced::advanced::graphics::text::cosmic_text::Buffer;
use iced::advanced::graphics::text::font_system;
use iced::{Color, Font, Point, Rectangle};

pub use image::{ImageNote, ImageView, no_images};

use crate::model::Block;
use crate::theme::Theme;

/// A shaped text block, painted as-is (text is shaped once, at layout).
#[derive(Debug, Clone)]
pub struct LaidText {
    pub buffer: Arc<Buffer>,
    pub position: Point,
    pub bounds: Rectangle,
}

/// A run of identically styled glyphs on one line: the unit of hit-testing and selection.
#[derive(Debug, Clone, PartialEq)]
pub struct Fragment {
    pub bounds: Rectangle,
    pub text: String,
    pub font: Font,
    pub size: f32,
    pub color: Color,
    pub link: Option<String>,
    /// Caret stops: (x offset from `bounds.x`, byte offset into `text`), ascending.
    pub stops: Vec<(f32, usize)>,
    /// Visual line id: fragments side by side (marker + text, table cells) share one.
    pub line: usize,
    pub joint: Joint,
    /// The top-level block it belongs to: a key that holds while images arrive.
    pub block: usize,
    /// For an image's clickable note: the image source a click loads.
    pub action: Option<String>,
    /// Part of an image's note: interface text, never copied.
    pub note: bool,
}

/// How a fragment continues from the one before it, for copying text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Joint {
    /// Same shaped line, directly adjacent.
    Flush,
    /// Soft wrap: the paragraph continues on the next visual line.
    Wrap,
    /// Hard break or a new block.
    Break,
}

/// A shown image's box, for painting and clicks.
#[derive(Debug, Clone, PartialEq)]
pub struct LaidImage {
    pub src: String,
    pub bounds: Rectangle,
    pub link: Option<String>,
}

/// A filled rectangle: code backgrounds, quote bars, rules, underlines, checkboxes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fill {
    pub bounds: Rectangle,
    pub color: Color,
}

#[derive(Debug, Clone, Default)]
pub struct Page {
    pub texts: Vec<LaidText>,
    pub fills: Vec<Fill>,
    pub fragments: Vec<Fragment>,
    pub images: Vec<LaidImage>,
    /// Heading slug → top of the heading, in document order.
    pub anchors: Vec<(String, f32)>,
    pub height: f32,
}

/// A position in the text: fragment index + byte offset within it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Caret {
    pub fragment: usize,
    pub byte: usize,
}

/// `images` says how each image source shows: drawn at a known size, or as its
/// description and a note.
pub fn layout(blocks: &[Block], theme: &Theme, width: f32, images: &dyn Fn(&str) -> ImageView) -> Page {
    let mut system = font_system().write().expect("font system lock");
    flow::run(blocks, theme, system.raw(), width, images)
}

impl Page {
    pub fn anchor_y(&self, slug: &str) -> Option<f32> {
        self.anchors.iter().find(|(s, _)| s == slug).map(|(_, y)| *y)
    }

    /// Where to scroll so the heading sits `margin` below the top of the view.
    pub fn scroll_offset_for(&self, slug: &str, margin: f32) -> Option<f32> {
        self.anchor_y(slug).map(|y| (y - margin).max(0.0))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::model::parse;
    use crate::theme::{Look, ReadingFont, Spacing, TextSize, fonts, reading};

    const WIDTH: f32 = 400.0;

    pub fn page_at(source: &str, font: ReadingFont, width: f32) -> Page {
        fonts::install();
        layout(&parse(source).blocks, &reading(Look { font, ..Look::default() }), width, &no_images)
    }

    fn page(source: &str) -> Page {
        page_at(source, ReadingFont::Literata, WIDTH)
    }

    fn page_looking(source: &str, look: Look) -> Page {
        fonts::install();
        layout(&parse(source).blocks, &reading(look), WIDTH, &no_images)
    }

    const SPACED: &str = "# Title\n\nA paragraph that runs over a few lines so the line height shows up in the page's \
        height, because the text wraps at this width.\n\n- one\n- two\n- three\n\nAnother paragraph.";

    #[test]
    fn compact_pages_are_shorter_and_spacious_pages_taller() {
        let height = |spacing| page_looking(SPACED, Look { spacing, ..Look::default() }).height;
        let (compact, comfortable, spacious) =
            (height(Spacing::Compact), height(Spacing::Comfortable), height(Spacing::Spacious));
        assert!(compact < comfortable && comfortable < spacious, "{compact} {comfortable} {spacious}");
    }

    #[test]
    fn larger_text_is_set_larger_and_a_longer_page() {
        let at = |size: TextSize| page_looking(SPACED, Look { size, ..Look::default() });
        let (small, large) = (at(TextSize::default().smaller()), at(TextSize::default().larger()));
        let body = |p: &Page| p.fragments.iter().find(|f| f.text.starts_with("Another")).map(|f| f.size).unwrap();
        assert!(body(&small) < body(&large));
        assert!(small.height < large.height);
    }

    fn centre(rect: Rectangle) -> Point {
        Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0)
    }

    #[test]
    fn long_paragraph_wraps_inside_the_measure() {
        let p = page(&"word ".repeat(80));
        let lines: std::collections::BTreeSet<_> = p.fragments.iter().map(|f| f.line).collect();
        assert!(lines.len() > 3, "expected wrapping, got {} lines", lines.len());
        let left = p.fragments.iter().map(|f| f.bounds.x).fold(f32::MAX, f32::min);
        let measure = reading(Look::default()).measure.min(WIDTH);
        assert!(p.fragments.iter().all(|f| f.bounds.x + f.bounds.width <= left + measure + 0.5));
    }

    #[test]
    fn blocks_stack_top_to_bottom_and_page_covers_them() {
        let p = page("# Heading\n\nBody text\n\n- item");
        let ys: Vec<f32> = p.fragments.iter().map(|f| f.bounds.y).collect();
        assert!(ys.windows(2).all(|w| w[0] <= w[1]), "{ys:?}");
        let bottom = p.fragments.iter().map(|f| f.bounds.y + f.bounds.height).fold(0.0, f32::max);
        assert!(p.height >= bottom);
    }

    #[test]
    fn heading_is_larger_and_bold() {
        let p = page("# Big\n\nsmall");
        assert!(p.fragments[0].size > p.fragments[1].size);
        assert_eq!(p.fragments[0].font.weight, iced::font::Weight::Bold);
    }

    #[test]
    fn body_text_uses_the_chosen_reading_font() {
        let family = |font| page_at("words", font, WIDTH).fragments[0].font.family;
        assert_eq!(family(ReadingFont::Inter), Font::with_name("Inter").family);
        assert_eq!(family(ReadingFont::SourceSerif), Font::with_name("Source Serif 4").family);
    }

    #[test]
    fn list_items_get_a_marker_left_of_the_text() {
        let p = page("- item");
        let marker = p.fragments.iter().find(|f| f.text == "•").expect("bullet");
        let text = p.fragments.iter().find(|f| f.text == "item").expect("text");
        assert!(marker.bounds.x < text.bounds.x);
        assert_eq!(marker.line, text.line);
    }

    #[test]
    fn task_boxes_are_drawn_not_typed() {
        let open = page("- [ ] todo");
        let done = page("- [x] done");
        assert!(open.fragments.iter().all(|f| !f.text.contains(['☐', '☑'])));
        assert!(done.fills.len() > open.fills.len(), "a checked box adds its tick");
    }

    #[test]
    fn link_hit_testing() {
        let p = page("go [there](b.md) now");
        let link = p.fragments.iter().find(|f| f.link.is_some()).expect("link fragment");
        assert_eq!(p.link_at(centre(link.bounds)), Some("b.md"));
        let plain = p.fragments.iter().find(|f| f.link.is_none()).unwrap();
        assert_eq!(p.link_at(centre(plain.bounds)), None);
    }

    #[test]
    fn shaped_texts_cover_every_fragment() {
        let p = page("# Title\n\nSome *styled* text\n\n```\ncode\n```\n\n| a | b |\n|-|-|\n| 1 | 2 |");
        for f in &p.fragments {
            assert!(p.texts.iter().any(|t| t.bounds.contains(centre(f.bounds))), "{:?} not painted", f.text);
        }
    }

    #[test]
    fn headings_record_anchor_positions() {
        let p = page("# Top\n\nbody\n\n## Intro");
        let intro = p.fragments.iter().find(|f| f.text == "Intro").unwrap();
        assert_eq!(p.anchor_y("intro"), Some(intro.bounds.y));
        assert_eq!(p.anchor_y("top"), Some(p.fragments[0].bounds.y));
        assert_eq!(p.anchor_y("missing"), None);
    }

    #[test]
    fn scrolling_to_an_anchor_leaves_a_margin_but_never_goes_negative() {
        let p = page("# Top\n\nbody\n\n## Intro");
        let intro = p.anchor_y("intro").unwrap();
        assert_eq!(p.scroll_offset_for("intro", 20.0), Some(intro - 20.0));
        assert_eq!(p.scroll_offset_for("top", 1000.0), Some(0.0));
        assert_eq!(p.scroll_offset_for("missing", 20.0), None);
    }

    #[test]
    fn selection_spans_blocks_and_joins_lines() {
        let p = page("Alpha beta\n\nGamma delta");
        let first = &p.fragments[0];
        let last = p.fragments.last().unwrap();
        let start = p.caret_at(Point::new(first.bounds.x + 1.0, centre(first.bounds).y)).unwrap();
        let end = p.caret_at(Point::new(last.bounds.x + last.bounds.width + 50.0, centre(last.bounds).y)).unwrap();
        assert_eq!(p.selected_text(start, end), "Alpha beta\nGamma delta");
        assert_eq!(p.selected_text(end, start), "Alpha beta\nGamma delta");
        assert_eq!(p.highlight(start, end).len(), 2);
    }

    #[test]
    fn code_inside_a_word_is_shaped_in_its_own_font() {
        let width = |source: &str| {
            let p = page(source);
            let code = p.fragments.iter().find(|f| f.text == "Markdown.pl").expect("code fragment");
            code.bounds.width
        };
        assert!((width("a (`Markdown.pl`) b") - width("`Markdown.pl`")).abs() < 0.5);
        assert_eq!(
            page("a (`x`) b").selected_text(Caret { fragment: 0, byte: 0 }, Caret { fragment: 2, byte: 3 }),
            "a (x) b"
        );
    }

    fn copy_all(p: &Page) -> String {
        let last = p.fragments.len() - 1;
        p.selected_text(Caret { fragment: 0, byte: 0 }, Caret { fragment: last, byte: p.fragments[last].text.len() })
    }

    #[test]
    fn copying_rejoins_soft_wraps_but_keeps_hard_breaks() {
        let words = "word ".repeat(80);
        assert_eq!(copy_all(&page(&words)), words.trim_end());
        assert_eq!(copy_all(&page("one  \ntwo")), "one\ntwo");
    }

    #[test]
    fn copying_separates_markers_and_cells_with_spaces() {
        assert_eq!(copy_all(&page("- a\n- b")), "• a\n• b");
        assert_eq!(copy_all(&page("| x | y |\n|-|-|\n| 1 | 2 |")), "x y\n1 2");
    }

    #[test]
    fn caret_lands_between_glyphs() {
        let p = page("Hello");
        let f = &p.fragments[0];
        let middle = p.caret_at(centre(f.bounds)).unwrap();
        assert!(middle.byte > 0 && middle.byte < 5, "{middle:?}");
    }
}
