//! Every user-facing string.

use crate::session::NoticeKind;
use crate::session::error::ErrorCode;
use crate::session::images::{ImageLoading, Note};
use crate::theme::{Appearance, ReadingFont, Spacing, TextSize};

pub fn error(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::Empty => "Type the path or web address of a Markdown file.",
        ErrorCode::NotAbsolute => "Use a full path starting with / or ~/, or a web address starting with https://.",
        ErrorCode::UnsupportedScheme => "Only files and http or https addresses can be opened.",
        ErrorCode::NotMarkdown => "That isn't a Markdown file.",
        ErrorCode::NotMarkdownWithLlms => "That isn't a Markdown file, but its site has an llms.txt.",
        ErrorCode::NotFound => "There's no file at this address.",
        ErrorCode::Unreadable => "This file couldn't be read as text.",
        ErrorCode::ServerError => "The site had a problem sending this page.",
        ErrorCode::Unreachable => "Couldn't reach this site.",
        ErrorCode::BadRedirect => "This site redirected somewhere that can't be followed safely.",
        ErrorCode::LocalAddress => "This address leads into your local network.",
    }
}

/// A failed page fetch: what went wrong, and what to try next. `None` for errors
/// about the typed address, which never reach the network.
pub fn web_error(code: ErrorCode) -> Option<(&'static str, &'static str)> {
    match code {
        ErrorCode::Empty | ErrorCode::NotAbsolute | ErrorCode::UnsupportedScheme => None,
        ErrorCode::NotMarkdown => Some(("This page isn't Markdown.", "You can open it in your web browser instead.")),
        ErrorCode::NotMarkdownWithLlms => Some((
            "This page isn't Markdown, but the site has an llms.txt.",
            "You can read that, or open this page in your web browser.",
        )),
        ErrorCode::NotFound => Some(("There's no page at this address.", "Check it for typos.")),
        ErrorCode::Unreadable => Some(("This page couldn't be read as text.", "It may be too large or not in UTF-8.")),
        ErrorCode::ServerError => Some((error(code), "Try again later.")),
        ErrorCode::Unreachable => Some((error(code), "Check your connection, then reload.")),
        ErrorCode::BadRedirect => Some((error(code), "Try opening the address in your web browser.")),
        ErrorCode::LocalAddress => {
            Some((error(code), "Links from web pages can't open it. Type the address yourself if you trust it."))
        }
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
        NoticeKind::LocalNetworkFromWeb => "Web pages can't open addresses on your local network.",
        NoticeKind::BadHome => "That isn't a path or web address of a Markdown file, so the home page is unchanged.",
    }
}

pub fn appearance(appearance: Appearance) -> &'static str {
    match appearance {
        Appearance::System => "System",
        Appearance::Light => "Light",
        Appearance::Dark => "Dark",
        Appearance::Custom => "Custom",
    }
}

pub fn spacing(spacing: Spacing) -> &'static str {
    match spacing {
        Spacing::Compact => "Compact",
        Spacing::Comfortable => "Comfortable",
        Spacing::Spacious => "Spacious",
    }
}

/// The text size as the panel shows it: whole pixels.
pub fn text_size(size: TextSize) -> String {
    size.px().to_string()
}

/// The Web images setting, one entry per mode.
pub fn image_loading(mode: ImageLoading) -> &'static str {
    match mode {
        ImageLoading::Automatic => "Automatic",
        ImageLoading::OnClick => "On click",
        ImageLoading::Off => "Off",
    }
}

/// The key that goes with Shift+I to load this page's images.
const SHORTCUT_MODIFIER: &str = if cfg!(target_os = "macos") { "Cmd" } else { "Ctrl" };

