//! Caches the laid-out page for (document, font, width, image generation). While
//! the window is being resized, expensive documents keep their previous layout
//! until the width settles, so dragging stays smooth. The image generation is the
//! one the app last applied: arriving images are coalesced and laid out from
//! `update`, where the scroll position can be kept (`layout::anchor`).

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::layout::{self, ImageView, Page};
use crate::model::Document;
use crate::theme::{self, ReadingFont};

/// How long the width must stay put before an expensive document is re-laid out.
pub const SETTLE: Duration = Duration::from_millis(150);

struct Laid {
    /// Held so its address can't be reused by a later document while cached.
    document: Arc<Document>,
    font: ReadingFont,
    width: f32,
    generation: u64,
    page: Arc<Page>,
    cost: Duration,
}

pub struct Pages {
    expensive: Duration,
    laid: Option<Laid>,
    /// The latest width asked for, and when it first was.
    requested: Option<(f32, Instant)>,
}

impl Pages {
    /// `expensive`: layouts slower than this are deferred during resizes.
    pub fn new(expensive: Duration) -> Self {
        Pages { expensive, laid: None, requested: None }
    }

    /// The page to draw, and whether it is a stale layout waiting for the width to
    /// settle. `images` is how each image shows, and the generation it belongs to.
    pub fn get(
        &mut self,
        document: &Arc<Document>,
        font: ReadingFont,
        width: f32,
        images: (&dyn Fn(&str) -> ImageView, u64),
        now: Instant,
    ) -> (Arc<Page>, bool) {
        let (view, generation) = images;
        let same_width = |a: f32, b: f32| (a - b).abs() < 0.5;
        if !self.requested.is_some_and(|(requested, _)| same_width(requested, width)) {
            self.requested = Some((width, now));
        }
        let since_resize = self.requested.map_or(Duration::ZERO, |(_, at)| now.saturating_duration_since(at));
        let same =
            |laid: &&Laid| Arc::ptr_eq(&laid.document, document) && laid.font == font && laid.generation == generation;
        if let Some(laid) = self.laid.as_ref().filter(same) {
            if same_width(laid.width, width) {
                return (laid.page.clone(), false);
            }
            if laid.cost > self.expensive && since_resize < SETTLE {
                return (laid.page.clone(), true);
            }
        }
        let started = Instant::now();
        let page = Arc::new(layout::layout(&document.blocks, &theme::reading(font), width, view));
        let cost = started.elapsed();
        self.laid = Some(Laid { document: document.clone(), font, width, generation, page: page.clone(), cost });
        (page, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::parse;
    use crate::theme::fonts;

    fn document() -> Arc<Document> {
        fonts::install();
        Arc::new(parse("# Title\n\nSome words to wrap."))
    }

    fn get(pages: &mut Pages, doc: &Arc<Document>, width: f32, generation: u64, now: Instant) -> (Arc<Page>, bool) {
        pages.get(doc, ReadingFont::Literata, width, (&layout::no_images, generation), now)
    }

    #[test]
    fn a_new_image_generation_lays_out_again_and_the_same_one_reuses() {
        let (doc, now) = (document(), Instant::now());
        let mut pages = Pages::new(Duration::MAX);
        let (first, _) = get(&mut pages, &doc, 500.0, 1, now);
        let (same, _) = get(&mut pages, &doc, 500.0, 1, now);
        let (next, settling) = get(&mut pages, &doc, 500.0, 2, now);
        assert!(Arc::ptr_eq(&first, &same));
        assert!(!Arc::ptr_eq(&first, &next) && !settling);
    }

    #[test]
    fn same_inputs_reuse_the_page() {
        let (doc, now) = (document(), Instant::now());
        let mut pages = Pages::new(Duration::MAX);
        let (first, _) = get(&mut pages, &doc, 500.0, 0, now);
        let (again, settling) = get(&mut pages, &doc, 500.0, 0, now);
        assert!(Arc::ptr_eq(&first, &again) && !settling);
    }

    #[test]
    fn a_new_font_or_document_lays_out_again() {
        let (doc, now) = (document(), Instant::now());
        let mut pages = Pages::new(Duration::MAX);
        let (first, _) = get(&mut pages, &doc, 500.0, 0, now);
        let (inter, _) = pages.get(&doc, ReadingFont::Inter, 500.0, (&layout::no_images, 0), now);
        let (other, _) = pages.get(&document(), ReadingFont::Inter, 500.0, (&layout::no_images, 0), now);
        assert!(!Arc::ptr_eq(&first, &inter) && !Arc::ptr_eq(&inter, &other));
    }

    #[test]
    fn cheap_documents_follow_the_width_immediately() {
        let (doc, now) = (document(), Instant::now());
        let mut pages = Pages::new(Duration::MAX);
        let (wide, _) = get(&mut pages, &doc, 500.0, 0, now);
        let (narrow, settling) = get(&mut pages, &doc, 300.0, 0, now);
        assert!(!Arc::ptr_eq(&wide, &narrow) && !settling);
    }

    #[test]
    fn expensive_documents_wait_for_the_width_to_settle() {
        let (doc, start) = (document(), Instant::now());
        let mut pages = Pages::new(Duration::ZERO);
        let (wide, _) = get(&mut pages, &doc, 500.0, 0, start);
        let (during, settling) = get(&mut pages, &doc, 300.0, 0, start + Duration::from_millis(10));
        assert!(Arc::ptr_eq(&wide, &during) && settling);
        let (still, settling) = get(&mut pages, &doc, 300.0, 0, start + Duration::from_millis(100));
        assert!(Arc::ptr_eq(&wide, &still) && settling);
        let (after, settling) = get(&mut pages, &doc, 300.0, 0, start + SETTLE + Duration::from_millis(20));
        assert!(!Arc::ptr_eq(&wide, &after) && !settling);
    }
}
