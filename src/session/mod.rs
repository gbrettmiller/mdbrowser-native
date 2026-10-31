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
use crate::theme::{Appearance, Colours, Palette, ReadingFont, Spacing, TextSize, Which};
use error::ErrorCode;
use location::Location;
use navigation::Navigation;
use visits::Visit;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Settings {
    pub font: ReadingFont,
    pub images: images::ImageLoading,
    pub appearance: Appearance,
    pub text_size: TextSize,
    pub spacing: Spacing,
    /// The reader's own colours, once they have chosen Custom or typed one. Kept when
    /// another appearance is chosen.
    pub custom: Option<Colours>,
    /// A canonical address; empty means the built-in welcome page.
    pub home: String,
}

impl Settings {
    /// The reader's colours, or the light palette's before they have any.
    pub fn colours(&self) -> Colours {
        self.custom.unwrap_or_default()
    }
}

/// How the text size moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextStep {
    Larger,
    Smaller,
    Reset,
}

/// What a load found: the source, and where it was finally found (after redirects).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loaded {
    pub source: String,
    pub location: Location,
    /// The address asked for, when this Markdown was found in its place (ADR-005).
    pub fallback_from: Option<String>,
}

/// A document ready to show.
#[derive(Debug, PartialEq)]
pub struct Shown {
    pub document: Arc<Document>,
    pub title: String,
    /// The address the reader asked for, when this Markdown was found in its place.
    pub fallback_from: Option<String>,
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
    /// The Home page setting wasn't an address we can open.
    BadHome,
    /// A custom colour wasn't a hex value.
    BadColour,
    /// A custom set whose text can't be read on its page.
    UnreadableText,
    /// A custom set whose links can't be read on its page.
    UnreadableLink,
}

impl NoticeKind {
    /// Every kind, for exhaustive content tests.
    pub const ALL: [Self; 8] = [
        Self::OpenFailed,
        Self::UnsupportedLink,
        Self::LocalFromWeb,
        Self::LocalNetworkFromWeb,
        Self::BadHome,
        Self::BadColour,
        Self::UnreadableText,
        Self::UnreadableLink,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Notice {
    pub id: u64,
    pub kind: NoticeKind,
}

/// A post's neighbour in its blog, named by its `urlPrev` and `urlNext` front matter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Neighbour {
    Previous,
    Next,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Enter(String),
    Follow(String),
    /// Go to the shown post's previous or next one; nothing happens if it names none.
    Neighbour(Neighbour),
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
    /// The reader agreed to the link out of the app that `State::leaving` holds.
    ConfirmLeaving,
    /// The reader chose to stay instead.
    StayHere,
    /// Open the site's `llms.txt` offered on the error (see `State::llms_offer`), as the
    /// failed page's origin would.
    OpenLlmsTxt,
    OpenHistory(String),
    ClearHistory,
    ChooseFont(ReadingFont),
    ChooseAppearance(Appearance),
    /// One of the reader's own colours, as typed: `#rgb` or `#rrggbb`.
    ChooseCustomColour(Which, String),
    /// What the operating system reports for light or dark, if anything.
    SystemPalette(Option<Palette>),
    /// Move the text size one step, or back to the default.
    StepTextSize(TextStep),
    ChooseSpacing(Spacing),
    /// Put text size and spacing back to the defaults, leaving everything else.
    ResetReaderControls,
    /// Set the home page from typed input; empty clears it.
    ChooseHome(String),
    /// Open the home page, or the welcome page when none is set.
    Home,
    /// The app started, with the address given on the command line if any. That
    /// address wins; otherwise the home page opens.
    Launch(Option<String>),
    ChooseImageLoading(images::ImageLoading),
    /// An image load finished.
    ImageLoaded {
        location: Location,
        result: Result<images::Picture, images::Problem>,
    },
    /// A click on the note of a waiting or failed image, by its source on the page.
    LoadImage(String),
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
    /// A link out of the app (system browser or mail) waiting for the reader to agree.
    pub leaving: Option<String>,
    pub settings: Settings,
    /// What the operating system reports for light or dark, if anything.
    pub system_palette: Option<Palette>,
    /// Images: their states, the current page's sources, and the cache.
    pub images: images::Images,
}

impl State {
    /// The palette in use: the reader's choice, or the OS's for System.
    pub fn palette(&self) -> Palette {
        self.settings.appearance.palette(self.system_palette, self.settings.colours())
    }

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
            leaving: None,
            settings: Settings::default(),
            system_palette: None,
            images: images::Images::new(),
        }
    }

    /// What the address bar shows.
    pub fn address(&self) -> String {
        self.nav.current().map(|entry| entry.input.clone()).unwrap_or_default()
    }

    /// The link, as written, to the shown post's previous or next post.
    pub fn neighbour(&self, which: Neighbour) -> Option<String> {
        let Status::Ready(shown) = &self.status else { return None };
        let front = &shown.document.front;
        match which {
            Neighbour::Previous => front.url_prev.clone(),
            Neighbour::Next => front.url_next.clone(),
        }
    }

    pub fn current_location(&self) -> Option<&Location> {
        self.nav.current()?.target.as_ref().map(|target| &target.location)
    }

    /// The address the system browser may open instead: a web page that turned
    /// out not to be Markdown, or the original address of one whose Markdown
    /// version is shown. Local files are never handed over.
    pub fn browser_offer(&self) -> Option<String> {
        if let Status::Ready(shown) = &self.status {
            return shown.fallback_from.clone();
        }
        let not_markdown = matches!(self.status, Status::Failed(code) if code.is_not_markdown());
        self.current_location().filter(|location| not_markdown && location.is_web()).map(Location::address)
    }

    /// The site's `llms.txt`, offered (never substituted) when a web page has no
    /// Markdown version and the fetch found an `llms.txt`. Opening it is an ordinary navigation.
    pub fn llms_offer(&self) -> Option<String> {
        if self.status != Status::Failed(ErrorCode::NotMarkdownWithLlms) {
            return None;
        }
        match self.current_location()? {
            Location::Web(url) => url.join(web::LLMS_TXT).ok().map(String::from),
            Location::File(_) | Location::Welcome => None,
        }
    }
}

#[cfg(test)]
mod tests;
