//! What following a link inside a document should do. A link is read against
//! the kind of document it is in: a file may link to files and the web; a web
//! page may link only to the web, never to the reader's disk.

use std::path::Path;

use url::Url;

use super::address::{
    WEB_SCHEMES, decode, is_network_path, normalise, parse_file_url, scheme_of, split_reference, web_target,
    without_credentials,
};
use super::location::{Location, Target, is_markdown, is_markdown_name};
use super::web::is_local_network;

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
        Location::Web(page) => from_page(reference, page),
    }
}

/// The href as a browser's URL parser reads it: whitespace and control characters
/// trimmed from the ends, tabs and newlines removed throughout. Checking this form
/// keeps the checks below from seeing a different string than `Url::join` does.
fn as_browsers_read(href: &str) -> String {
    href.trim().trim_matches(|c: char| c <= ' ').chars().filter(|c| !matches!(c, '\t' | '\n' | '\r')).collect()
}

/// For the system browser: the href as written, unless it carries credentials.
fn external(reference: &str, url: Url) -> Link {
    let spoofable = !url.username().is_empty() || url.password().is_some();
    Link::External(if spoofable { without_credentials(url).to_string() } else { reference.to_string() })
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

/// A local reference (relative, absolute or `file://`) resolved against the current file.
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
    let directory = current.as_file()?.parent().unwrap_or(Path::new("/"));
    Some((directory.join(path).display().to_string(), fragment))
}

/// Resolved as a browser would against the page's URL. Anything shaped like a
/// disk path is refused before resolving, since the URL parser would quietly
/// read `\\host\x` or `C:\x` as a path on the same site.
fn from_page(reference: &str, page: &Url) -> Link {
    if is_disk_shaped(reference) {
        return Link::LocalFromWeb;
    }
    match scheme_of(reference).as_deref() {
        Some(MAIL_SCHEME) => return Link::External(reference.to_string()),
        Some(scheme) if !WEB_SCHEMES.contains(&scheme) && scheme != "file" => return Link::Unsupported,
        _ if reference.is_empty() => return Link::Unsupported,
        _ => {}
    }
    // The boundary: whatever the checks above missed, only http(s) leaves this function.
    match page.join(reference) {
        Ok(url) if url.scheme() == "file" => Link::LocalFromWeb,
        Ok(url) if !WEB_SCHEMES.contains(&url.scheme()) => Link::Unsupported,
        Ok(url) if !is_markdown_name(url.path()) => Link::External(without_credentials(url).to_string()),
        Ok(url) if is_local_network(&url) && !is_local_network(page) => Link::LocalNetworkFromWeb,
        Ok(url) => Link::Navigate(web_target(url)),
        Err(_) => Link::Unsupported,
    }
}

