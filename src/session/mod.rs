//! The browsing session as a pure reducer: `step(state, event) → (state, effects)`.
//! Effects are data; `services::runner` executes them and feeds results back as events.

pub mod links;
pub mod location;
pub mod navigation;
pub mod visits;

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
        todo!()
    }

    pub fn current_location(&self) -> Option<&Location> {
        todo!()
    }
}

pub fn step(_state: State, _event: Event) -> (State, Vec<Effect>) {
    todo!()
}
