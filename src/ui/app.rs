//! The application: hands events to the session reducer, runs its effects,
//! and turns session state into a window. No browsing logic lives here.

use std::cell::{Cell, RefCell};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant};

use iced::keyboard::{self, Key, key::Named};
use iced::widget::operation::{self, AbsoluteOffset, RelativeOffset};
use iced::widget::{column, container, responsive, row, scrollable, text};
use iced::{Element, Fill, Font, Subscription, Task, window};

use super::document::Document;
use super::pages::{Pages, SETTLE};
use super::{body, toolbar};
use crate::content;
use crate::model::Document as Doc;
use crate::services::store::Store;
use crate::services::{background, files, runner};
use crate::session::{self, Event, Shown, State, Status};
use crate::theme::{self, fonts};

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
/// ...and the width must have held this long, so a compositor resizing a new
/// window (e.g. tiling it) doesn't move the heading after we've scrolled to it.
const ANCHOR_STABLE: Duration = Duration::from_millis(150);
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
    Resized,
    Relayout,
    ToggleHistory,
    /// Try to scroll to this pending anchor; the count bounds the retries.
    ResolveAnchor(String, u32),
}

pub struct App {
    session: Option<State>,
    /// What the address bar shows; follows the session until the reader types.
    draft: String,
    store: Option<Arc<Store>>,
    hover: Option<String>,
    history_open: bool,
    pages: RefCell<Pages>,
    /// Last laid-out width, for scrolling to anchors outside `view`.
    width: Cell<f32>,
    settling: Cell<bool>,
    /// The document the view has laid out at its settled width. Weak, so the
    /// allocation (and its address) can't be reused by a later document.
    laid_out: RefCell<Weak<Doc>>,
    /// When the laid-out width last changed.
    width_since: Cell<Instant>,
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
            width: Cell::new(WINDOW_SIZE.0 - SCROLLBAR),
            settling: Cell::new(false),
            laid_out: RefCell::new(Weak::new()),
            width_since: Cell::new(Instant::now()),
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
        Task::batch(effects.into_iter().chain([scroll]))
    }

    /// New documents start at the top; a pending anchor is resolved once the
    /// view has laid the page out at its real width.
    fn scroll_after(&mut self, before: Option<Arc<Shown>>, fragment_before: Option<String>) -> Task<Message> {
        let Some(shown) = self.shown() else { return Task::none() };
        let is_new = !before.is_some_and(|before| Arc::ptr_eq(&before, &shown));
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
        let laid_out = self.laid_out.borrow().upgrade().is_some_and(|doc| Arc::ptr_eq(&doc, &shown.document));
        if !laid_out || self.width_since.get().elapsed() < ANCHOR_STABLE {
            return if attempt < ANCHOR_ATTEMPTS {
                Task::perform(background::after(ANCHOR_RETRY), move |_| Message::ResolveAnchor(fragment, attempt + 1))
            } else {
                self.dispatch(Event::FragmentHandled)
            };
        }
        let font = self.session().settings.font;
        let (page, _) = self.pages.borrow_mut().get(&shown.document, font, self.width.get(), Instant::now());
        let scroll = match page.scroll_offset_for(&fragment, theme::reading(font).gutter) {
            Some(y) => operation::scroll_to(SCROLL_ID, AbsoluteOffset { x: None, y: Some(y) }),
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
            Message::Resized => self.schedule_relayout(),
            Message::Relayout => {
                self.relayout_scheduled = false;
                if self.settling.get() { self.schedule_relayout() } else { Task::none() }
            }
            Message::ToggleHistory => {
                self.history_open = !self.history_open;
                Task::none()
            }
            Message::ResolveAnchor(fragment, attempt) => self.resolve_anchor(fragment, attempt),
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
        match key.as_ref() {
            Key::Named(Named::ArrowLeft) if modifiers.alt() => self.dispatch(Event::Back),
            Key::Named(Named::ArrowRight) if modifiers.alt() => self.dispatch(Event::Forward),
            Key::Named(Named::F5) => self.dispatch(Event::Reload),
            Key::Character("r") if modifiers.command() => self.dispatch(Event::Reload),
            Key::Character("l") if modifiers.command() => {
                Task::batch([operation::focus(ADDRESS_ID), operation::select_all(ADDRESS_ID)])
            }
            Key::Character("h") if modifiers.command() => self.update(Message::ToggleHistory),
            _ => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([keyboard::listen().map(Message::Key), window::resize_events().map(|_| Message::Resized)])
    }

    fn title(&self) -> String {
        match &self.session().status {
            Status::Ready(shown) => format!("{} — {}", shown.title, content::APP_NAME),
            _ => content::APP_NAME.into(),
        }
    }

    fn document_view(&self, shown: Arc<Shown>) -> Element<'_, Message> {
        let font = self.session().settings.font;
        responsive(move |size| {
            let width = size.width - SCROLLBAR;
            if (self.width.get() - width).abs() >= 0.5 {
                self.width_since.set(Instant::now());
            }
            self.width.set(width);
            let (page, settling) = self.pages.borrow_mut().get(&shown.document, font, width, Instant::now());
            self.settling.set(settling);
            if !settling {
                *self.laid_out.borrow_mut() = Arc::downgrade(&shown.document);
            }
            let document =
                Document::new(page, theme::reading(font), |href| Message::Session(Event::Follow(href)), Message::Hover);
            scrollable(document).id(SCROLL_ID).into()
        })
        .into()
    }

    fn view(&self) -> Element<'_, Message> {
        let state = self.session();
        let reader = theme::reading(state.settings.font);
        let main: Element<'_, Message> = match &state.status {
            Status::Idle => body::message(content::WELCOME),
            Status::Loading => body::message(content::LOADING),
            Status::Failed(code) => body::error(*code, state.address(), &reader),
            Status::Ready(shown) => self.document_view(shown.clone()),
        };
        let background = reader.background;
        let page = container(main)
            .width(Fill)
            .height(Fill)
            .style(move |_| container::Style { background: Some(background.into()), ..container::Style::default() });
        let content: Element<'_, Message> =
            if self.history_open { row![page, super::history::panel(state)].into() } else { page.into() };
        let status = self.hover.clone().unwrap_or_default();
        column![
            toolbar::toolbar(state, &self.draft),
            body::notices(&state.notices),
            content,
            container(text(status).size(12)).padding([2, 8]),
        ]
        .into()
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
