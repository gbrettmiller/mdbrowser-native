//! What following a link inside a document should do.

use std::path::Path;

use super::address::{decode, is_network_path, normalise, parse_file_url, scheme_of, split_reference};
use super::location::{Location, Target, is_markdown};

const EXTERNAL_SCHEMES: [&str; 3] = ["http", "https", "mailto"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Link {
    /// Another Markdown document: browse to it.
    Navigate(Target),
    /// An anchor in the current document.
    Scroll(String),
    /// http, https or mailto: hand to the system.
    External(String),
    /// Everything else: other schemes, non-Markdown files, malformed refs.
    Unsupported,
}

/// A local reference (relative, absolute or `file://`) resolved against the current document.
fn local_target(reference: &str, current: &Location) -> Option<(String, Option<String>)> {
    if is_network_path(reference) {
        return None; // protocol-relative or a network share: not a local file
    }
    let (path, fragment) = match scheme_of(reference).as_deref() {
        Some("file") => parse_file_url(reference)?,
        Some(_) => return None,
        None => split_reference(reference),
    };
    if path.is_empty() || is_network_path(&path) {
        return None;
    }
    let directory = current.as_file()?.parent().unwrap_or(Path::new("/")); // web bases arrive in step 8
    Some((directory.join(path).display().to_string(), fragment))
}

pub fn classify(href: &str, current: &Location) -> Link {
    let reference = href.trim();
    if let Some(anchor) = reference.strip_prefix('#') {
        let fragment = decode(anchor);
        return if fragment.is_empty() { Link::Unsupported } else { Link::Scroll(fragment) };
    }
    if scheme_of(reference).is_some_and(|scheme| EXTERNAL_SCHEMES.contains(&scheme.as_str())) {
        return Link::External(reference.to_string());
    }
    match local_target(reference, current) {
        Some((path, fragment)) if is_markdown(Path::new(&path)) => {
            Link::Navigate(Target { location: Location::file(normalise(Path::new(&path))), fragment })
        }
        _ => Link::Unsupported,
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

    #[test]
    fn only_web_and_mail_links_leave_the_app() {
        for href in ["https://example.com", "http://x.org/a.md", "mailto:a@b.co", "HTTPS://EXAMPLE.COM"] {
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
