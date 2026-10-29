//! Every user-facing string.

use crate::session::NoticeKind;
use crate::session::location::ErrorCode;
use crate::theme::ReadingFont;

pub fn error(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::Empty => "Type the path of a Markdown file.",
        ErrorCode::NotAbsolute => "Use a full path, starting with / or ~/.",
        ErrorCode::UnsupportedScheme => "Only local files can be opened for now.",
        ErrorCode::NotMarkdown => "That isn't a Markdown file.",
        ErrorCode::NotFound => "There's no file at this address.",
        ErrorCode::Unreadable => "This file couldn't be read as text.",
    }
}

pub fn notice(kind: NoticeKind) -> &'static str {
    match kind {
        NoticeKind::OpenFailed => "Couldn't open this link.",
        NoticeKind::UnsupportedLink => "Can't open this kind of link yet",
    }
}

pub fn font_name(font: ReadingFont) -> &'static str {
    match font {
        ReadingFont::Literata => "Literata",
        ReadingFont::SourceSerif => "Source Serif 4",
        ReadingFont::Quattro => "iA Writer Quattro",
        ReadingFont::Inter => "Inter",
    }
}

pub const APP_NAME: &str = "mdbrowser";
pub const BACK: &str = "←";
pub const FORWARD: &str = "→";
pub const RELOAD: &str = "Reload";
pub const HISTORY: &str = "History";
pub const CLEAR_HISTORY: &str = "Clear History";
pub const HISTORY_EMPTY: &str = "Nothing here yet. Documents you open are listed here.";
pub const DISMISS: &str = "Dismiss";
pub const ADDRESS_PLACEHOLDER: &str = "Path to a Markdown file, e.g. ~/notes/index.md";
pub const WELCOME: &str = "Type the path of a Markdown file above, or pass one on the command line.";
pub const LOADING: &str = "Loading…";
