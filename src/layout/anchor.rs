//! Keeping the reader's place when the page is laid out again (images arriving).

use super::Page;

/// The scroll offset for `new` that keeps the text the reader was looking at in
/// `old`, at `scroll`, where it was on screen. The first fragment reaching into
/// the view is the reference, found again by its top-level block and its place
/// among that block's fragments; blocks don't change when images arrive.
pub fn keep(old: &Page, new: &Page, scroll: f32) -> f32 {
    let Some((index, reference)) =
        old.fragments.iter().enumerate().find(|(_, fragment)| fragment.bounds.y + fragment.bounds.height > scroll)
    else {
        return scroll;
    };
    let ordinal = old.fragments[..index].iter().filter(|fragment| fragment.block == reference.block).count();
    let same = new.fragments.iter().filter(|fragment| fragment.block == reference.block).nth(ordinal);
    let same = same.or_else(|| new.fragments.iter().find(|fragment| fragment.block == reference.block));
    match same {
        Some(fragment) => (fragment.bounds.y - (reference.bounds.y - scroll)).max(0.0),
        None => scroll,
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Page, layout};
    use super::*;
    use crate::layout::{ImageNote, ImageView, no_images};
    use crate::model::parse;
    use crate::theme::{ReadingFont, fonts, reading};

    fn page(source: &str, view: &dyn Fn(&str) -> ImageView) -> Page {
        fonts::install();
        layout(&parse(source).blocks, &reading(ReadingFont::Literata), 400.0, view)
    }

    const POST: &str = "Intro paragraph.\n\n![pic](a.png)\n\nFirst after.\n\nSecond after.\n\nThird after.";

    fn y_of(page: &Page, text: &str) -> f32 {
        page.fragments.iter().find(|f| f.text.starts_with(text)).unwrap().bounds.y
    }

    #[test]
    fn an_image_arriving_above_the_view_keeps_the_text_in_place() {
        let before = page(POST, &no_images);
        let after = page(POST, &|_| ImageView::Shown { width: 100, height: 300 });
        let scroll = y_of(&before, "Second after") - 10.0;
        let kept = keep(&before, &after, scroll);
        assert_eq!(y_of(&after, "Second after") - kept, 10.0);
    }

    #[test]
    fn an_image_arriving_below_the_view_changes_nothing() {
        let before = page(POST, &no_images);
        let after = page(POST, &|_| ImageView::Shown { width: 100, height: 300 });
        assert_eq!(keep(&before, &after, 0.0), 0.0);
    }

    #[test]
    fn a_placeholder_becoming_an_image_inside_the_view_keeps_the_top_line() {
        let before = page(POST, &|_| ImageView::Placeholder {
            note: Some(ImageNote { text: "Loading image…".into(), clickable: false }),
        });
        let after = page(POST, &|_| ImageView::Shown { width: 100, height: 300 });
        let scroll = y_of(&before, "Intro") + 2.0;
        let kept = keep(&before, &after, scroll);
        let first_visible = before.fragments.iter().find(|f| f.bounds.y + f.bounds.height > scroll).unwrap();
        let offset = first_visible.bounds.y - scroll;
        let same = after.fragments.iter().find(|f| f.block == first_visible.block).unwrap();
        assert_eq!(same.bounds.y - kept, offset);
    }

    #[test]
    fn a_view_starting_at_the_image_itself_keeps_the_image_there() {
        let before = page(POST, &|_| ImageView::Placeholder {
            note: Some(ImageNote { text: "Loading image…".into(), clickable: false }),
        });
        let after = page(POST, &|_| ImageView::Shown { width: 100, height: 300 });
        let scroll = y_of(&before, "[image: pic]") - 4.0;
        let kept = keep(&before, &after, scroll);
        assert_eq!(after.images[0].bounds.y - kept, 4.0, "the image starts where its description did");
    }

    #[test]
    fn scrolling_never_goes_negative() {
        let before = page(POST, &|_| ImageView::Shown { width: 100, height: 300 });
        let after = page(POST, &no_images);
        assert!(keep(&before, &after, 500.0) >= 0.0);
    }
}
