//! Every user-facing string.

use crate::session::NoticeKind;
use crate::session::error::ErrorCode;
use crate::theme::ReadingFont;

pub fn error(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::Empty => "Type the path or web address of a Markdown file.",
        ErrorCode::NotAbsolute => "Use a full path starting with / or ~/, or a web address starting with https://.",
        ErrorCode::UnsupportedScheme => "Only files and http or https addresses can be opened.",
        ErrorCode::NotMarkdown => "That isn't a Markdown file.",
        ErrorCode::NotFound => "There's no file at this address.",
        ErrorCode::Unreadable => "This file couldn't be read as text.",
        ErrorCode::ServerError => "The site had a problem sending this page.",
        ErrorCode::Unreachable => "Couldn't reach this site.",
        ErrorCode::BadRedirect => "This site redirected somewhere that can't be followed safely.",
    }
}

/// A failed page fetch: what went wrong, and what to try next. `None` for errors
/// about the typed address, which never reach the network.
pub fn web_error(code: ErrorCode) -> Option<(&'static str, &'static str)> {
    match code {
        ErrorCode::Empty | ErrorCode::NotAbsolute | ErrorCode::UnsupportedScheme => None,
        ErrorCode::NotMarkdown => Some(("This page isn't Markdown.", "Open it in your web browser instead.")),
        ErrorCode::NotFound => Some(("There's no page at this address.", "Check it for typos.")),
        ErrorCode::Unreadable => Some(("This page couldn't be read as text.", "It may be too large or not in UTF-8.")),
        ErrorCode::ServerError => Some((error(code), "Try again later.")),
        ErrorCode::Unreachable => Some((error(code), "Check your connection, then reload.")),
        ErrorCode::BadRedirect => Some((error(code), "Try opening the address in your web browser.")),
    }
}

/// What went wrong, plus what to try next for a page on the web.
pub fn failure(code: ErrorCode, web: bool) -> (&'static str, Option<&'static str>) {
    match web_error(code).filter(|_| web) {
        Some((message, hint)) => (message, Some(hint)),
        None => (error(code), None),
    }
}

/// "Loading example.com…" for a page, plain "Loading…" for a file.
pub fn loading(host: Option<&str>) -> String {
    host.map_or_else(|| LOADING.to_string(), |host| format!("Loading {host}…"))
}

pub fn notice(kind: NoticeKind) -> &'static str {
    match kind {
        NoticeKind::OpenFailed => "Couldn't open this link.",
        NoticeKind::UnsupportedLink => "Can't open this kind of link yet",
        NoticeKind::LocalFromWeb => "Web pages can't open files on your computer.",
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
pub const ADDRESS_PLACEHOLDER: &str = "Path or web address of a Markdown file, e.g. ~/notes/index.md";
pub const WELCOME: &str = "Type the path or web address of a Markdown file above, or pass one on the command line.";
pub const LOADING: &str = "Loading…";

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    /// Every error a fetch can end in; the rest are about the typed address.
    const WEB_ERRORS: [ErrorCode; 6] = [
        ErrorCode::NotMarkdown,
        ErrorCode::NotFound,
        ErrorCode::Unreadable,
        ErrorCode::ServerError,
        ErrorCode::Unreachable,
        ErrorCode::BadRedirect,
    ];

    fn distinct_and_non_empty(strings: &[&str]) {
        assert!(strings.iter().all(|s| !s.trim().is_empty()), "{strings:?}");
        assert_eq!(strings.iter().collect::<HashSet<_>>().len(), strings.len(), "{strings:?}");
    }

    #[test]
    fn every_error_and_notice_has_its_own_string() {
        distinct_and_non_empty(&ErrorCode::ALL.map(error));
        distinct_and_non_empty(&ErrorCode::ALL.map(ErrorCode::key));
        distinct_and_non_empty(&NoticeKind::ALL.map(notice));
    }

    #[test]
    fn every_web_error_says_what_went_wrong_and_what_to_try() {
        for code in WEB_ERRORS {
            let (message, hint) = web_error(code).unwrap_or_else(|| panic!("{code:?} has no web string"));
            distinct_and_non_empty(&[message, hint]);
        }
        let messages: Vec<_> = WEB_ERRORS.iter().filter_map(|code| web_error(*code)).map(|(m, _)| m).collect();
        distinct_and_non_empty(&messages);
    }

    #[test]
    fn loading_names_the_site_when_there_is_one() {
        assert_eq!(loading(Some("example.com")), "Loading example.com…");
        assert_eq!(loading(None), LOADING);
    }

    #[test]
    fn failures_on_the_web_carry_a_hint_and_local_ones_do_not() {
        for code in WEB_ERRORS {
            let (message, hint) = failure(code, true);
            assert!(!message.is_empty() && hint.is_some_and(|hint| !hint.is_empty()), "{code:?}");
            assert_eq!(failure(code, false), (error(code), None), "{code:?}");
        }
        assert_eq!(failure(ErrorCode::NotFound, true).0, "There's no page at this address.");
        assert_eq!(failure(ErrorCode::NotAbsolute, true), (error(ErrorCode::NotAbsolute), None));
    }

    #[test]
    fn address_errors_have_no_web_string() {
        for code in ErrorCode::ALL.into_iter().filter(|code| !WEB_ERRORS.contains(code)) {
            assert_eq!(web_error(code), None, "{code:?}");
        }
    }
}
