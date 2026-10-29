//! The application: hands events to the session reducer, runs its effects,
//! and turns session state into a window. No browsing logic lives here.

use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, Instant};

use iced::keyboard;
use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};
use iced::{Font, Subscription, Task, window};

use super::keys::{self, Command};
use super::pages::{Pages, SETTLE};
use super::viewport::Viewport;
use crate::model::Document as Doc;
use crate::services::store::Store;
use crate::services::{background, files, runner};
use crate::session::{self, Event, Shown, State, Status};
use crate::theme::{self, fonts};

mod view;

pub const ADDRESS_ID: &str = "address";
const SCROLL_ID: &str = "document";
const SCROLLBAR: f32 = 12.0;
/// Layouts slower than this wait for a resize to settle.
const EXPENSIVE_LAYOUT: Duration = Duration::from_millis(32);
/// Slack after `SETTLE` before re-checking a resize.
const RELAYOUT_SLACK: Duration = Duration::from_millis(20);
/// An anchor waits for the page to be laid out at its real width: this often, at most this many times.
const ANCHOR_RETRY: Duration = Duration::from_millis(16);
const ANCHOR_ATTEMPTS: u32 = 60;
/// Window size at launch, and the layout width assumed before the first frame.
const WINDOW_SIZE: (f32, f32) = (1000.0, 1000.0);

#[derive(Debug, Clone)]
pub enum Message {
    /// Straight to the reducer.
    Session(Event),
    /// An effect finished; some report back.
    Effect(Option<Event>),
    AddressChanged(String),
    Submit,
    Hover(Option<String>),
    Key(keyboard::Event),
    Resized(f32),
    Relayout,
    ToggleHistory,
    /// Try to scroll to this pending anchor; the count bounds the retries.
    ResolveAnchor(String, u32),
    /// Images arrived since the last layout: lay out again, keeping the reader's place.
    ImagesArrived,
    /// The document was scrolled to this offset.
    Scrolled(f32),
}

