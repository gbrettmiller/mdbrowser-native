//! Every user-facing string.

use crate::session::NoticeKind;
use crate::session::location::ErrorCode;

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
