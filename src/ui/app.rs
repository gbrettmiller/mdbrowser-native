//! The application: hands events to the session reducer, runs its effects,
//! and turns session state into a window. No browsing logic lives here.

use std::cell::RefCell;
use std::sync::Arc;
use std::time::Duration;

use iced::keyboard;
use iced::widget::operation;
use iced::{Font, Subscription, Task, window};

use super::keys::{self, Command, Scroll};
use super::pages::{Pages, SETTLE};
use super::settings::ColourDrafts;
use super::viewport::Viewport;
use crate::services::store::Store;
use crate::services::{background, files, runner};
use crate::session::{self, Event, Shown, State, Status};
use crate::theme::{self, Palette, Which, fonts};

mod scroll;
mod view;

pub const ADDRESS_ID: &str = "address";
const SCROLL_ID: &str = "document";
/// The page itself, as a stop in the focus order.
const PAGE_ID: &str = "page";
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
    Resized,
    Relayout,
    ToggleHistory,
    ToggleSettings,
    HomeDraftChanged(String),
    SubmitHome,
    ColourDraftChanged(Which, String),
    SubmitColour(Which),
    /// The OS reported its light or dark mode.
    SystemMode(iced::theme::Mode),
    /// Try to scroll to this pending anchor; the count bounds the retries.
    ResolveAnchor(String, u32),
    /// Images arrived since the last layout: lay out again, keeping the reader's place.
    ImagesArrived,
    /// The document was scrolled to this offset.
    Scrolled(f32),
    /// Focus moved to a stop spanning these heights of the page: scroll it into view.
    Reveal(f32, f32),
    /// A key asked to scroll the page.
    Scroll(Scroll),
}

