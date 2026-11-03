//! Images in the flow: a shown image is a box of known size; any other is its
//! description, then a note saying why (ADR-004).

use iced::Rectangle;
use iced::advanced::graphics::text::cosmic_text as ct;

use super::flow::{Column, Flow, Ink};
use super::{Fragment, Joint, LaidImage};
use crate::model::{Align, Span, Style, image_description};

/// How an image source shows on the page. Built by the UI from the session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageView {
    /// Decoded, at this size in pixels.
    Shown { width: u32, height: u32 },
    /// Not shown: the description, then this note, if any.
    Placeholder { note: Option<ImageNote> },
}

/// The line under a placeholder's description, in the words `content` gives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageNote {
    pub text: String,
    /// Clicking it acts on the image (loads or retries it).
    pub clickable: bool,
}

/// An image block's source, description and enclosing link.
#[derive(Debug, Clone, Copy)]
pub(super) struct Source<'a> {
    pub src: &'a str,
    pub alt: &'a str,
    pub link: &'a Option<String>,
}

/// Every image as its bare description: for callers with no images to show.
pub fn no_images(_: &str) -> ImageView {
    ImageView::Placeholder { note: None }
}

/// An image of `own` pixels laid out to fit within `max_width` (whole pixels) and `max_height`:
/// one scale factor, the smaller of the two, never above 1; each side rounded, at least 1.
pub(super) fn fit((own_width, own_height): (u32, u32), max_width: f32, max_height: f32) -> (f32, f32) {
    let (own_width, own_height) = (own_width as f32, own_height as f32);
    let scale = (max_width.floor() / own_width).min(max_height / own_height).min(1.0);
    ((own_width * scale).round().max(1.0), (own_height * scale).round().max(1.0))
}

