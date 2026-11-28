//! Why an address or load failed.

/// `key()` is a stable identifier; `content` holds the words. HTTP failures carry
/// the status the site answered with, so the reader can be told it.
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
    /// 401 or 403: the page is private or needs a sign-in, which this app never sends.
    NeedsSignIn(u16),
    /// Any other 4xx but 404, 410 and 429.
    Refused(u16),
    /// 429 or 503: worth trying again later.
    Busy(u16),
    /// Any other status that isn't a page, a redirect or a 4xx.
    ServerError(u16),
    Unreachable,
    TimedOut,
    /// The name didn't resolve.
    NoSuchSite,
    /// The site's TLS certificate wasn't valid for it.
    Certificate,
    BadRedirect,
    LocalAddress,
}

impl ErrorCode {
    /// Every code, for exhaustive content tests; `key` fails to compile until a new code is handled.
    pub const ALL: [Self; 18] = [
        Self::Empty,
        Self::NotAbsolute,
        Self::UnsupportedScheme,
        Self::NotMarkdown,
        Self::NotMarkdownWithLlms,
        Self::IsFile,
        Self::NotFound,
        Self::Unreadable,
        Self::NeedsSignIn(403),
        Self::Refused(400),
        Self::Busy(503),
        Self::ServerError(500),
        Self::Unreachable,
        Self::TimedOut,
        Self::NoSuchSite,
        Self::Certificate,
        Self::BadRedirect,
        Self::LocalAddress,
    ];

    /// Whether the system browser may do better: the address wasn't Markdown (a page,
    /// with or without an `llms.txt` to offer, or a file), or it needs a sign-in.
    pub const fn offers_browser(self) -> bool {
        matches!(self, Self::NotMarkdown | Self::NotMarkdownWithLlms | Self::IsFile | Self::NeedsSignIn(_))
    }

    /// The HTTP status the site answered with, for the codes that have one.
    pub const fn status(self) -> Option<u16> {
        match self {
            Self::NeedsSignIn(status) | Self::Refused(status) | Self::Busy(status) | Self::ServerError(status) => {
                Some(status)
            }
            _ => None,
        }
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
            Self::NeedsSignIn(_) => "needsSignIn",
            Self::Refused(_) => "refused",
            Self::Busy(_) => "busy",
            Self::ServerError(_) => "serverError",
            Self::Unreachable => "unreachable",
            Self::TimedOut => "timedOut",
            Self::NoSuchSite => "noSuchSite",
            Self::Certificate => "certificate",
            Self::BadRedirect => "badRedirect",
            Self::LocalAddress => "localAddress",
        }
    }
}
