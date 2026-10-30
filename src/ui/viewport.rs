//! Where the reader is in the document view: the laid-out width and whether it has
//! settled, the scroll offset, the anchor being held, and which image generation
//! the drawn page shows. Cells, because `view` records what it laid out.

use std::cell::{Cell, RefCell};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use super::keys::Scroll;
use crate::layout::{Page, anchor};
use crate::model::Document;

/// An anchor waits until the width has held this long, so a compositor resizing a
/// new window (e.g. tiling it) doesn't move the heading after we've scrolled to it.
const ANCHOR_STABLE: Duration = Duration::from_millis(150);

/// A page-length scroll keeps this many lines of the last screen in view.
const PAGE_OVERLAP_LINES: f32 = 3.0;

/// A revealed stop stays this far from the edge of the view.
const REVEAL_MARGIN: f32 = 24.0;

pub struct Viewport {
    /// Last laid-out height of the view.
    height: Cell<f32>,
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
    pub fn new(width: f32, height: f32) -> Self {
        Viewport {
            height: Cell::new(height),
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

    /// `view` was given a window of this height.
    pub fn sized(&self, height: f32) {
        self.height.set(height);
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

    /// Where to scroll so the page-space span `top` to `bottom` can be seen: `None` if it already
    /// is, the centre of the view if it was off screen, and its top if it is taller than the view.
    pub fn reveal(&mut self, top: f32, bottom: f32) -> Option<f32> {
        let target = reveal_target(self.scroll_y.get(), self.height.get(), top, bottom)?;
        self.move_to(target)
    }

    /// Scrolls to `target` unless the view is already there (within half a pixel).
    fn move_to(&self, target: f32) -> Option<f32> {
        if (target - self.scroll_y.get()).abs() < 0.5 {
            return None;
        }
        self.scroll_y.set(target);
        Some(target)
    }

    /// Where a key scroll takes the view, on a page `content` tall with lines `line` apart: a
    /// screen less an overlap, a line, or an end. Never past either end; `None` if already there.
    pub fn key_scroll(&mut self, by: Scroll, content: f32, line: f32) -> Option<f32> {
        let (scroll, height) = (self.scroll_y.get(), self.height.get());
        let page = page_step(height, line);
        let target = match by {
            Scroll::PageDown => scroll + page,
            Scroll::PageUp => scroll - page,
            Scroll::LineDown => scroll + line,
            Scroll::LineUp => scroll - line,
            Scroll::Top => 0.0,
            Scroll::Bottom => f32::INFINITY,
        };
        self.move_to(target.clamp(0.0, (content - height).max(0.0)))
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
        self.relaid(before, after, gutter)
    }

    /// The page moved from `before` to `after` without new images (the reader changed size or
    /// spacing). Where to scroll so a held anchor stays at the top, or else the text in view
    /// stays put; `None` if the view is already there.
    pub fn relaid(&mut self, before: &Page, after: &Page, gutter: f32) -> Option<f32> {
        let scroll = self.scroll_y.get();
        let held = self.anchor_held.take().filter(|(_, at)| (at - scroll).abs() < 1.0);
        let target = match held.and_then(|(slug, _)| after.scroll_offset_for(&slug, gutter).map(|y| (slug, y))) {
            Some((slug, y)) => {
                self.anchor_held = Some((slug, y));
                y
            }
            None => anchor::keep(before, after, scroll),
        };
        self.move_to(target)
    }
}

/// How far a page key moves a view `height` tall on lines `line` apart: a screen less an overlap
/// of three lines, and never less than a line.
pub fn page_step(height: f32, line: f32) -> f32 {
    (height - PAGE_OVERLAP_LINES * line).max(line)
}

/// Where a view `height` tall, scrolled to `scroll`, should go to show the span `top` to
/// `bottom` (the same space as `scroll`): `None` if it is already in view with a margin, the
/// span's top if it is taller than the view, else the view centred on it. Never above 0.
pub fn reveal_target(scroll: f32, height: f32, top: f32, bottom: f32) -> Option<f32> {
    let target = if bottom - top > height - 2.0 * REVEAL_MARGIN {
        top - REVEAL_MARGIN
    } else if top >= scroll + REVEAL_MARGIN && bottom <= scroll + height - REVEAL_MARGIN {
        return None;
    } else {
        (top + bottom) / 2.0 - height / 2.0
    };
    Some(target.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::{self, ImageView, layout};
    use crate::model::parse;
    use crate::theme::{Look, TextSize, fonts, reading};

    const POST: &str = "![a](a.png)\n\n# Intro\n\nText.\n\n# Later\n\nMore.";

    fn page(height: u32) -> Page {
        fonts::install();
        let view = move |_: &str| ImageView::Shown { width: 100, height };
        layout(&parse(POST).blocks, &reading(Look::default()), 600.0, &view)
    }

    fn scrolled_to(y: f32) -> Viewport {
        let mut viewport = Viewport::new(600.0, 600.0);
        viewport.scrolled(y);
        viewport
    }

    // A view 600 tall with lines 30 apart: a page is 600 - 90.
    const LINE: f32 = 30.0;

    #[test]
    fn a_page_key_moves_a_screen_less_three_lines() {
        assert_eq!(scrolled_to(0.0).key_scroll(Scroll::PageDown, 5000.0, LINE), Some(510.0));
        assert_eq!(scrolled_to(1000.0).key_scroll(Scroll::PageUp, 5000.0, LINE), Some(490.0));
    }

    #[test]
    fn a_page_step_is_the_screen_less_three_lines_and_at_least_a_line() {
        assert_eq!(page_step(600.0, LINE), 510.0);
        assert_eq!(page_step(60.0, LINE), LINE);
    }

    #[test]
    fn an_arrow_moves_a_line_and_the_ends_jump() {
        assert_eq!(scrolled_to(100.0).key_scroll(Scroll::LineDown, 5000.0, LINE), Some(130.0));
        assert_eq!(scrolled_to(100.0).key_scroll(Scroll::LineUp, 5000.0, LINE), Some(70.0));
        assert_eq!(scrolled_to(100.0).key_scroll(Scroll::Top, 5000.0, LINE), Some(0.0));
        assert_eq!(scrolled_to(100.0).key_scroll(Scroll::Bottom, 5000.0, LINE), Some(4400.0));
    }

    #[test]
    fn key_scrolling_stops_at_both_ends() {
        assert_eq!(scrolled_to(0.0).key_scroll(Scroll::PageUp, 5000.0, LINE), None);
        assert_eq!(scrolled_to(0.0).key_scroll(Scroll::LineUp, 5000.0, LINE), None);
        assert_eq!(scrolled_to(4400.0).key_scroll(Scroll::PageDown, 5000.0, LINE), None);
        assert_eq!(
            scrolled_to(4300.0).key_scroll(Scroll::PageDown, 5000.0, LINE),
            Some(4400.0),
            "the last screen is short"
        );
    }

    #[test]
    fn a_page_that_fits_the_window_does_not_move() {
        for by in [Scroll::PageDown, Scroll::LineDown, Scroll::Bottom, Scroll::PageUp, Scroll::Top] {
            assert_eq!(scrolled_to(0.0).key_scroll(by, 400.0, LINE), None, "{by:?}");
        }
    }

    #[test]
    fn a_page_is_never_less_than_a_line_in_a_tiny_window() {
        let mut viewport = Viewport::new(600.0, 60.0);
        assert_eq!(viewport.key_scroll(Scroll::PageDown, 5000.0, LINE), Some(LINE));
    }

    /// A page whose "later" heading is held, then the view moved by `by` and followed: the
    /// anchor must be let go, so images arriving don't pull the view back to the heading.
    fn assert_anchor_let_go(by: impl FnOnce(&mut Viewport) -> Option<f32>) {
        let (before, after) = (page(10), page(300));
        let gutter = reading(Look::default()).gutter;
        let mut viewport = Viewport::new(600.0, 600.0);
        viewport.hold_anchor("later".into(), before.scroll_offset_for("later", gutter).unwrap());
        let target = by(&mut viewport).unwrap();
        viewport.scrolled(target);
        let pulled = after.scroll_offset_for("later", gutter);
        let arrived = viewport.images_arrived(&before, &after, gutter, 1);
        assert_ne!(arrived, pulled, "not pulled back to the anchor");
        assert_eq!(arrived, Some(anchor::keep(&before, &after, target)).filter(|y| (y - target).abs() >= 0.5));
    }

    #[test]
    fn a_key_scroll_lets_go_of_a_held_anchor_once_the_view_has_moved() {
        assert_anchor_let_go(|viewport| viewport.key_scroll(Scroll::PageDown, 5000.0, LINE));
    }

    #[test]
    fn a_stop_already_in_view_stays_put() {
        assert_eq!(scrolled_to(100.0).reveal(200.0, 220.0), None);
    }

    #[test]
    fn a_stop_below_or_above_the_view_is_brought_to_its_centre() {
        assert_eq!(scrolled_to(0.0).reveal(1000.0, 1020.0), Some(710.0));
        assert_eq!(scrolled_to(900.0).reveal(500.0, 520.0), Some(210.0));
    }

    #[test]
    fn a_stop_near_the_top_cannot_scroll_above_the_page() {
        assert_eq!(scrolled_to(500.0).reveal(50.0, 70.0), Some(0.0));
    }

    #[test]
    fn a_stop_hard_against_an_edge_is_moved_in_to_the_centre() {
        assert_eq!(scrolled_to(100.0).reveal(105.0, 125.0), Some(0.0), "too close to the top");
        assert_eq!(scrolled_to(100.0).reveal(670.0, 695.0), Some(382.5), "too close to the bottom");
    }

    #[test]
    fn the_margin_is_where_a_stop_starts_to_count_as_hard_against_the_edge() {
        let at_top = 100.0 + REVEAL_MARGIN;
        assert_eq!(scrolled_to(100.0).reveal(at_top, at_top + 20.0), None, "exactly the margin from the top");
        assert!(scrolled_to(100.0).reveal(at_top - 1.0, at_top + 19.0).is_some());
        let bottom = 100.0 + 600.0 - REVEAL_MARGIN;
        assert_eq!(scrolled_to(100.0).reveal(bottom - 20.0, bottom), None, "exactly the margin from the bottom");
        assert!(scrolled_to(100.0).reveal(bottom - 19.0, bottom + 1.0).is_some());
    }

    #[test]
    fn a_stop_just_short_of_the_view_height_is_centred_not_top_aligned() {
        let fits = 600.0 - 2.0 * REVEAL_MARGIN;
        assert_eq!(reveal_target(0.0, 600.0, 1000.0, 1000.0 + fits), Some(1000.0 + fits / 2.0 - 300.0));
        assert_eq!(reveal_target(0.0, 600.0, 1000.0, 1000.0 + fits + 1.0), Some(1000.0 - REVEAL_MARGIN));
    }

    #[test]
    fn a_stop_taller_than_the_view_shows_its_top() {
        let mut viewport = scrolled_to(0.0);
        assert_eq!(viewport.reveal(1000.0, 1800.0), Some(976.0));
        assert_eq!(viewport.reveal(1000.0, 1800.0), None, "already there");
    }

    #[test]
    fn a_reveal_lets_go_of_a_held_anchor_when_the_view_follows_it() {
        assert_anchor_let_go(|viewport| viewport.reveal(2000.0, 2020.0));
    }

    #[test]
    fn a_held_anchor_follows_its_heading_as_images_arrive() {
        let (before, after) = (page(10), page(300));
        let gutter = reading(Look::default()).gutter;
        let mut viewport = Viewport::new(600.0, 600.0);
        let at = before.scroll_offset_for("later", gutter).unwrap();
        viewport.hold_anchor("later".into(), at);
        let target = viewport.images_arrived(&before, &after, gutter, 1);
        assert_eq!(target, after.scroll_offset_for("later", gutter));
        assert_eq!(viewport.applied_images(), 1);
    }

    #[test]
    fn a_held_anchor_survives_a_change_of_text_size() {
        fonts::install();
        let at = |size| {
            let look = Look { size, ..Look::default() };
            (layout(&parse(POST).blocks, &reading(look), 600.0, &layout::no_images), reading(look).gutter)
        };
        let ((before, gutter), (after, gutter_after)) = (at(TextSize::default()), at(TextSize::default().larger()));
        assert_ne!(gutter, gutter_after);
        let mut viewport = Viewport::new(600.0, 600.0);
        viewport.hold_anchor("later".into(), before.scroll_offset_for("later", gutter).unwrap());
        let target = viewport.images_arrived(&before, &after, gutter_after, 0);
        assert_eq!(target, after.scroll_offset_for("later", gutter_after));
    }

    #[test]
    fn scrolling_away_lets_go_of_the_anchor() {
        let (before, after) = (page(10), page(300));
        let gutter = reading(Look::default()).gutter;
        let mut viewport = Viewport::new(600.0, 600.0);
        viewport.hold_anchor("later".into(), before.scroll_offset_for("later", gutter).unwrap());
        viewport.scrolled(0.0);
        assert_eq!(viewport.images_arrived(&before, &after, gutter, 1), None, "at the top, the top stays put");
    }

    #[test]
    fn image_relayouts_are_scheduled_one_at_a_time() {
        let mut viewport = Viewport::new(600.0, 600.0);
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
        let mut viewport = Viewport::new(600.0, 600.0);
        viewport.hold_anchor("x".into(), 400.0);
        viewport.new_page(7);
        assert_eq!(viewport.applied_images(), 7);
        let (before, after) = (page(10), page(300));
        assert_eq!(viewport.images_arrived(&before, &after, 0.0, 8), None);
    }
}
