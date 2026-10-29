//! The browsing session as a pure reducer: `step(state, event) → (state, effects)`.
//! Effects are data; `services::runner` executes them and feeds results back as events.

pub mod links;
pub mod location;
pub mod navigation;
mod reducer;
pub mod visits;

pub use reducer::step;

use std::path::PathBuf;
use std::sync::Arc;

use crate::model::Document;
use crate::theme::ReadingFont;
use location::{ErrorCode, Location};
use navigation::Navigation;
use visits::Visit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Settings {
    pub font: ReadingFont,
}

/// A document ready to show.
#[derive(Debug, PartialEq)]
pub struct Shown {
    pub document: Document,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Idle,
    Loading,
    Ready(Arc<Shown>),
    Failed(ErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeKind {
    OpenFailed,
    UnsupportedLink,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notice {
    pub id: u64,
    pub kind: NoticeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Enter(String),
    Follow(String),
    Back,
    Forward,
    Reload,
    Loaded { request_id: u64, result: Result<String, ErrorCode>, now: u64 },
    OpenerFailed,
    OpenHistory(String),
    ClearHistory,
    ChooseFont(ReadingFont),
    DismissNotice(u64),
    FragmentHandled,
    Restore { visits: Vec<Visit>, settings: Settings },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Load { request_id: u64, location: Location },
    OpenExternal(String),
    PersistVisits(Vec<Visit>),
    PersistSettings(Settings),
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub home: PathBuf,
    pub nav: Navigation,
    pub request_id: u64,
    pub pending: Option<u64>,
    pub status: Status,
    /// Anchor waiting to be scrolled to once the page is laid out.
    pub fragment: Option<String>,
    pub visits: Vec<Visit>,
    pub notices: Vec<Notice>,
    pub notice_seq: u64,
    pub settings: Settings,
}

impl State {
    pub fn new(home: impl Into<PathBuf>) -> Self {
        State {
            home: home.into(),
            nav: Navigation::default(),
            request_id: 0,
            pending: None,
            status: Status::Idle,
            fragment: None,
            visits: Vec::new(),
            notices: Vec::new(),
            notice_seq: 0,
            settings: Settings::default(),
        }
    }

    /// What the address bar shows.
    pub fn address(&self) -> String {
        self.nav.current().map(|entry| entry.input.clone()).unwrap_or_default()
    }

    pub fn current_location(&self) -> Option<&Location> {
        self.nav.current()?.target.as_ref().map(|target| &target.location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(events: impl IntoIterator<Item = Event>) -> (State, Vec<Effect>) {
        events.into_iter().fold((State::new("/home/r"), Vec::new()), |(state, _), event| step(state, event))
    }

    fn loaded(request_id: u64, source: &str) -> Event {
        Event::Loaded { request_id, result: Ok(source.into()), now: 1 }
    }

    #[test]
    fn entering_an_address_requests_a_load_and_waits() {
        let (state, effects) = run([Event::Enter("/d/a.md".into())]);
        assert_eq!(state.status, Status::Loading);
        assert_eq!(effects, [Effect::Load { request_id: 1, location: Location::file("/d/a.md") }]);
    }

    #[test]
    fn a_load_shows_the_document_and_records_history() {
        let (state, effects) = run([Event::Enter("/d/a.md#intro".into()), loaded(1, "# A")]);
        let Status::Ready(shown) = &state.status else { panic!("{:?}", state.status) };
        assert_eq!(shown.title, "A");
        assert_eq!(state.fragment.as_deref(), Some("intro"));
        assert!(matches!(&effects[..], [Effect::PersistVisits(visits)] if visits[0].address == "/d/a.md"));
    }

    #[test]
    fn stale_loads_are_ignored() {
        let (state, effects) = run([Event::Enter("/d/a.md".into()), Event::Enter("/d/b.md".into()), loaded(1, "# A")]);
        assert_eq!(state.status, Status::Loading);
        assert!(effects.is_empty());
    }

    #[test]
    fn links_do_nothing_until_a_document_is_showing() {
        let (state, effects) = run([Event::Enter("/d/a.md".into()), Event::Follow("https://x.org".into())]);
        assert!(effects.is_empty() && state.notices.is_empty());
    }

    #[test]
    fn notices_accumulate_and_dismiss_by_id() {
        let (state, _) = run([Event::OpenerFailed, Event::OpenerFailed, Event::DismissNotice(1)]);
        assert_eq!(state.notices, [Notice { id: 2, kind: NoticeKind::OpenFailed }]);
    }

    #[test]
    fn choosing_a_font_persists_it_and_restore_brings_it_back() {
        let (state, effects) = run([Event::ChooseFont(ReadingFont::Inter)]);
        let settings = Settings { font: ReadingFont::Inter };
        assert_eq!((state.settings, effects), (settings, vec![Effect::PersistSettings(settings)]));
        let (restored, _) = run([Event::Restore { visits: vec![], settings }]);
        assert_eq!(restored.settings.font, ReadingFont::Inter);
    }
}
