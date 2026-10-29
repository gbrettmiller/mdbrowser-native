//! The browsing session as a pure reducer: `step(state, event) → (state, effects)`.
//! Effects are data; `services::runner` executes them and feeds results back as events.

pub mod address;
pub mod error;
mod href;
pub mod images;
pub mod links;
pub mod location;
pub mod navigation;
pub mod network;
mod reducer;
pub mod visits;
pub mod web;

pub use reducer::step;

use std::path::PathBuf;
use std::sync::Arc;

use crate::model::Document;
use crate::theme::ReadingFont;
use error::ErrorCode;
use location::Location;
use navigation::Navigation;
use visits::Visit;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Settings {
    pub font: ReadingFont,
    pub images: images::ImageLoading,
}

/// What a load found: the source, and where it was finally found (after redirects).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    pub source: String,
    pub location: Location,
}

/// A document ready to show.
#[derive(Debug, PartialEq)]
pub struct Shown {
    pub document: Arc<Document>,
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
    LocalFromWeb,
    LocalNetworkFromWeb,
}

impl NoticeKind {
    /// Every kind, for exhaustive content tests.
    pub const ALL: [Self; 4] = [Self::OpenFailed, Self::UnsupportedLink, Self::LocalFromWeb, Self::LocalNetworkFromWeb];
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
    Loaded {
        request_id: u64,
        result: Result<Loaded, ErrorCode>,
        now: u64,
    },
    OpenerFailed,
    /// Hand the current page to the system browser (see `State::browser_offer`).
    OpenInBrowser,
    OpenHistory(String),
    ClearHistory,
    ChooseFont(ReadingFont),
    ChooseImageLoading(images::ImageLoading),
    /// An image load finished.
    ImageLoaded {
        location: Location,
        result: Result<images::Picture, images::Problem>,
    },
    /// A click on a waiting or failed image.
    LoadImage(Location),
    /// Ctrl/Cmd+Shift+I: load this page's images, whatever the setting.
    LoadPageImages,
    DismissNotice(u64),
    FragmentHandled,
    Restore {
        visits: Vec<Visit>,
        settings: Settings,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Load { request_id: u64, location: Location, origin: web::Origin },
    LoadImage { location: Location, origin: web::Origin },
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
    /// Images: their states, the current page's sources, and the cache.
    pub images: images::Images,
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
            images: images::Images::new(),
        }
    }

    /// What the address bar shows.
    pub fn address(&self) -> String {
        self.nav.current().map(|entry| entry.input.clone()).unwrap_or_default()
    }

    pub fn current_location(&self) -> Option<&Location> {
        self.nav.current()?.target.as_ref().map(|target| &target.location)
    }

    /// The address the system browser may open instead: only a web page that
    /// turned out not to be Markdown. Local files are never handed over.
    pub fn browser_offer(&self) -> Option<String> {
        let not_markdown = self.status == Status::Failed(ErrorCode::NotMarkdown);
        self.current_location().filter(|location| not_markdown && location.is_web()).map(Location::address)
    }
}

#[cfg(test)]
mod tests;
