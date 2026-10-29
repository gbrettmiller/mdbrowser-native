//! Images in the flow: a shown image is a box of known size; any other is its
//! description, then a note saying why (ADR-004).

use iced::Rectangle;
use iced::advanced::graphics::text::cosmic_text as ct;

use super::flow::{Flow, Ink};
use super::{Fragment, Joint, LaidImage};
use crate::model::{Align, Span, Style, image_description};

/// How an image source shows on the page. Built by the UI from the session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageView {
    /// Decoded, at this size in pixels.
    Shown { width: u32, height: u32 },
    /// Not shown: the description, then this note, if any; `true` if clicking it acts.
    Placeholder { note: Option<(String, bool)> },
}

/// Every image as its bare description: for callers with no images to show.
pub fn no_images(_: &str) -> ImageView {
    ImageView::Placeholder { note: None }
}

/// The note under a placeholder is smaller than the text.
const NOTE_SCALE: f32 = 0.85;

impl Flow<'_> {
    pub(super) fn image(&mut self, src: &str, alt: &str, link: &Option<String>, x: f32, width: f32, ink: Ink) {
        match (self.images)(src) {
            ImageView::Shown { width: own, height } => self.picture(src, alt, link, (x, width), (own, height), ink),
            ImageView::Placeholder { note } => {
                let style = Style { emphasis: true, ..Style::default() };
                let description = [Span { text: image_description(alt), style, link: link.clone() }];
                self.text(&description, x, width, ink, Align::Left, ct::Wrap::WordOrGlyph);
                if let Some((note, clickable)) = note {
                    self.note(src, &note, clickable, x, width, ink);
                }
            }
        }
    }

    /// Never scaled up; scaled down to the column (`x` and its width), aspect kept,
    /// in whole pixels.
    fn picture(&mut self, src: &str, alt: &str, link: &Option<String>, column: (f32, f32), size: (u32, u32), ink: Ink) {
        let (x, column) = column;
        let (own_width, own_height) = (size.0 as f32, size.1 as f32);
        let width = own_width.min(column.floor());
        let height = (own_height * width / own_width).round().max(1.0);
        let bounds = Rectangle { x, y: self.y, width, height };
        self.page.images.push(LaidImage { src: src.to_string(), bounds, link: link.clone() });
        // Paints nothing: the box takes part in selection, copying and link clicks.
        let text = image_description(alt);
        let stops = vec![(0.0, 0), (width, text.len())];
        self.page.fragments.push(Fragment {
            bounds,
            text,
            font: ink.font,
            size: ink.size,
            color: ink.color,
            link: link.clone(),
            stops,
            line: self.line,
            joint: Joint::Break,
            block: self.block,
            action: None,
            note: false,
        });
        self.line += 1;
        self.y += height;
    }

    /// The second line. A clickable note borrows the link look (colour, underline)
    /// but acts on the image, never as a link.
    fn note(&mut self, src: &str, note: &str, clickable: bool, x: f32, width: f32, ink: Ink) {
        let small = Ink { size: (ink.size * NOTE_SCALE).round(), color: self.theme.muted, ..ink };
        let look = clickable.then(|| src.to_string());
        let first = self.page.fragments.len();
        self.text(
            &[Span { text: note.to_string(), style: Style::default(), link: look }],
            x,
            width,
            small,
            Align::Left,
            ct::Wrap::WordOrGlyph,
        );
        for fragment in &mut self.page.fragments[first..] {
            fragment.action = fragment.link.take();
            fragment.note = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use iced::Point;

    use super::super::{Caret, Page, layout};
    use super::*;
    use crate::model::parse;
    use crate::theme::{ReadingFont, fonts, reading};

    const WIDTH: f32 = 400.0;

    fn page_with(source: &str, view: impl Fn(&str) -> ImageView) -> Page {
        fonts::install();
        layout(&parse(source).blocks, &reading(ReadingFont::Literata), WIDTH, &view)
    }

    fn shown(width: u32, height: u32) -> impl Fn(&str) -> ImageView {
        move |_| ImageView::Shown { width, height }
    }

    fn column() -> f32 {
        let theme = reading(ReadingFont::Literata);
        (WIDTH - 2.0 * theme.gutter).min(theme.measure).max(80.0)
    }

    fn size(page: &Page) -> (f32, f32) {
        let bounds = page.images[0].bounds;
        (bounds.width, bounds.height)
    }

    #[test]
    fn a_shown_image_keeps_its_size_when_it_fits() {
        assert_eq!(size(&page_with("![a](a.png)", shown(100, 50))), (100.0, 50.0));
        assert_eq!(size(&page_with("![a](a.png)", shown(1, 1))), (1.0, 1.0));
        assert_eq!(size(&page_with("![a](a.png)", shown(1, 10_000))), (1.0, 10_000.0));
    }

    #[test]
    fn a_wider_image_is_scaled_to_the_column_with_its_aspect_kept() {
        let column = column();
        assert_eq!(size(&page_with("![a](a.png)", shown(4_000, 100))), (column, (100.0 * column / 4_000.0).round()));
        let exact = column as u32;
        assert_eq!(size(&page_with("![a](a.png)", shown(exact, 30))), (exact as f32, 30.0));
    }

    #[test]
    fn laid_images_carry_their_source_and_link() {
        let p = page_with("[![a](a.png)](https://x.org/)", shown(10, 10));
        assert_eq!(p.images[0].src, "a.png");
        assert_eq!(p.images[0].link.as_deref(), Some("https://x.org/"));
    }

    #[test]
    fn a_shown_image_is_selectable_and_linked_as_its_description() {
        let p = page_with("Before\n\n[![a red box](a.png)](https://x.org/)\n\nafter", shown(40, 20));
        let last = p.fragments.len() - 1;
        let all = p.selected_text(
            Caret { fragment: 0, byte: 0 },
            Caret { fragment: last, byte: p.fragments[last].text.len() },
        );
        assert_eq!(all, "Before\n[image: a red box]\nafter");
        let bounds = p.images[0].bounds;
        let centre = Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height / 2.0);
        assert_eq!(p.link_at(centre), Some("https://x.org/"));
        assert!(p.texts.len() == 2, "the image's fragment paints nothing");
    }

    #[test]
    fn a_placeholder_is_its_description_then_its_note() {
        let note =
            |clickable| move |_: &str| ImageView::Placeholder { note: Some(("Click to load".into(), clickable)) };
        let p = page_with("![a cat](a.png)", note(true));
        let texts: Vec<&str> = p.fragments.iter().map(|f| f.text.as_str()).collect();
        assert_eq!(texts, ["[image: a cat]", "Click to load"]);
        assert_eq!(p.fragments[1].action.as_deref(), Some("a.png"));
        assert_eq!(p.fragments[1].link, None, "a note never follows a link");
        assert!(p.fragments[1].size < p.fragments[0].size);
        assert!(p.images.is_empty());
        let p = page_with("![a cat](a.png)", note(false));
        assert_eq!(p.fragments[1].action, None);
        let p = page_with("![a cat](a.png)", |_| ImageView::Placeholder { note: None });
        assert_eq!(p.fragments.len(), 1);
    }

    #[test]
    fn copying_across_a_placeholder_copies_its_description_but_not_its_note() {
        let view = |_: &str| ImageView::Placeholder { note: Some(("Click to load".into(), true)) };
        let p = page_with("Before\n\n![a red box](a.png)\n\nafter", view);
        let last = p.fragments.len() - 1;
        let all = p.selected_text(
            Caret { fragment: 0, byte: 0 },
            Caret { fragment: last, byte: p.fragments[last].text.len() },
        );
        assert_eq!(all, "Before\n[image: a red box]\nafter");
    }

    #[test]
    fn a_linked_placeholder_keeps_its_link_on_the_description() {
        let view = |_: &str| ImageView::Placeholder { note: Some(("Click to load".into(), true)) };
        let p = page_with("[![a](a.png)](https://x.org/)", view);
        assert_eq!(p.fragments[0].link.as_deref(), Some("https://x.org/"));
        assert_eq!(p.fragments[1].action.as_deref(), Some("a.png"));
    }

    #[test]
    fn a_point_on_a_clickable_note_names_its_image_and_elsewhere_nothing() {
        let clickable = |_: &str| ImageView::Placeholder { note: Some(("Click to load".into(), true)) };
        let p = page_with("![a](a.png)", clickable);
        let note = p.fragments[1].bounds;
        let on_note = Point::new(note.x + 2.0, note.y + note.height / 2.0);
        assert_eq!(p.action_at(on_note), Some("a.png"));
        assert_eq!(p.link_at(on_note), None);
        let description = p.fragments[0].bounds;
        assert_eq!(p.action_at(Point::new(description.x + 2.0, description.y + 2.0)), None);
        let off = |_: &str| ImageView::Placeholder { note: Some(("Web images are off.".into(), false)) };
        let p = page_with("![a](a.png)", off);
        let note = p.fragments[1].bounds;
        assert_eq!(p.action_at(Point::new(note.x + 2.0, note.y + note.height / 2.0)), None);
    }

    #[test]
    fn fragments_know_their_top_level_block() {
        let p = page_with("one\n\n> two\n\n- three", no_images);
        let blocks: Vec<usize> = p.fragments.iter().map(|f| f.block).collect();
        assert_eq!(blocks.first(), Some(&0));
        assert!(p.fragments.iter().any(|f| f.text == "two" && f.block == 1));
        assert!(p.fragments.iter().any(|f| f.text == "three" && f.block == 2));
    }
}
