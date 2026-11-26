//! Why an address or load failed.

/// `key()` is a stable identifier; `content` holds the words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Empty,
    NotAbsolute,
    UnsupportedScheme,
    NotMarkdown,
    /// Not Markdown, and the site has an `llms.txt` to offer instead.
    NotMarkdownWithLlms,
    /// A file, not a page: audio, video, an archive, a PDF and the like.
    IsFile,
    NotFound,
    Unreadable,
    ServerError,
    Unreachable,
    BadRedirect,
    LocalAddress,
}

impl ErrorCode {
    /// Every code, for exhaustive content tests; `key` fails to compile until a new code is handled.
    pub const ALL: [Self; 12] = [
        Self::Empty,
        Self::NotAbsolute,
        Self::UnsupportedScheme,
        Self::NotMarkdown,
        Self::NotMarkdownWithLlms,
        Self::IsFile,
        Self::NotFound,
        Self::Unreadable,
        Self::ServerError,
        Self::Unreachable,
        Self::BadRedirect,
        Self::LocalAddress,
    ];

    /// Whether the address was fetched and wasn't Markdown: a page, with or without an
    /// `llms.txt` to offer, or a file.
    pub const fn is_not_markdown(self) -> bool {
        matches!(self, Self::NotMarkdown | Self::NotMarkdownWithLlms | Self::IsFile)
    }

    pub const fn key(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::NotAbsolute => "notAbsolute",
            Self::UnsupportedScheme => "unsupportedScheme",
            Self::NotMarkdown => "notMarkdown",
            Self::NotMarkdownWithLlms => "notMarkdownWithLlms",
            Self::IsFile => "isFile",
            Self::NotFound => "notFound",
            Self::Unreadable => "unreadable",
            Self::ServerError => "serverError",
            Self::Unreachable => "unreachable",
            Self::BadRedirect => "badRedirect",
            Self::LocalAddress => "localAddress",
        }
    }
}
