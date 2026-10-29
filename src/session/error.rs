//! Why an address or load failed.

/// `key()` is a stable identifier; `content` holds the words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Empty,
    NotAbsolute,
    UnsupportedScheme,
    NotMarkdown,
    NotFound,
    Unreadable,
    ServerError,
    Unreachable,
    BadRedirect,
}

impl ErrorCode {
    /// Every code, for exhaustive content tests; `key` fails to compile until a new code is handled.
    pub const ALL: [Self; 9] = [
        Self::Empty,
        Self::NotAbsolute,
        Self::UnsupportedScheme,
        Self::NotMarkdown,
        Self::NotFound,
        Self::Unreadable,
        Self::ServerError,
        Self::Unreachable,
        Self::BadRedirect,
    ];

    pub const fn key(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::NotAbsolute => "notAbsolute",
            Self::UnsupportedScheme => "unsupportedScheme",
            Self::NotMarkdown => "notMarkdown",
            Self::NotFound => "notFound",
            Self::Unreadable => "unreadable",
            Self::ServerError => "serverError",
            Self::Unreachable => "unreachable",
            Self::BadRedirect => "badRedirect",
        }
    }
}
