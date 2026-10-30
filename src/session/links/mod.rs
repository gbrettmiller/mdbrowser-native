//! What following a link inside a document should do. A link is read against
//! the kind of document it is in: a file may link to files and the web; a web
//! page may link only to the web, never to the reader's disk.

use std::path::Path;

use url::Url;

mod page;

use super::address::{decode, normalise, scheme_of, web_target};
use super::href::{as_browsers_read, local_target};
use super::location::{Location, Target, WEB_SCHEMES, is_markdown, is_markdown_name, without_credentials};

const MAIL_SCHEME: &str = "mailto";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    /// Another Markdown document: browse to it.
    Navigate(Target),
    /// An anchor in the current document.
    Scroll(String),
    /// A web page that isn't Markdown, or mail: hand to the system.
    External(String),
    /// A web page pointing at the reader's disk or network shares: refused.
    LocalFromWeb,
    /// A public page pointing into the reader's own machine or network: refused
    /// in the app, since a disguised address (`//2130706433/`) looks harmless.
    LocalNetworkFromWeb,
    /// Everything else: other schemes, non-Markdown files, malformed refs.
    Unsupported,
}

pub fn classify(href: &str, current: &Location) -> Link {
    let reference = &as_browsers_read(href);
    if let Some(anchor) = reference.strip_prefix('#') {
        let fragment = decode(anchor);
        return if fragment.is_empty() { Link::Unsupported } else { Link::Scroll(fragment) };
    }
    match current {
        Location::File(_) => from_file(reference, current),
        Location::Web(page) => page::classify(reference, page),
        Location::Welcome => from_welcome(reference),
    }
}

/// For the system browser: the href as written, unless it carries credentials.
fn external(reference: &str, url: Url) -> Link {
    let spoofable = !url.username().is_empty() || url.password().is_some();
    Link::External(if spoofable { without_credentials(url).to_string() } else { reference.to_string() })
}

/// The built-in page links to the web and mail by absolute address, and nowhere else.
fn from_welcome(reference: &str) -> Link {
    match scheme_of(reference).as_deref() {
        Some(MAIL_SCHEME) => Link::External(reference.to_string()),
        Some(scheme) if WEB_SCHEMES.contains(&scheme) => match Url::parse(reference) {
            Ok(url) if is_markdown_name(url.path()) => Link::Navigate(web_target(url)),
            Ok(url) => external(reference, url),
            Err(_) => Link::Unsupported,
        },
        _ => Link::Unsupported,
    }
}

fn from_file(reference: &str, current: &Location) -> Link {
    match scheme_of(reference).as_deref() {
        Some(MAIL_SCHEME) => Link::External(reference.to_string()),
        Some(scheme) if WEB_SCHEMES.contains(&scheme) => match Url::parse(reference) {
            Ok(url) if is_markdown_name(url.path()) => Link::Navigate(web_target(url)),
            Ok(url) => external(reference, url),
            Err(_) => Link::Unsupported,
        },
        _ => match local_target(reference, current) {
            Some((path, fragment)) if is_markdown(Path::new(&path)) => {
                Link::Navigate(Target { location: Location::file(normalise(Path::new(&path))), fragment })
            }
            _ => Link::Unsupported,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::address::{native, native_url};

    fn here() -> Location {
        Location::file(native("/docs/a/index.md"))
    }

    fn navigate(path: &str, fragment: Option<&str>) -> Link {
        Link::Navigate(Target { location: Location::file(native(path)), fragment: fragment.map(str::to_string) })
    }

    #[test]
    fn relative_and_absolute_markdown_links_navigate() {
        assert_eq!(classify("../b/page.md#intro", &here()), navigate("/docs/b/page.md", Some("intro")));
        assert_eq!(classify("notes.md", &here()), navigate("/docs/a/notes.md", None));
        assert_eq!(classify("/abs/c.markdown", &here()), navigate("/abs/c.markdown", None));
        assert_eq!(classify(&native_url("/x/b.md"), &here()), navigate("/x/b.md", None));
        assert_eq!(classify("my%20notes.md", &here()), navigate("/docs/a/my notes.md", None));
    }

    #[test]
    #[cfg(windows)]
    fn windows_links_resolve_to_drive_paths() {
        let expected = |path: &str| Link::Navigate(Target { location: Location::file(path), fragment: None });
        assert_eq!(classify("file:///C:/x/b.md", &here()), expected("C:\\x\\b.md"));
        assert_eq!(classify("..\\b\\page.md", &here()), expected("C:\\docs\\b\\page.md"));
        assert_eq!(classify("/abs/c.md", &here()), expected("C:\\abs\\c.md"));
    }

    #[test]
    fn anchors_scroll() {
        assert_eq!(classify("#intro", &here()), Link::Scroll("intro".into()));
        assert_eq!(classify("#caf%C3%A9", &here()), Link::Scroll("café".into()));
        assert_eq!(classify("#", &here()), Link::Unsupported);
    }

    fn web(url: &str) -> Location {
        Location::web(url::Url::parse(url).unwrap())
    }

    fn on_the_web(url: &str, fragment: Option<&str>) -> Link {
        Link::Navigate(Target { location: web(url), fragment: fragment.map(str::to_string) })
    }

    #[test]
    fn markdown_links_on_the_web_open_in_the_app() {
        assert_eq!(classify("https://e.com/p.md", &here()), on_the_web("https://e.com/p.md", None));
        assert_eq!(classify("HTTP://X.ORG/A.MD#Top", &here()), on_the_web("http://x.org/A.MD", Some("Top")));
    }

    #[test]
    fn other_web_and_mail_links_leave_the_app() {
        for href in ["https://example.com", "http://x.org/a.txt", "mailto:a@b.co", "HTTPS://EXAMPLE.COM"] {
            assert_eq!(classify(href, &here()), Link::External(href.into()), "{href}");
        }
    }

    #[test]
    fn everything_else_is_unsupported() {
        let hrefs = [
            "",
            "  ",
            "ftp://x.org/f",
            "javascript:alert(1)",
            "tel:123",
            "data:text/html,hi",
            "file:///etc/passwd",
            "data.txt",
            "pic.png",
            "//host/x.md",
            "\\\\host\\share\\x.md",
            "\\/host/x.md",
            "file:////host/share/x.md",
        ];
        for href in hrefs {
            assert_eq!(classify(href, &here()), Link::Unsupported, "{href:?}");
        }
    }
}