/// The line under an image that isn't shown.
pub fn image_note(note: Note) -> String {
    match note {
        Note::Loading => "Loading image…".into(),
        Note::ClickToLoad => format!("Click to load, or press {SHORTCUT_MODIFIER}+Shift+I for this page's."),
        Note::Off => format!("Web images are off. Press {SHORTCUT_MODIFIER}+Shift+I to load this page's."),
        Note::Failed => format!("Couldn't load this image. Click or press {SHORTCUT_MODIFIER}+Shift+I to try again."),
        Note::Unsupported => "This image's format isn't supported.".into(),
        Note::TooLarge => "This image is too large to show.".into(),
        Note::Refused => "This page can't show this image.".into(),
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
pub const OPEN_IN_BROWSER: &str = "Open in browser";
pub const ADDRESS_PLACEHOLDER: &str = "Path or web address of a Markdown file, e.g. ~/notes/index.md";
pub const HOME: &str = "Home";
pub const SETTINGS: &str = "Settings";
pub const FONT_LABEL: &str = "Font";
pub const IMAGES_LABEL: &str = "Web images";
pub const APPEARANCE_LABEL: &str = "Appearance";
pub const TEXT_SIZE_LABEL: &str = "Text size";
pub const SMALLER_TEXT: &str = "A−";
pub const LARGER_TEXT: &str = "A+";
pub const SPACING_LABEL: &str = "Spacing";
pub const RESET_READER_CONTROLS: &str = "Reset size and spacing";
pub const HOME_PAGE_LABEL: &str = "Home page";
pub const HOME_PAGE_PLACEHOLDER: &str = "Empty for the welcome page";
pub const LOADING: &str = "Loading…";
pub const LISTEN: &str = "Listen:";
pub const PREVIOUS: &str = "← Previous";
pub const NEXT: &str = "Next →";
pub const TRY_LLMS_TXT: &str = "Try this site's llms.txt";

/// The banner over a page whose Markdown version was found instead of `original`.
pub fn found_instead(original: &str) -> String {
    format!("This site doesn't serve Markdown at {original}. Showing the Markdown version instead.")
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    /// Every error a fetch can end in; the rest are about the typed address.
    const WEB_ERRORS: [ErrorCode; 8] = [
        ErrorCode::NotMarkdown,
        ErrorCode::NotMarkdownWithLlms,
        ErrorCode::NotFound,
        ErrorCode::Unreadable,
        ErrorCode::ServerError,
        ErrorCode::Unreachable,
        ErrorCode::BadRedirect,
        ErrorCode::LocalAddress,
    ];

    fn distinct_and_non_empty(strings: &[&str]) {
        assert!(strings.iter().all(|s| !s.trim().is_empty()), "{strings:?}");
        assert_eq!(strings.iter().collect::<HashSet<_>>().len(), strings.len(), "{strings:?}");
    }

    #[test]
    fn the_reader_controls_are_labelled() {
        distinct_and_non_empty(&Spacing::ALL.map(spacing));
        distinct_and_non_empty(&[TEXT_SIZE_LABEL, SPACING_LABEL, SMALLER_TEXT, LARGER_TEXT, RESET_READER_CONTROLS]);
        assert_eq!(text_size(TextSize::default()), "18");
    }

    #[test]
    fn the_welcome_page_is_titled_and_names_the_controls_it_describes() {
        let source = include_str!("../assets/welcome.md");
        assert_eq!(crate::model::parse(source).title(""), "Welcome");
        for label in [SETTINGS, HOME_PAGE_LABEL, HOME] {
            assert!(source.contains(label), "the welcome page should mention {label}");
        }
    }

    #[test]
    fn every_error_and_notice_has_its_own_string() {
        distinct_and_non_empty(&ErrorCode::ALL.map(error));
        distinct_and_non_empty(&ErrorCode::ALL.map(ErrorCode::key));
        distinct_and_non_empty(&NoticeKind::ALL.map(notice));
    }

    #[test]
    fn the_web_images_menu_and_image_notes() {
        let labels = ImageLoading::ALL.map(image_loading);
        assert_eq!(labels, ["Automatic", "On click", "Off"]);
        let notes = Note::ALL.map(image_note);
        distinct_and_non_empty(&notes.each_ref().map(String::as_str));
        assert_eq!(image_note(Note::Loading), "Loading image…");
        assert_eq!(image_note(Note::Unsupported), "This image's format isn't supported.");
        assert_eq!(image_note(Note::TooLarge), "This image is too large to show.");
        assert_eq!(image_note(Note::Refused), "This page can't show this image.");
        let modifier = if cfg!(target_os = "macos") { "Cmd" } else { "Ctrl" };
        // Each clickable note names the keyboard route too.
        assert_eq!(
            image_note(Note::ClickToLoad),
            format!("Click to load, or press {modifier}+Shift+I for this page's.")
        );
        assert_eq!(
            image_note(Note::Failed),
            format!("Couldn't load this image. Click or press {modifier}+Shift+I to try again.")
        );
        assert_eq!(image_note(Note::Off), format!("Web images are off. Press {modifier}+Shift+I to load this page's."));
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
    fn the_fallback_banner_names_the_original_address() {
        let banner = found_instead("https://x.org/post");
        assert!(banner.contains("https://x.org/post"));
        distinct_and_non_empty(&[&banner, TRY_LLMS_TXT, OPEN_IN_BROWSER]);
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