pub struct App {
    session: Option<State>,
    /// What the address bar shows; follows the session until the reader types.
    draft: String,
    store: Option<Arc<Store>>,
    hover: Option<String>,
    history_open: bool,
    settings_open: bool,
    /// What the Home page field shows; follows the saved value until the reader types.
    home_draft: String,
    /// What the custom colour fields show; follows the saved colours until the reader types.
    colour_drafts: ColourDrafts,
    pages: RefCell<Pages>,
    viewport: Viewport,
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
            settings_open: false,
            home_draft: settings.home.clone(),
            colour_drafts: ColourDrafts::of(settings.colours()),
            pages: RefCell::new(Pages::new(EXPENSIVE_LAYOUT)),
            viewport: Viewport::new(WINDOW_SIZE.0 - SCROLLBAR, WINDOW_SIZE.1),
            relayout_scheduled: false,
        };
        let restore = app.dispatch(Event::Restore { visits, settings });
        let argument = std::env::args().nth(1).map(|argument| files::from_command_line(&argument));
        let focus = if argument.is_none() { operation::focus(ADDRESS_ID) } else { Task::none() };
        let open = Task::batch([app.dispatch(Event::Launch(argument)), focus]);
        let system = iced::system::theme().map(Message::SystemMode);
        (app, Task::batch([restore, open, system]))
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

    /// The palette in use: the reader's choice, or the OS's for System.
    pub(super) fn palette(&self) -> Palette {
        self.session().palette()
    }

    /// How pages look: the reader's font, size and spacing, in the palette in use.
    pub(super) fn look(&self) -> theme::Look {
        let settings = &self.session().settings;
        theme::Look {
            font: settings.font,
            palette: self.palette(),
            size: settings.text_size,
            spacing: settings.spacing,
        }
    }

    /// Steps the reducer, runs its effects, and keeps the view in step with it.
    fn dispatch(&mut self, event: Event) -> Task<Message> {
        let before = self.shown();
        let fragment_before = self.session().fragment.clone();
        let previous_address = self.session().address();
        let home_before = self.session().settings.home.clone();
        let colours_before = self.session().settings.colours();
        let look_before = self.look();
        let page_before = if may_change_look(&event) {
            self.shown().map(|shown| self.page(&shown.document, self.viewport.width()).0)
        } else {
            None
        };
        let (state, effects) = session::step(self.session.take().expect("session"), event);
        self.session = Some(state);
        if self.session().address() != previous_address {
            self.draft = self.session().address();
        }
        if self.session().settings.home != home_before {
            self.home_draft = self.session().settings.home.clone();
        }
        if self.session().settings.colours() != colours_before {
            self.colour_drafts = ColourDrafts::of(self.session().settings.colours());
        }
        let store = self.store.clone();
        let effects: Vec<_> =
            effects.into_iter().map(|effect| runner::run(effect, store.clone()).map(Message::Effect)).collect();
        let scroll = self.scroll_after(before, fragment_before);
        let images = self.schedule_images();
        let kept = self.keep_place(look_before, page_before);
        Task::batch(effects.into_iter().chain([scroll, images, kept]))
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
                if self.viewport.settling() { self.schedule_relayout() } else { Task::none() }
            }
            Message::ToggleHistory => {
                self.history_open = !self.history_open;
                Task::none()
            }
            Message::ToggleSettings => {
                self.settings_open = !self.settings_open;
                Task::none()
            }
            Message::HomeDraftChanged(draft) => {
                self.home_draft = draft;
                Task::none()
            }
            Message::ColourDraftChanged(which, typed) => {
                self.colour_drafts = self.colour_drafts.clone().typed(which, typed);
                Task::none()
            }
            Message::SubmitColour(which) => {
                self.dispatch(Event::ChooseCustomColour(which, self.colour_drafts.get(which).to_string()))
            }
            Message::SubmitHome => self.dispatch(Event::ChooseHome(self.home_draft.clone())),
            Message::SystemMode(mode) => self.dispatch(Event::SystemPalette(match mode {
                iced::theme::Mode::None => None,
                iced::theme::Mode::Light => Some(Palette::Light),
                iced::theme::Mode::Dark => Some(Palette::Dark),
            })),
            Message::ResolveAnchor(fragment, attempt) => self.resolve_anchor(fragment, attempt),
            Message::ImagesArrived => self.images_arrived(),
            Message::Reveal(top, bottom) => self.reveal(top, bottom),
            Message::Scroll(by) => self.key_scroll(by),
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
            Some(Command::Scroll(by)) => self.key_scroll(by),
            Some(Command::ClosePanel) => self.close_panel(),
            Some(Command::FocusNext) => operation::focus_next(),
            Some(Command::FocusPrevious) => operation::focus_previous(),
            Some(Command::ToggleHistory) => self.update(Message::ToggleHistory),
            Some(Command::ToggleSettings) => self.update(Message::ToggleSettings),
            None => Task::none(),
        }
    }

    /// Esc: closes the Settings panel, or else History, and puts the keyboard back on the page.
    fn close_panel(&mut self) -> Task<Message> {
        let Some((history, settings)) = after_escape(self.history_open, self.settings_open) else {
            return Task::none();
        };
        (self.history_open, self.settings_open) = (history, settings);
        operation::focus(PAGE_ID)
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::listen().map(Message::Key),
            window::resize_events().map(|_| Message::Resized),
            iced::system::theme_changes().map(Message::SystemMode),
        ])
    }
}

pub fn run() -> iced::Result {
    fonts::install();
    iced::application(App::boot, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .default_font(Font::with_name(theme::ReadingFont::Inter.family()))
        .theme(|app: &App| super::chrome::theme(app.palette()))
        .window_size(WINDOW_SIZE)
        .run()
}

/// Whether an event can change how pages look, which is when the page as it was is needed
/// to keep the reader's place.
const fn may_change_look(event: &Event) -> bool {
    matches!(
        event,
        Event::ChooseFont(_)
            | Event::ChooseAppearance(_)
            | Event::ChooseCustomColour(..)
            | Event::SystemPalette(_)
            | Event::StepTextSize(_)
            | Event::ChooseSpacing(_)
            | Event::ResetReaderControls
            | Event::Restore { .. }
    )
}

/// The panels that stay open after Esc: Settings goes first, then History. `None` if none was open.
const fn after_escape(history: bool, settings: bool) -> Option<(bool, bool)> {
    match (history, settings) {
        (false, false) => None,
        (_, true) => Some((history, false)),
        (true, false) => Some((false, false)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_closes_settings_first_then_history_and_does_nothing_with_none_open() {
        assert_eq!(after_escape(false, false), None);
        assert_eq!(after_escape(true, true), Some((true, false)));
        assert_eq!(after_escape(false, true), Some((false, false)));
        assert_eq!(after_escape(true, false), Some((false, false)));
    }
}
