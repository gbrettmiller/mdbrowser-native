//! Links on a web page: resolved as a browser would, and never allowed to reach
//! the reader's disk, network shares, or (from a public page) local network.

use url::Url;

use super::{Link, MAIL_SCHEME};
use crate::session::address::{scheme_of, web_target};
use crate::session::href::{OnPage, join_on_page};
use crate::session::network::crosses_into_local_network;

/// Resolved as a browser would against the page's URL (`href::join_on_page`). Any web link is tried
/// in the app, which finds its Markdown (ADR-005) or offers the system browser; only mail leaves.
pub(super) fn classify(reference: &str, page: &Url) -> Link {
    if scheme_of(reference).as_deref() == Some(MAIL_SCHEME) {
        return Link::External(reference.to_string());
    }
    match join_on_page(reference, page) {
        OnPage::Disk => Link::LocalFromWeb,
        OnPage::Unsupported => Link::Unsupported,
        OnPage::Web(url) if crosses_into_local_network(page, &url) => Link::LocalNetworkFromWeb,
        OnPage::Web(url) => Link::Navigate(web_target(url)),
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::session::address::native;
    use crate::session::links::{Link, classify};
    use crate::session::location::{Location, Target, WEB_SCHEMES};

    fn here() -> Location {
        Location::file(native("/docs/a/index.md"))
    }

    fn web(url: &str) -> Location {
        Location::web(Url::parse(url).unwrap())
    }

    fn on_the_web(url: &str, fragment: Option<&str>) -> Link {
        Link::Navigate(Target { location: web(url), fragment: fragment.map(str::to_string) })
    }

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
            ("HTTP://X.ORG/A.MD", "http://x.org/A.MD", None),
            ("https://y.org/a.md?x=1#f", "https://y.org/a.md?x=1", Some("f")),
            ("notes.markdown#caf%C3%A9", "https://x.org/docs/a/notes.markdown", Some("café")),
            ("  b.md  ", "https://x.org/docs/a/b.md", None),
            ("https://user:pw@y.org/p.md", "https://y.org/p.md", None),
        ] {
            assert_eq!(from_page(href), on_the_web(url, fragment), "{href}");
        }
    }

    #[test]
    fn other_web_pages_open_in_the_app_to_find_their_markdown() {
        for (href, url) in [
            ("page.html", "https://x.org/docs/a/page.html"),
            ("/", "https://x.org/"),
            ("/fail", "https://x.org/fail"),
            ("/llms.txt", "https://x.org/llms.txt"),
            ("https://x.org/about#team", "https://x.org/about"),
            ("https://u:p@x.org/about", "https://x.org/about"),
            ("https://e.com/about", "https://e.com/about"),
            ("//other.host/", "https://other.host/"),
        ] {
            assert_eq!(from_page(href), on_the_web(url, href.split_once('#').map(|(_, f)| f)), "{href}");
        }
    }

    #[test]
    fn only_mail_on_a_page_leaves_the_app() {
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
            "http://localhost:8080/",
            "http://127.0.0.1/page",
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
        // Names that only mean something at home count as local too.
        for href in ["//router/x.md", "//printer.local/x.md", "//nas.lan/x.md", "//100.100.100.100/x.md"] {
            assert_eq!(from_page(href), Link::LocalNetworkFromWeb, "{href:?}");
        }
    }

    #[test]
    fn local_pages_and_files_may_open_the_local_network() {
        let local = web("http://127.0.0.1:8000/index.md");
        assert_eq!(classify("/b.md", &local), on_the_web("http://127.0.0.1:8000/b.md", None));
        assert_eq!(classify("http://192.168.1.2/b.md", &local), on_the_web("http://192.168.1.2/b.md", None));
        assert_eq!(classify("http://localhost/b.md", &here()), on_the_web("http://localhost/b.md", None));
    }

    #[test]
    fn credentials_never_reach_the_address_bar_or_the_system() {
        let spoof = "https://yourbank.com@evil.example/login";
        assert_eq!(from_page(spoof), on_the_web("https://evil.example/login", None));
        assert_eq!(classify(spoof, &here()), on_the_web("https://evil.example/login", None));
        assert_eq!(classify("https://u:p@e.com/a", &here()), on_the_web("https://e.com/a", None));
        assert_eq!(classify("mailto:u:p@e.com", &here()), Link::External("mailto:u:p@e.com".into()));
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
