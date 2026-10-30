//! Where the reader is in the document view: the laid-out width and whether it has
//! settled, the scroll offset, the anchor being held, and which image generation
//! the drawn page shows. Cells, because `view` records what it laid out.

use std::cell::{Cell, RefCell};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use crate::layout::{Page, anchor};
use crate::model::Document;

/// An anchor waits until the width has held this long, so a compositor resizing a
/// new window (e.g. tiling it) doesn't move the heading after we've scrolled to it.
const ANCHOR_STABLE: Duration = Duration::from_millis(150);

pub struct Viewport {
    /// Last laid-out width, for scrolling to anchors outside `view`.
    width: Cell<f32>,
    /// When the laid-out width last changed.
    width_since: Cell<Instant>,
    /// Whether the last layout was deferred until a resize settles.
    settling: Cell<bool>,
    /// The document laid out at its settled width. Weak, so the allocation (and
    /// its address) can't be reused by a later document.
    laid_out: RefCell<Weak<Document>>,
    /// The image generation the drawn page was laid out with. Newer images wait
    /// for `ImagesArrived`, which lays out again from `update` so the scroll can follow.
    applied_images: Cell<u64>,
    /// An `ImagesArrived` relayout is scheduled.
    images_pending: bool,
    scroll_y: Cell<f32>,
    /// An anchor the view was scrolled to, and where: re-applied as images arrive,
    /// until the reader scrolls away.
    anchor_held: Option<(String, f32)>,
}

impl Viewport {
    pub fn new(width: f32) -> Self {
        Viewport {
            width: Cell::new(width),
            width_since: Cell::new(Instant::now()),
            settling: Cell::new(false),
            laid_out: RefCell::new(Weak::new()),
            applied_images: Cell::new(0),
            images_pending: false,
            scroll_y: Cell::new(0.0),
            anchor_held: None,
        }
    }

    pub fn width(&self) -> f32 {
        self.width.get()
    }

    pub fn applied_images(&self) -> u64 {
        self.applied_images.get()
    }

    pub fn settling(&self) -> bool {
        self.settling.get()
    }

    /// `view` laid `document` out at `width`; `settling` if it waited on a resize.
    pub fn laid_out(&self, document: &Arc<Document>, width: f32, settling: bool) {
        if (self.width.get() - width).abs() >= 0.5 {
            self.width_since.set(Instant::now());
        }
        self.width.set(width);
        self.settling.set(settling);
        if !settling {
            *self.laid_out.borrow_mut() = Arc::downgrade(document);
        }
    }

    /// `document` is laid out at a width that has held long enough to scroll to an anchor.
    pub fn ready_for_anchor(&self, document: &Arc<Document>) -> bool {
        let laid_out = self.laid_out.borrow().upgrade().is_some_and(|doc| Arc::ptr_eq(&doc, document));
        laid_out && self.width_since.get().elapsed() >= ANCHOR_STABLE
    }

    /// Images changed to `image_generation`: whether to schedule a relayout. One at a
    /// time, so a burst of arrivals costs one layout and one scroll correction.
    pub fn begin_images(&mut self, image_generation: u64) -> bool {
        let start = image_generation != self.applied_images.get() && !self.images_pending;
        self.images_pending |= start;
        start
    }

    /// The scheduled relayout found no page to lay out.
    pub fn cancel_images(&mut self) {
        self.images_pending = false;
    }

    /// A new page, laid out with its images as they are now, from the top.
    pub fn new_page(&mut self, image_generation: u64) {
        self.applied_images.set(image_generation);
        self.anchor_held = None;
        self.scroll_y.set(0.0);
    }

    /// The view was scrolled to `anchor` at `y`: hold it there as images arrive.
    pub fn hold_anchor(&mut self, anchor: String, y: f32) {
        self.anchor_held = Some((anchor, y));
        self.scroll_y.set(y);
    }

    pub fn scrolled(&mut self, y: f32) {
        self.scroll_y.set(y);
        if self.anchor_held.as_ref().is_some_and(|(_, at)| (at - y).abs() >= 1.0) {
            self.anchor_held = None; // the reader scrolled away
        }
    }

    /// Images moved the page from `before` to `after`, laid out at `image_generation`.
    /// Returns where to scroll so a held anchor stays at the top, or else the text
    /// in view stays put; `None` if the view is already there.
    pub fn images_arrived(&mut self, before: &Page, after: &Page, gutter: f32, image_generation: u64) -> Option<f32> {
        self.images_pending = false;
        self.applied_images.set(image_generation);
        let scroll = self.scroll_y.get();
        let held = self.anchor_held.take().filter(|(_, at)| (at - scroll).abs() < 1.0);
        let target = match held.and_then(|(slug, _)| after.scroll_offset_for(&slug, gutter).map(|y| (slug, y))) {
            Some((slug, y)) => {
                self.anchor_held = Some((slug, y));
                y
            }
            None => anchor::keep(before, after, scroll),
        };
        if (target - scroll).abs() < 0.5 {
            return None;
        }
        self.scroll_y.set(target);
        Some(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{ImageView, layout};
    use crate::model::parse;
    use crate::theme::{Palette, ReadingFont, fonts, reading};

    const POST: &str = "![a](a.png)\n\n# Intro\n\nText.\n\n# Later\n\nMore.";

    fn page(height: u32) -> Page {
        fonts::install();
        let view = move |_: &str| ImageView::Shown { width: 100, height };
        layout(&parse(POST).blocks, &reading(ReadingFont::Literata, Palette::Light), 600.0, &view)
    }

    #[test]
    fn a_held_anchor_follows_its_heading_as_images_arrive() {
        let (before, after) = (page(10), page(300));
        let gutter = reading(ReadingFont::Literata, Palette::Light).gutter;
        let mut viewport = Viewport::new(600.0);
        let at = before.scroll_offset_for("later", gutter).unwrap();
        viewport.hold_anchor("later".into(), at);
        let target = viewport.images_arrived(&before, &after, gutter, 1);
        assert_eq!(target, after.scroll_offset_for("later", gutter));
        assert_eq!(viewport.applied_images(), 1);
    }

    #[test]
    fn scrolling_away_lets_go_of_the_anchor() {
        let (before, after) = (page(10), page(300));
        let gutter = reading(ReadingFont::Literata, Palette::Light).gutter;
        let mut viewport = Viewport::new(600.0);
        viewport.hold_anchor("later".into(), before.scroll_offset_for("later", gutter).unwrap());
        viewport.scrolled(0.0);
        assert_eq!(viewport.images_arrived(&before, &after, gutter, 1), None, "at the top, the top stays put");
    }

    #[test]
    fn image_relayouts_are_scheduled_one_at_a_time() {
        let mut viewport = Viewport::new(600.0);
        assert!(!viewport.begin_images(0), "nothing new");
        assert!(viewport.begin_images(1));
        assert!(!viewport.begin_images(2), "one is already scheduled");
        let (before, after) = (page(10), page(10));
        viewport.images_arrived(&before, &after, 0.0, 2);
        assert!(!viewport.begin_images(2), "applied");
        assert!(viewport.begin_images(3));
        viewport.cancel_images();
        assert!(viewport.begin_images(3));
    }

    #[test]
    fn a_new_page_starts_at_the_top_holding_nothing() {
        let mut viewport = Viewport::new(600.0);
        viewport.hold_anchor("x".into(), 400.0);
        viewport.new_page(7);
        assert_eq!(viewport.applied_images(), 7);
        let (before, after) = (page(10), page(300));
        assert_eq!(viewport.images_arrived(&before, &after, 0.0, 8), None);
    }
}
