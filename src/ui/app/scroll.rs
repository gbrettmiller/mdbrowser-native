//! Scroll and layout upkeep: images arriving, anchors, and keeping the reader's
//! place when the page is laid out again.

use std::sync::Arc;
use std::time::Instant;

use iced::Task;
use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};

use super::{ANCHOR_ATTEMPTS, ANCHOR_RETRY, App, Message, SCROLL_ID};
use crate::model::Document as Doc;
use crate::services::background;
use crate::session::{Event, Shown};
use crate::theme;
use crate::ui::images;
use crate::ui::pages::SETTLE;

impl App {
    /// Images changed what the page shows: lay out again once arrivals settle, so a
    /// burst of them costs one layout and one scroll correction.
    pub(super) fn schedule_images(&mut self) -> Task<Message> {
        let generation = self.session().images.generation();
        if self.shown().is_none() || !self.viewport.begin_images(generation) {
            return Task::none();
        }
        Task::perform(background::after(SETTLE), |_| Message::ImagesArrived)
    }

    /// The page as laid out at the applied image generation.
    pub(super) fn page(&self, document: &Arc<Doc>, width: f32) -> (Arc<crate::layout::Page>, bool) {
        self.page_at(document, width, self.viewport.applied_images())
    }

    /// The page as laid out with the images as of `generation`.
    fn page_at(&self, document: &Arc<Doc>, width: f32, generation: u64) -> (Arc<crate::layout::Page>, bool) {
        let images = &self.session().images;
        let view = |src: &str| images::view(images, src);
        let look = (self.session().settings.font, self.palette());
        self.pages.borrow_mut().get(document, look, width, (&view, generation), Instant::now())
    }

    /// Lays the page out with the newest images and scrolls so the text in view
    /// stays put, or so a held anchor stays at the top.
    pub(super) fn images_arrived(&mut self) -> Task<Message> {
        let Some(shown) = self.shown() else {
            self.viewport.cancel_images();
            return Task::none();
        };
        let width = self.viewport.width();
        let (before, _) = self.page(&shown.document, width);
        let generation = self.session().images.generation();
        let gutter = theme::reading(self.session().settings.font, self.palette()).gutter;
        let (after, _) = self.page_at(&shown.document, width, generation);
        match self.viewport.images_arrived(&before, &after, gutter, generation) {
            Some(y) => operation::scroll_to(SCROLL_ID, AbsoluteOffset { x: None, y: Some(y) }),
            None => Task::none(),
        }
    }

    /// New documents start at the top; a pending anchor is resolved once the
    /// view has laid the page out at its real width.
    pub(super) fn scroll_after(
        &mut self,
        before: Option<Arc<Shown>>,
        fragment_before: Option<String>,
    ) -> Task<Message> {
        let Some(shown) = self.shown() else { return Task::none() };
        let is_new = !before.is_some_and(|before| Arc::ptr_eq(&before, &shown));
        if is_new {
            self.viewport.new_page(self.session().images.generation());
        }
        let to_top = if is_new { operation::snap_to(SCROLL_ID, RelativeOffset::START) } else { Task::none() };
        // Start one resolution per new anchor; other dispatches leave a running one alone.
        match self.session().fragment.clone() {
            Some(fragment) if is_new || Some(&fragment) != fragment_before.as_ref() => {
                Task::batch([to_top, Task::done(Message::ResolveAnchor(fragment, 0))])
            }
            _ => to_top,
        }
    }

    pub(super) fn resolve_anchor(&mut self, fragment: String, attempt: u32) -> Task<Message> {
        let Some(shown) = self.shown() else { return Task::none() };
        if self.session().fragment.as_ref() != Some(&fragment) {
            return Task::none(); // superseded or already handled
        }
        if !self.viewport.ready_for_anchor(&shown.document) {
            return if attempt < ANCHOR_ATTEMPTS {
                Task::perform(background::after(ANCHOR_RETRY), move |_| Message::ResolveAnchor(fragment, attempt + 1))
            } else {
                self.dispatch(Event::FragmentHandled)
            };
        }
        let font = self.session().settings.font;
        let (page, _) = self.page(&shown.document, self.viewport.width());
        let scroll = match page.scroll_offset_for(&fragment, theme::reading(font, self.palette()).gutter) {
            Some(y) => {
                self.viewport.hold_anchor(fragment.clone(), y);
                operation::scroll_to(SCROLL_ID, AbsoluteOffset { x: None, y: Some(y) })
            }
            None => Task::none(),
        };
        Task::batch([self.dispatch(Event::FragmentHandled), scroll])
    }
}
