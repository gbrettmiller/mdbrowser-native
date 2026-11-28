//! Every user-facing string.

use crate::session::NoticeKind;
use crate::session::error::ErrorCode;
use crate::session::images::{ImageLoading, Note};
use crate::theme::icons::Icon;
use crate::theme::{Appearance, ReadingFont, Spacing, TextSize, Which};

pub fn error(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::Empty => "Type the path or web address of a Markdown file.",
        ErrorCode::NotAbsolute => "Use a full path starting with / or ~/, or a web address starting with https://.",
        ErrorCode::UnsupportedScheme => "Only files and http or https addresses can be opened.",
        ErrorCode::NotMarkdown => "That isn't a Markdown file.",
        ErrorCode::NotMarkdownWithLlms => "That isn't a Markdown file, but its site has an llms.txt.",
        ErrorCode::IsFile => "That's a file to download, not a page.",
        ErrorCode::NotFound => "There's no file at this address.",
        ErrorCode::Unreadable => "This file couldn't be read as text.",
        ErrorCode::NeedsSignIn(_) => "This page is private or needs you to sign in.",
        ErrorCode::Refused(_) => "The site wouldn't send this page.",
        ErrorCode::Busy(_) => "The site is too busy to send this page.",
        ErrorCode::ServerError(_) => "The site had a problem sending this page.",
        ErrorCode::Unreachable => "Couldn't reach this site.",
        ErrorCode::TimedOut => "The site took too long to answer.",
        ErrorCode::NoSuchSite => "There's no site by that name.",
        ErrorCode::Certificate => "This site's security certificate isn't valid.",
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
        ErrorCode::IsFile => Some(("This is a file, not a page.", "You can download it with your web browser.")),
        ErrorCode::NotFound => Some(("There's no page at this address.", "Check it for typos.")),
        ErrorCode::Unreadable => Some(("This page couldn't be read as text.", "It may be too large or not in UTF-8.")),
        ErrorCode::NeedsSignIn(_) => {
            Some((error(code), "mdBrowser doesn't sign in to sites. You can open it in your web browser."))
        }
        ErrorCode::Refused(_) => Some((error(code), "Check the address for typos.")),
        ErrorCode::Busy(_) => Some((error(code), "Try again in a while.")),
        ErrorCode::ServerError(_) => Some((error(code), "Try again later.")),
        ErrorCode::Unreachable => Some((error(code), "Check your connection, then reload.")),
        ErrorCode::TimedOut => Some((error(code), "Check your connection, or reload to try again.")),
        ErrorCode::NoSuchSite => Some((error(code), "Check the address for typos.")),
        ErrorCode::Certificate => Some((error(code), "The connection may not be private, so the page wasn't loaded.")),
        ErrorCode::BadRedirect => Some((error(code), "Try opening the address in your web browser.")),
        ErrorCode::LocalAddress => {
            Some((error(code), "Links from web pages can't open it. Type the address yourself if you trust it."))
        }
    }
}

/// "The site answered 403 Forbidden.", for a failure that came with an HTTP status.
/// A status without a common name is given as a number.
pub fn status_line(code: ErrorCode) -> Option<String> {
    let status = code.status()?;
    Some(match reason(status) {
        Some(reason) => format!("The site answered {status} {reason}."),
        None => format!("The site answered {status}."),
    })
}

/// The standard name of a status the reader may meet (RFC 9110 and RFC 6585).
/// HTTP/2 sends no reason phrase, so these are ours, not the server's.
const fn reason(status: u16) -> Option<&'static str> {
    Some(match status {
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Content Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        421 => "Misdirected Request",
        422 => "Unprocessable Content",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        _ => return None,
    })
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
        NoticeKind::BadColour => "That isn't a colour. Use a hex value such as #222 or #fdf6e3.",
        NoticeKind::UnreadableText => {
            "That text colour is too close to the page colour to read comfortably, so the colours are unchanged."
        }
        NoticeKind::UnreadableLink => {
            "That link colour is too close to the page colour to read comfortably, so the colours are unchanged."
        }
    }
}