pub struct App {
    session: Option<State>,
    /// What the address bar shows; follows the session until the reader types.
    draft: String,
    store: Option<Arc<Store>>,
    hover: Option<String>,
    history_open: bool,
    pages: RefCell<Pages>,
    viewport: Viewport,
    /// The window's width, for the toolbar's layout.
    window_width: f32,
    relayout_scheduled: bool,
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let store = Store::default_location().map(Arc::new);
        let visits = store.as_ref().map(|store| store.load_visits()).unwrap_or_default();
        let settings = store.as_ref().map(|store| store.load_settings()).unwrap_or_default();
        let home = dirs::home_dir().unwrap_or_else(|| "/".into());
        let mut app = App {
            session: Some(State::new(home)),
            draft: String::new(),
            store,
            hover: None,
            history_open: false,
            pages: RefCell::new(Pages::new(EXPENSIVE_LAYOUT)),
            viewport: Viewport::new(WINDOW_SIZE.0 - SCROLLBAR),
            window_width: WINDOW_SIZE.0,
            relayout_scheduled: false,
        };
        let restore = app.dispatch(Event::Restore { visits, settings });
        let open = match std::env::args().nth(1) {
            Some(argument) => app.dispatch(Event::Enter(files::from_command_line(&argument))),
            None => operation::focus(ADDRESS_ID),
        };
        (app, Task::batch([restore, open]))
    }

    pub(super) fn session(&self) -> &State {
        self.session.as_ref().expect("session")
    }

    fn shown(&self) -> Option<Arc<Shown>> {
        match &self.session().status {
            Status::Ready(shown) => Some(shown.clone()),
            _ => None,
        }
    }

    /// Steps the reducer, runs its effects, and keeps the view in step with it.
    fn dispatch(&mut self, event: Event) -> Task<Message> {
        let before = self.shown();
        let fragment_before = self.session().fragment.clone();
        let previous_address = self.session().address();
        let (state, effects) = session::step(self.session.take().expect("session"), event);
        self.session = Some(state);
        if self.session().address() != previous_address {
            self.draft = self.session().address();
        }
        let store = self.store.clone();
        let effects: Vec<_> =
            effects.into_iter().map(|effect| runner::run(effect, store.clone()).map(Message::Effect)).collect();
        let scroll = self.scroll_after(before, fragment_before);
        let images = self.schedule_images();
        Task::batch(effects.into_iter().chain([scroll, images]))
    }

    /// Images changed what the page shows: lay out again once arrivals settle, so a
    /// burst of them costs one layout and one scroll correction.
    fn schedule_images(&mut self) -> Task<Message> {
        let changed = self.session().images.generation() != self.viewport.applied_images();
        if !changed || self.viewport.images_pending || self.shown().is_none() {
            return Task::none();
        }
        self.viewport.images_pending = true;
        Task::perform(background::after(SETTLE), |_| Message::ImagesArrived)
    }

    /// The page as laid out at the applied image generation.
    fn page(&self, document: &Arc<Doc>, width: f32) -> (Arc<crate::layout::Page>, bool) {
        self.page_at(document, width, self.viewport.applied_images())
    }

    /// The page as laid out with the images as of `generation`.
    fn page_at(&self, document: &Arc<Doc>, width: f32, generation: u64) -> (Arc<crate::layout::Page>, bool) {
        let images = &self.session().images;
        let view = |src: &str| super::images::view(images, src);
        let font = self.session().settings.font;
        self.pages.borrow_mut().get(document, font, width, (&view, generation), Instant::now())
    }

    /// Lays the page out with the newest images and scrolls so the text in view
    /// stays put, or so a held anchor stays at the top.
    fn images_arrived(&mut self) -> Task<Message> {
        let Some(shown) = self.shown() else {
            self.viewport.images_pending = false;
            return Task::none();
        };
        let width = self.viewport.width();
        let (before, _) = self.page(&shown.document, width);
        let generation = self.session().images.generation();
        let gutter = theme::reading(self.session().settings.font).gutter;
        let (after, _) = self.page_at(&shown.document, width, generation);
        match self.viewport.images_arrived(&before, &after, gutter, generation) {
            Some(y) => operation::scroll_to(SCROLL_ID, AbsoluteOffset { x: None, y: Some(y) }),
            None => Task::none(),
        }
    }

    /// New documents start at the top; a pending anchor is resolved once the
    /// view has laid the page out at its real width.
    fn scroll_after(&mut self, before: Option<Arc<Shown>>, fragment_before: Option<String>) -> Task<Message> {
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

    fn resolve_anchor(&mut self, fragment: String, attempt: u32) -> Task<Message> {
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
        let scroll = match page.scroll_offset_for(&fragment, theme::reading(font).gutter) {
            Some(y) => {
                self.viewport.hold_anchor(fragment.clone(), y);
                operation::scroll_to(SCROLL_ID, AbsoluteOffset { x: None, y: Some(y) })
            }
            None => Task::none(),
        };
        Task::batch([self.dispatch(Event::FragmentHandled), scroll])
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Session(event) | Message::Effect(Some(event)) => self.dispatch(event),
            Message::Effect(None) => Task::none(),
            Message::AddressChanged(draft) => {
                self.draft = draft;
                Task::none()
            }
            Message::Submit => self.dispatch(Event::Enter(self.draft.clone())),
            Message::Hover(target) => {
                self.hover = target;
                Task::none()
            }
            Message::Key(event) => self.shortcut(event),
            Message::Resized(width) => {
                self.window_width = width;
                self.schedule_relayout()
            }
            Message::Relayout => {
                self.relayout_scheduled = false;
                if self.viewport.settling() { self.schedule_relayout() } else { Task::none() }
            }
            Message::ToggleHistory => {
                self.history_open = !self.history_open;
                Task::none()
            }
            Message::ResolveAnchor(fragment, attempt) => self.resolve_anchor(fragment, attempt),
            Message::ImagesArrived => self.images_arrived(),
            Message::Scrolled(y) => {
                self.viewport.scrolled(y);
                Task::none()
            }
        }
    }

    /// Wakes the view once the width has had time to settle.
    fn schedule_relayout(&mut self) -> Task<Message> {
        if self.relayout_scheduled {
            return Task::none();
        }
        self.relayout_scheduled = true;
        Task::perform(background::after(SETTLE + RELAYOUT_SLACK), |_| Message::Relayout)
    }

    fn shortcut(&mut self, event: keyboard::Event) -> Task<Message> {
        let keyboard::Event::KeyPressed { key, modifiers, .. } = event else { return Task::none() };
        match keys::command(key.as_ref(), modifiers) {
            Some(Command::Session(event)) => self.dispatch(event),
            Some(Command::FocusAddress) => {
                Task::batch([operation::focus(ADDRESS_ID), operation::select_all(ADDRESS_ID)])
            }
            Some(Command::ToggleHistory) => self.update(Message::ToggleHistory),
            None => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::listen().map(Message::Key),
            window::resize_events().map(|(_, size)| Message::Resized(size.width)),
        ])
    }
}

pub fn run() -> iced::Result {
    fonts::install();
    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .default_font(Font::with_name(theme::ReadingFont::Inter.family()))
        .theme(|_: &App| iced::Theme::Light)
        .window_size(WINDOW_SIZE)
        .run()
}