/// A backslash start (`\\host\x`, `\x`), a slash-backslash share (`/\host`), or a
/// drive letter (`C:\x`, `c:/x`, `c:x`). `//host` is a web address, not a share.
fn is_disk_shaped(reference: &str) -> bool {
    let bytes = reference.as_bytes();
    let drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    let share = is_network_path(reference) && !reference.starts_with("//");
    reference.starts_with('\\') || share || drive
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

    // ── From a page on the web ──────────────────────────────────────────────

    const PAGE: &str = "https://x.org/docs/a/index.md";

    fn from_page(href: &str) -> Link {
        classify(href, &web(PAGE))
    }

    #[test]
    fn markdown_links_resolve_against_the_page() {
        for (href, url, fragment) in [
            ("b.md", "https://x.org/docs/a/b.md", None),
            ("../index.md", "https://x.org/docs/index.md", None),
            ("/abs.md", "https://x.org/abs.md", None),
            ("//other.host/p.md", "https://other.host/p.md", None),
            ("HTTP://X/A.MD", "http://x/A.MD", None),
            ("https://x/a.md?x=1#f", "https://x/a.md?x=1", Some("f")),
            ("notes.markdown#caf%C3%A9", "https://x.org/docs/a/notes.markdown", Some("café")),
            ("  b.md  ", "https://x.org/docs/a/b.md", None),
            ("https://user:pw@y.org/p.md", "https://y.org/p.md", None),
        ] {
            assert_eq!(from_page(href), on_the_web(url, fragment), "{href}");
        }
    }

    #[test]
    fn other_links_on_a_page_leave_the_app_as_full_urls() {
        assert_eq!(from_page("page.html"), Link::External("https://x.org/docs/a/page.html".into()));
        assert_eq!(from_page("/"), Link::External("https://x.org/".into()));
        assert_eq!(from_page("https://e.com/about"), Link::External("https://e.com/about".into()));
        assert_eq!(from_page("mailto:a@b.co"), Link::External("mailto:a@b.co".into()));
    }

    #[test]
    fn anchors_on_a_page_scroll() {
        assert_eq!(from_page("#x"), Link::Scroll("x".into()));
    }

    #[test]
    fn a_page_can_never_reach_the_disk() {
        for href in
            ["file:///etc/x.md", "FILE:///etc/x.md", "C:\\x.md", "c:/x.md", "\\\\host\\x.md", "\\/host/x.md", "\\x.md"]
        {
            assert_eq!(from_page(href), Link::LocalFromWeb, "{href:?}");
        }
    }

    #[test]
    fn other_schemes_on_a_page_are_unsupported() {
        for href in ["javascript:alert(1)", "data:text/markdown,hi", "ftp://x.org/a.md", "tel:1", "", "  ", "#"] {
            assert_eq!(from_page(href), Link::Unsupported, "{href:?}");
        }
    }

    #[test]
    fn noise_a_browser_ignores_does_not_hide_a_disk_path() {
        for href in [
            "\x01\\\\host\\x.md",
            "\x00C:\\x.md",
            "C\t:\\x.md",
            "/\\host/x.md",
            "\\\\?\\UNC\\host\\share\\x.md",
            "fi\tle:///x.md",
            "file\n:///x.md",
            "f\rile:///x.md",
            "\x00file:///x.md",
            "\x1ffile:///x.md",
            "file:\\\\host\\share\\x.md",
            "file:",
            "FILE:",
        ] {
            assert_eq!(from_page(href), Link::LocalFromWeb, "{href:?}");
        }
    }

    #[test]
    fn a_public_page_cannot_open_the_local_network_in_the_app() {
        for href in [
            "http://127.0.0.1/x.md",
            "//2130706433/x.md",
            "//0x7f.1/x.md",
            "//[::ffff:127.0.0.1]/x.md",
            "//localhost./x.md",
            "http://admin.localhost/x.md",
            "http:\\\\192.168.0.1\\a.md",
            "//10.0.0.1/a.md",
            "//169.254.169.254/latest.md",
            "//[fe80::1]/x.md",
        ] {
            assert_eq!(from_page(href), Link::LocalNetworkFromWeb, "{href:?}");
        }
        // The system browser applies its own rules; only in-app fetching is refused.
        assert_eq!(from_page("http://localhost:8080/"), Link::External("http://localhost:8080/".into()));
    }

    #[test]
    fn local_pages_and_files_may_open_the_local_network() {
        let local = web("http://127.0.0.1:8000/index.md");
        assert_eq!(classify("/b.md", &local), on_the_web("http://127.0.0.1:8000/b.md", None));
        assert_eq!(classify("http://192.168.1.2/b.md", &local), on_the_web("http://192.168.1.2/b.md", None));
        assert_eq!(classify("http://localhost/b.md", &here()), on_the_web("http://localhost/b.md", None));
    }

    #[test]
    fn credentials_never_reach_the_system_browser() {
        let spoof = "https://yourbank.com@evil.example/login";
        assert_eq!(from_page(spoof), Link::External("https://evil.example/login".into()));
        assert_eq!(classify(spoof, &here()), Link::External("https://evil.example/login".into()));
        assert_eq!(classify("https://u:p@e.com/a", &here()), Link::External("https://e.com/a".into()));
    }

    /// Whatever a page links to: in-app links are plain http(s) with no credentials,
    /// and only http(s) or mail goes to the system.
    #[test]
    fn links_on_a_page_keep_their_guarantees() {
        let hrefs = [
            "a.md",
            "../../../../etc/passwd.md",
            "/etc/hosts.md",
            "//localhost/x.md",
            "///x.md",
            "////x.md",
            "file:x.md",
            "file:/x.md",
            "file://localhost/x.md",
            "fILe:///x.md",
            " file:///x.md",
            "\tfile:///x.md",
            "C|/x.md",
            "c:x.md",
            "Z:\\x.md",
            "\\\\?\\C:\\x.md",
            "\\\\.\\pipe\\x.md",
            "/\\host/x.md",
            "%2e%2e/x.md",
            "..%2fx.md",
            "x.md%00.txt",
            "?q.md",
            "#x.md",
            "https:x.md",
            "http:/x.md",
            "https:///x.md",
            "~/x.md",
            "~",
            ".",
            "..",
            "/",
            "x",
            "x.MD",
            "x.markdown/",
            "blob:https://x.org/1",
            "about:blank",
            "view-source:https://x.org/a.md",
            "jar:file:///x.md!/a.md",
            "smb://host/x.md",
            "ssh://host/x.md",
            "http:\\\\host\\x.md",
            "https\t://x/a.md",
            "//2130706433/x.md",
            "//[::ffff:127.0.0.1]/x.md",
            "//[fe80::1%25eth0]/x.md",
            "mail\tto:a@b.co?attach=/etc/passwd",
            "MAILTO:a@b.co?attach=x",
            "https://bank.com@evil.example/x.md",
            "https://bank.com@evil.example/x",
            "http://x.org/a.md",
            "\x01\\\\host\\x.md",
            "fi\tle:///x.md",
            "\x00file:///x.md",
            "javascript:alert(1)",
            "data:,x",
        ];
        assert!(hrefs.len() >= 50);
        for href in hrefs {
            match from_page(href) {
                Link::Navigate(Target { location: Location::Web(url), .. }) => {
                    assert!(WEB_SCHEMES.contains(&url.scheme()), "{href:?} → {url}");
                    assert!(url.username().is_empty() && url.password().is_none(), "{href:?} → {url}");
                }
                Link::Navigate(target) => panic!("{href:?} → {target:?}"),
                Link::External(url) if url.to_ascii_lowercase().starts_with("mailto:") => {}
                Link::External(url) => {
                    let parsed = Url::parse(&url).unwrap_or_else(|_| panic!("{href:?} → {url}"));
                    assert!(WEB_SCHEMES.contains(&parsed.scheme()), "{href:?} → {url}");
                    assert!(parsed.username().is_empty() && parsed.password().is_none(), "{href:?} → {url}");
                }
                Link::Scroll(_) | Link::LocalFromWeb | Link::LocalNetworkFromWeb | Link::Unsupported => {}
            }
        }
    }
}