impl Flow<'_> {
    pub(super) fn image(&mut self, source: Source, column: Column, ink: Ink) {
        match (self.images)(source.src) {
            ImageView::Shown { width, height } => self.picture(source, column, (width, height), ink),
            ImageView::Placeholder { note } => {
                let style = Style { emphasis: true, ..Style::default() };
                let description = [Span { text: image_description(source.alt), style, link: source.link.clone() }];
                self.text(&description, column.x, column.width, ink, Align::Left, ct::Wrap::WordOrGlyph);
                if let Some(note) = note {
                    self.note(source.src, &note, column, ink);
                }
            }
        }
    }

    /// Never scaled up; scaled down to the column and to the maximum image height, aspect kept.
    fn picture(&mut self, source: Source, column: Column, own: (u32, u32), ink: Ink) {
        let (width, height) = fit(own, column.width, self.theme.image_max_height());
        let bounds = Rectangle { x: column.x, y: self.y, width, height };
        let link = source.link;
        self.page.images.push(LaidImage { src: source.src.to_string(), bounds, link: link.clone() });
        // Paints nothing: the box takes part in selection, copying and link clicks.
        let text = image_description(source.alt);
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
    fn note(&mut self, src: &str, note: &ImageNote, column: Column, ink: Ink) {
        let small = Ink { size: (ink.size * self.theme.note_scale).round(), color: self.theme.muted, ..ink };
        let look = note.clickable.then(|| src.to_string());
        let first = self.page.fragments.len();
        self.text(
            &[Span { text: note.text.clone(), style: Style::default(), link: look }],
            column.x,
            column.width,
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
    use crate::theme::{Look, fonts, reading};

    const WIDTH: f32 = 400.0;

    fn page_with(source: &str, view: impl Fn(&str) -> ImageView) -> Page {
        fonts::install();
        layout(&parse(source).blocks, &reading(Look::default()), WIDTH, &view)
    }

    fn shown(width: u32, height: u32) -> impl Fn(&str) -> ImageView {
        move |_| ImageView::Shown { width, height }
    }

    fn column() -> f32 {
        reading(Look::default()).column_width(WIDTH)
    }

    fn size(page: &Page) -> (f32, f32) {
        let bounds = page.images[0].bounds;
        (bounds.width, bounds.height)
    }

    #[test]
    fn a_shown_image_keeps_its_size_when_it_fits() {
        assert_eq!(size(&page_with("![a](a.png)", shown(100, 50))), (100.0, 50.0));
        assert_eq!(size(&page_with("![a](a.png)", shown(1, 1))), (1.0, 1.0));
        assert_eq!(size(&page_with("![a](a.png)", shown(300, 400))), (300.0, 400.0));
    }

    fn max_height() -> f32 {
        reading(Look::default()).image_max_height()
    }

    #[test]
    fn fit_scales_by_the_smaller_factor_in_whole_pixels_and_never_up() {
        let table = [
            // own size, column, maximum height, expected
            ((200, 3000), 500.0, 540.0, (36.0, 540.0)),
            ((100, 540), 500.0, 540.0, (100.0, 540.0)),
            ((100, 541), 500.0, 540.0, (100.0, 540.0)),
            ((100, 10_000), 500.0, 540.0, (5.0, 540.0)),
            ((1, 10_000), 500.0, 540.0, (1.0, 540.0)),
            ((640, 480), 720.0, 540.0, (640.0, 480.0)),
            ((4000, 100), 500.0, 540.0, (500.0, 13.0)),
            ((4000, 6000), 500.0, 540.0, (360.0, 540.0)),
            ((6000, 4000), 500.0, 540.0, (500.0, 333.0)),
            ((10, 10), 500.0, 540.0, (10.0, 10.0)),
            ((100, 100), 500.5, 540.0, (100.0, 100.0)),
            ((1000, 1000), 500.5, 540.0, (500.0, 500.0)),
        ];
        for (own, column, max, expected) in table {
            assert_eq!(fit(own, column, max), expected, "{own:?} in {column} wide, {max} tall");
        }
    }

    #[test]
    fn a_tall_image_is_scaled_to_the_maximum_height_with_the_text_right_below_it() {
        let p = page_with("![t](t.png)\n\nThe first line.", shown(200, 3000));
        assert_eq!(size(&p), (36.0, max_height()));
        let image = p.images[0].bounds;
        let text = p.fragments.iter().find(|f| f.text == "The first line.").expect("the paragraph");
        let theme = reading(Look::default());
        assert_eq!(text.bounds.y, image.y + image.height + theme.block_gap, "a block gap below the scaled box");
        assert!(text.bounds.y <= image.y + theme.image_max_height() + theme.block_gap);
    }

    #[test]
    fn a_larger_text_size_allows_a_taller_image() {
        let at = |px: u64| {
            let look = Look { size: crate::theme::TextSize::from_px(px).unwrap(), ..Look::default() };
            fonts::install();
            let p = layout(&parse("![t](t.png)").blocks, &reading(look), WIDTH, &shown(200, 3000));
            p.images[0].bounds.height
        };
        assert_eq!((at(12), at(18), at(24), at(32)), (360.0, 540.0, 720.0, 960.0));
    }

    #[test]
    fn a_capped_image_is_clickable_only_inside_its_scaled_box() {
        let p = page_with("[![t](t.png)](https://x.org/)", shown(200, 3000));
        let b = p.images[0].bounds;
        assert_eq!(b.height, max_height());
        let at = |x: f32, y: f32| p.link_at(Point::new(x, y));
        assert_eq!(at(b.x + b.width / 2.0, b.y + b.height - 1.0), Some("https://x.org/"));
        assert_eq!(at(b.x + b.width + 5.0, b.y + 10.0), None, "right of the scaled box");
        assert_eq!(at(b.x + b.width / 2.0, b.y + b.height + 1.0), None, "below it, where the natural size would reach");
    }

    #[test]
    fn a_capped_linked_image_is_one_stop_with_the_scaled_ring() {
        let p = page_with("[![t](t.png)](https://x.org/)", shown(200, 3000));
        let stops = p.stops();
        assert_eq!(stops.len(), 1);
        assert_eq!(stops[0].rects, [p.images[0].bounds]);
    }

    #[test]
    fn a_capped_image_copies_as_its_description_once() {
        let p = page_with("Before\n\n![a tall one](t.png)\n\nafter", shown(200, 3000));
        let last = p.fragments.len() - 1;
        let all = p.selected_text(
            Caret { fragment: 0, byte: 0 },
            Caret { fragment: last, byte: p.fragments[last].text.len() },
        );
        assert_eq!(all, "Before\n[image: a tall one]\nafter");
    }

    #[test]
    fn a_very_thin_linked_image_still_has_a_box_a_stop_and_a_link() {
        let p = page_with("[![t](t.png)](https://x.org/)", shown(1, 10_000));
        assert_eq!(size(&p), (1.0, max_height()));
        assert_eq!(p.stops().len(), 1);
        let b = p.images[0].bounds;
        assert_eq!(p.link_at(Point::new(b.x + 0.5, b.y + 100.0)), Some("https://x.org/"));
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
        let note = |clickable| {
            move |_: &str| ImageView::Placeholder { note: Some(ImageNote { text: "Click to load".into(), clickable }) }
        };
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
        let view = |_: &str| ImageView::Placeholder {
            note: Some(ImageNote { text: "Click to load".into(), clickable: true }),
        };
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
        let view = |_: &str| ImageView::Placeholder {
            note: Some(ImageNote { text: "Click to load".into(), clickable: true }),
        };
        let p = page_with("[![a](a.png)](https://x.org/)", view);
        assert_eq!(p.fragments[0].link.as_deref(), Some("https://x.org/"));
        assert_eq!(p.fragments[1].action.as_deref(), Some("a.png"));
    }

    #[test]
    fn a_point_on_a_clickable_note_names_its_image_and_elsewhere_nothing() {
        let clickable = |_: &str| ImageView::Placeholder {
            note: Some(ImageNote { text: "Click to load".into(), clickable: true }),
        };
        let p = page_with("![a](a.png)", clickable);
        let note = p.fragments[1].bounds;
        let on_note = Point::new(note.x + 2.0, note.y + note.height / 2.0);
        assert_eq!(p.action_at(on_note), Some("a.png"));
        assert_eq!(p.link_at(on_note), None);
        let description = p.fragments[0].bounds;
        assert_eq!(p.action_at(Point::new(description.x + 2.0, description.y + 2.0)), None);
        let off = |_: &str| ImageView::Placeholder {
            note: Some(ImageNote { text: "Web images are off.".into(), clickable: false }),
        };
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
