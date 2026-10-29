//! Session image states, as layout and drawing need them.

use iced::Rectangle;
use iced::advanced::image::Handle;

use crate::content;
use crate::layout::{ImageNote, ImageView, Page};
use crate::session::images::{Images, View};

/// How `src` shows on the current page, with the note's words from `content`.
pub fn view(images: &Images, src: &str) -> ImageView {
    match images.view(src) {
        View::Shown { width, height } => ImageView::Shown { width, height },
        View::Note { note, clickable } => {
            ImageView::Placeholder { note: Some(ImageNote { text: content::image_note(note), clickable }) }
        }
    }
}

/// The laid-out boxes of shown images, with the pixels to draw in them.
pub fn pictures(page: &Page, images: &Images) -> Vec<(Rectangle, Handle)> {
    page.images
        .iter()
        .filter_map(|laid| images.picture(&laid.src).map(|picture| (laid.bounds, picture.handle.clone())))
        .collect()
}