pub fn appearance(appearance: Appearance) -> &'static str {
    match appearance {
        Appearance::System => "System",
        Appearance::Light => "Light",
        Appearance::Dark => "Dark",
        Appearance::Sepia => "Sepia",
        Appearance::PaperGreen => "Paper green",
        Appearance::Rose => "Rose",
        Appearance::Night => "Night",
        Appearance::SolarizedDark => "Solarized dark",
        Appearance::Nord => "Nord",
        Appearance::HighContrast => "High contrast",
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

/// The label of one of the reader's three colours.
pub fn colour_name(which: Which) -> &'static str {
    match which {
        Which::Page => "Page",
        Which::Text => "Text",
        Which::Link => "Link",
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

pub const APP_NAME: &str = "mdBrowser";
pub const HISTORY: &str = "History";
pub const CLEAR_HISTORY: &str = "Clear History";
pub const HISTORY_EMPTY: &str = "Nothing here yet. Documents you open are listed here.";
pub const BOOKMARKS: &str = "Bookmarks";
pub const BOOKMARKS_EMPTY: &str = "No bookmarks yet. The star in the toolbar keeps the page you're reading.";

/// An icon button's name, with its shortcut if it has one: its tooltip, and what the status
/// line shows while it has the keyboard (ADR-016).
pub fn icon_name(icon: Icon, mac: bool) -> String {
    let (primary, alt) = if mac { ("Cmd", "Option") } else { ("Ctrl", "Alt") };
    match icon {
        Icon::Back => format!("Back ({alt}+←)"),
        Icon::Forward => format!("Forward ({alt}+→)"),
        Icon::Reload => format!("Reload ({primary}+R)"),
        Icon::Source => source_tooltip(false, mac),
        Icon::Page => source_tooltip(true, mac),
        Icon::Home => format!("Home page ({alt}+Home)"),
        Icon::Bookmark => format!("Bookmark this page ({primary}+D)"),
        Icon::Bookmarked => format!("Remove this bookmark ({primary}+D)"),
        Icon::Bookmarks => format!("Bookmarks ({primary}+B)"),
        // Cmd+H hides the app on macOS, so there History has no shortcut.
        Icon::History if mac => "History".into(),
        Icon::History => format!("History ({primary}+H)"),
        Icon::Settings => format!("Settings ({primary}+,)"),
        Icon::MoveUp => "Move up".into(),
        Icon::MoveDown => "Move down".into(),
        Icon::Remove => "Remove bookmark".into(),
    }
}
pub const DISMISS: &str = "Dismiss";
pub const OPEN_IN_BROWSER: &str = "Open in browser";
pub const ADDRESS_PLACEHOLDER: &str = "Path or web address of a Markdown file, e.g. ~/notes/index.md";
/// The source button's tooltip: what pressing it shows, and the chord.
pub fn source_tooltip(in_source: bool, mac: bool) -> String {
    let what = if in_source { "View page" } else { "View Markdown source" };
    format!("{what} ({}+U)", if mac { "Cmd" } else { "Ctrl" })
}

/// The Settings panel's button that opens the About page, and its tooltip, which names both chords.
pub const ABOUT: &str = "About mdBrowser";
pub fn about_tooltip(mac: bool) -> String {
    format!("Version, licence and help (F1 or {}+?)", if mac { "Cmd" } else { "Ctrl" })
}

/// The line above the source of a page.
pub fn source_of(title: &str) -> String {
    format!("Source of {title}")
}
pub const SETTINGS: &str = "Settings";
pub const FONT_LABEL: &str = "Font";
pub const IMAGES_LABEL: &str = "Web images";
pub const APPEARANCE_LABEL: &str = "Appearance";
pub const TEXT_SIZE_LABEL: &str = "Text size";
pub const SMALLER_TEXT: &str = "A−";
pub const LARGER_TEXT: &str = "A+";
pub const COLOURS_LABEL: &str = "Custom colours";
/// What a colour field accepts, as its placeholder says.
/// Under the colour fields: how a typed colour is applied.
pub const COLOURS_HINT: &str = "Press Enter or Tab to apply a colour.";
pub const COLOUR_PLACEHOLDER: &str = "#rgb or #rrggbb";
pub const SPACING_LABEL: &str = "Spacing";
pub const RESET_READER_CONTROLS: &str = "Reset size and spacing";
pub const HOME_PAGE_LABEL: &str = "Home page";
pub const HOME_PAGE_PLACEHOLDER: &str = "Empty for the welcome page";
pub const LOADING: &str = "Loading…";
pub const LISTEN: &str = "Listen:";
pub const PREVIOUS: &str = "← Previous";
pub const NEXT: &str = "Next →";
pub const TRY_LLMS_TXT: &str = "Try this site's llms.txt";

pub const LEAVE_AND_OPEN: &str = "Open";
pub const STAY_HERE: &str = "Stay here";

/// The question over a page when a link would leave mdBrowser for the system.
pub fn leaving(address: &str) -> String {
    format!("This link leaves mdBrowser and opens in your system's own app: {address}")
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    /// Every error a fetch can end in; the rest are about the typed address.
    const WEB_ERRORS: [ErrorCode; 15] = [
        ErrorCode::NotMarkdown,
        ErrorCode::NotMarkdownWithLlms,
        ErrorCode::IsFile,
        ErrorCode::NotFound,
        ErrorCode::Unreadable,
        ErrorCode::NeedsSignIn(403),
        ErrorCode::Refused(400),
        ErrorCode::Busy(503),
        ErrorCode::ServerError(500),
        ErrorCode::Unreachable,
        ErrorCode::TimedOut,
        ErrorCode::NoSuchSite,
        ErrorCode::Certificate,
        ErrorCode::BadRedirect,
        ErrorCode::LocalAddress,
    ];

    fn distinct_and_non_empty(strings: &[&str]) {
        assert!(strings.iter().all(|s| !s.trim().is_empty()), "{strings:?}");
        assert_eq!(strings.iter().collect::<HashSet<_>>().len(), strings.len(), "{strings:?}");
    }

    #[test]
    fn every_icon_is_named_with_its_shortcut_on_each_platform() {
        for mac in [false, true] {
            let names = Icon::ALL.map(|icon| icon_name(icon, mac));
            distinct_and_non_empty(&names.each_ref().map(String::as_str));
            let (wrong_primary, wrong_alt) = if mac { ("Ctrl", "Alt") } else { ("Cmd", "Option") };
            assert!(names.iter().all(|name| !name.contains(wrong_primary) && !name.contains(wrong_alt)), "{names:?}");
        }
        assert_eq!(icon_name(Icon::Back, false), "Back (Alt+←)");
        assert_eq!(icon_name(Icon::Back, true), "Back (Option+←)");
        assert_eq!(icon_name(Icon::Bookmark, true), "Bookmark this page (Cmd+D)");
        assert_eq!(icon_name(Icon::History, false), "History (Ctrl+H)");
        assert_eq!(icon_name(Icon::History, true), "History", "Cmd+H hides the app on macOS");
        assert_eq!(icon_name(Icon::Source, false), source_tooltip(false, false));
        assert_eq!(icon_name(Icon::Page, true), source_tooltip(true, true));
    }

    #[test]
    fn the_reader_controls_are_labelled() {
        distinct_and_non_empty(&Spacing::ALL.map(spacing));
        distinct_and_non_empty(&Which::ALL.map(colour_name));
        assert!(COLOURS_HINT.contains("Enter") && COLOURS_HINT.contains("Tab"));
        assert_eq!(source_tooltip(false, false), "View Markdown source (Ctrl+U)");
        assert_eq!(source_tooltip(false, true), "View Markdown source (Cmd+U)");
        assert_eq!(source_tooltip(true, false), "View page (Ctrl+U)", "in the source view it goes back");
        assert_eq!(source_tooltip(true, true), "View page (Cmd+U)");
        assert_eq!(ABOUT, "About mdBrowser");
        assert_eq!(about_tooltip(false), "Version, licence and help (F1 or Ctrl+?)");
        assert_eq!(about_tooltip(true), "Version, licence and help (F1 or Cmd+?)");
        assert!(COLOUR_PLACEHOLDER.contains("#rgb") && COLOUR_PLACEHOLDER.contains("#rrggbb"));
        distinct_and_non_empty(&[TEXT_SIZE_LABEL, SPACING_LABEL, SMALLER_TEXT, LARGER_TEXT, RESET_READER_CONTROLS]);
        assert_eq!(text_size(TextSize::default()), "18");
    }

    #[test]
    fn the_packaged_identity_matches_the_apps_name_and_icons() {
        let cargo = include_str!("../Cargo.toml");
        assert!(cargo.contains(&format!("product-name = \"{APP_NAME}\"")), "Cargo.toml's product-name");
        assert!(cargo.contains("identifier = \"com.gbrettmiller.mdbrowser\""));
        for icon in ["icon.ico", "icon.icns", "icon-512.png", "icon-256.png", "icon-128.png", "icon-32.png"] {
            assert!(cargo.contains(&format!("assets/icon/{icon}")), "{icon}");
            assert!(std::path::Path::new("assets/icon").join(icon).is_file(), "{icon} exists");
        }
    }

    #[test]
    fn the_welcome_page_is_titled_and_names_the_controls_it_describes() {
        let source = include_str!("../assets/welcome.md");
        assert_eq!(crate::model::parse(source).title(""), "Welcome");
        for label in [SETTINGS, HOME_PAGE_LABEL, "the gear", "the house"] {
            assert!(source.contains(label), "the welcome page should mention {label}");
        }
        assert!(source.contains("[About mdBrowser](about:mdbrowser)"), "the welcome page links to About");
    }

    #[test]
    fn the_welcome_page_leads_with_the_web_of_markdown_and_gives_local_files_one_line() {
        let source = include_str!("../assets/welcome.md");
        let intro = source.split("\n## ").next().unwrap();
        assert!(intro.contains("web of Markdown"), "the first thing it says is what this is");
        assert!(intro.contains("web address"));
        assert!(!intro.contains("command line"), "local files are not the lead");
        let starts = source.find("## Somewhere to start").expect("a starting section");
        let local = source.find("path").expect("a line for local files");
        assert!(local > starts, "local files come after the live sites");
        assert_eq!(source.matches("path").count(), 1, "one short line, not a section");
    }

    #[test]
    fn every_error_and_notice_has_its_own_string() {
        distinct_and_non_empty(&ErrorCode::ALL.map(error));
        distinct_and_non_empty(&ErrorCode::ALL.map(ErrorCode::key));
        distinct_and_non_empty(&NoticeKind::ALL.map(notice));
    }

    #[test]
    fn the_colour_notices_say_what_to_type_and_what_failed() {
        assert!(notice(NoticeKind::BadColour).contains("#222") && notice(NoticeKind::BadColour).contains("hex"));
        assert!(notice(NoticeKind::UnreadableText).contains("text colour is too close to the page colour"));
        assert!(notice(NoticeKind::UnreadableLink).contains("link colour is too close to the page colour"));
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
    fn an_http_error_names_its_status() {
        assert_eq!(status_line(ErrorCode::NeedsSignIn(403)).as_deref(), Some("The site answered 403 Forbidden."));
        assert_eq!(status_line(ErrorCode::NeedsSignIn(401)).as_deref(), Some("The site answered 401 Unauthorized."));
        assert_eq!(status_line(ErrorCode::Busy(429)).as_deref(), Some("The site answered 429 Too Many Requests."));
        assert_eq!(status_line(ErrorCode::Busy(503)).as_deref(), Some("The site answered 503 Service Unavailable."));
        assert_eq!(
            status_line(ErrorCode::ServerError(500)).as_deref(),
            Some("The site answered 500 Internal Server Error.")
        );
        assert_eq!(status_line(ErrorCode::Refused(418)).as_deref(), Some("The site answered 418."));
        for code in [ErrorCode::NotFound, ErrorCode::Unreachable, ErrorCode::Certificate, ErrorCode::NotMarkdown] {
            assert_eq!(status_line(code), None, "{code:?}");
        }
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
