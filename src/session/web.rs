//! What to ask a web server for, and what its answers mean. Pure: `services::web`
//! sends the request and follows these rules.

use std::net::{Ipv4Addr, Ipv6Addr};

use url::{Host, Url};

use super::error::ErrorCode;
use super::location::is_markdown_name;

/// Markdown first; plain text next, since many hosts serve `.md` files as `text/plain`.
pub const ACCEPT: &str = "text/markdown, text/x-markdown;q=0.9, text/plain;q=0.5, */*;q=0.1";

/// Redirects followed before giving up.
pub const MAX_REDIRECTS: u32 = 5;

const MARKDOWN_TYPES: [&str; 2] = ["text/markdown", "text/x-markdown"];
/// Types that say nothing about the format, so the path decides.
const PLAIN_TYPES: [&str; 2] = ["", "text/plain"];
/// Charsets that decode as UTF-8 (`utf8` is a common misspelling).
const READABLE_CHARSETS: [&str; 3] = ["utf-8", "utf8", "us-ascii"];
const REDIRECTS: [u16; 5] = [301, 302, 303, 307, 308];
const GONE: [u16; 2] = [404, 410];
const WEB_SCHEMES: [&str; 2] = ["http", "https"];

/// A status that isn't a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reply {
    Body,
    Redirect,
}

pub fn classify_status(status: u16) -> Result<Reply, ErrorCode> {
    match status {
        200..=299 => Ok(Reply::Body),
        _ if REDIRECTS.contains(&status) => Ok(Reply::Redirect),
        _ if GONE.contains(&status) => Err(ErrorCode::NotFound),
        _ => Err(ErrorCode::ServerError),
    }
}

/// Where redirect number `hop` (from 1) leads. Never to another scheme than
/// http(s), never from https down to http, never from a public host into the
/// reader's own machine or network, and never carrying credentials.
pub fn next_hop(from: &Url, location: Option<&str>, hop: u32) -> Result<Url, ErrorCode> {
    let location = location.map(str::trim).filter(|location| !location.is_empty());
    if hop > MAX_REDIRECTS {
        return Err(ErrorCode::BadRedirect);
    }
    let mut to = from.join(location.ok_or(ErrorCode::BadRedirect)?).map_err(|_| ErrorCode::BadRedirect)?;
    let downgrade = from.scheme() == "https" && to.scheme() == "http";
    // A link to a private address is visible before the click (ADR-003); a redirect to one is not.
    let into_local_network = is_local_network(&to) && !is_local_network(from);
    if !WEB_SCHEMES.contains(&to.scheme()) || downgrade || into_local_network {
        return Err(ErrorCode::BadRedirect);
    }
    // Infallible for http(s) URLs, which always have a host.
    let _ = to.set_username("");
    let _ = to.set_password(None);
    Ok(to)
}

/// Loopback, private, link-local and unspecified hosts. The url crate has already
/// normalised spellings like `0x7f.1` to `127.0.0.1`. Names that merely resolve to
/// such addresses can only be caught when connecting.
fn is_local_network(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(name)) => {
            let name = name.trim_end_matches('.');
            name == "localhost" || name.ends_with(".localhost")
        }
        Some(Host::Ipv4(ip)) => is_local_ipv4(ip),
        Some(Host::Ipv6(ip)) => ip.to_ipv4_mapped().map_or_else(|| is_local_ipv6(ip), is_local_ipv4),
        None => false,
    }
}

fn is_local_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_loopback() || ip.is_private() || ip.is_link_local() || ip.is_unspecified()
}

fn is_local_ipv6(ip: Ipv6Addr) -> bool {
    ip.is_loopback() || ip.is_unspecified() || ip.is_unique_local() || ip.is_unicast_link_local()
}

/// Whether a successful response at `final_url` is a Markdown document we can read,
/// judged by its headers alone. The caller must still decode the bytes strictly.
pub fn body_policy(content_type: Option<&str>, final_url: &Url) -> Result<(), ErrorCode> {
    let mut parts = content_type.unwrap_or_default().split(';');
    let media = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
    let markdown = MARKDOWN_TYPES.contains(&media.as_str())
        || (PLAIN_TYPES.contains(&media.as_str()) && is_markdown_name(final_url.path()));
    if !markdown {
        return Err(ErrorCode::NotMarkdown);
    }
    // Every charset parameter must be readable, so a decoy can't vouch for the real one.
    let readable = parts
        .filter_map(|param| param.split_once('='))
        .filter(|(name, _)| name.trim().eq_ignore_ascii_case("charset"))
        .all(|(_, value)| READABLE_CHARSETS.contains(&value.trim().trim_matches('"').to_ascii_lowercase().as_str()));
    if readable { Ok(()) } else { Err(ErrorCode::Unreadable) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    fn body(content_type: Option<&str>, at: &str) -> Result<(), ErrorCode> {
        body_policy(content_type, &url(at))
    }

    const PAGE: &str = "https://example.com/page";

    #[test]
    fn markdown_types_are_markdown_at_any_path() {
        for content_type in ["text/markdown; charset=utf-8", "Text/Markdown", "text/x-markdown"] {
            assert_eq!(body(Some(content_type), PAGE), Ok(()), "{content_type}");
        }
    }

    #[test]
    fn plain_or_missing_types_are_markdown_only_at_markdown_paths() {
        assert_eq!(body(Some("text/plain ; charset=UTF-8"), "https://example.com/a.md"), Ok(()));
        assert_eq!(body(Some("text/plain"), "https://example.com/a.md?v=1"), Ok(()));
        assert_eq!(body(Some("text/plain"), "https://example.com/notes.markdown"), Ok(()));
        assert_eq!(body(None, "https://example.com/A.MD"), Ok(()));
        assert_eq!(body(Some(""), "https://example.com/a.md"), Ok(()));
        for at in ["https://example.com/a.txt", "https://example.com/a.md/", "https://example.com/readme"] {
            assert_eq!(body(Some("text/plain; charset=utf-8"), at), Err(ErrorCode::NotMarkdown), "{at}");
        }
        assert_eq!(body(None, "https://example.com/"), Err(ErrorCode::NotMarkdown));
    }

    #[test]
    fn other_types_are_not_markdown_even_at_markdown_paths() {
        for content_type in ["text/html", "application/json", "application/octet-stream", "text/markdownish"] {
            assert_eq!(
                body(Some(content_type), "https://example.com/a.md"),
                Err(ErrorCode::NotMarkdown),
                "{content_type}"
            );
        }
        assert_eq!(body(Some("text/html; charset=latin1"), PAGE), Err(ErrorCode::NotMarkdown));
    }

    #[test]
    fn only_utf8_and_ascii_charsets_are_readable() {
        for content_type in [
            "text/markdown; charset=utf-8",
            "text/markdown; charset=\"UTF-8\"",
            "text/markdown;CHARSET=us-ascii",
            "text/markdown; charset=utf8",
            "text/markdown; format=fixed; charset=utf-8",
            "text/markdown; charset",
        ] {
            assert_eq!(body(Some(content_type), PAGE), Ok(()), "{content_type}");
        }
        for content_type in [
            "text/markdown; charset=iso-8859-1",
            "text/markdown; format=fixed; charset=latin1",
            "text/markdown; charset=",
            "text/markdown; charset='utf-8'",
            "text/markdown; x=\"; charset=utf-8\"; charset=iso-8859-1",
        ] {
            assert_eq!(body(Some(content_type), PAGE), Err(ErrorCode::Unreadable), "{content_type}");
        }
        assert_eq!(
            body(Some("text/plain; charset=iso-8859-1"), "https://example.com/a.md"),
            Err(ErrorCode::Unreadable)
        );
    }

    #[test]
    fn every_2xx_has_a_body_to_read() {
        for code in [200, 203, 204, 206, 299] {
            assert_eq!(classify_status(code), Ok(Reply::Body), "{code}");
        }
    }

    #[test]
    fn five_3xx_statuses_redirect() {
        for code in [301, 302, 303, 307, 308] {
            assert_eq!(classify_status(code), Ok(Reply::Redirect), "{code}");
        }
    }

    #[test]
    fn missing_pages_are_not_found_and_other_statuses_are_server_errors() {
        for code in [404, 410] {
            assert_eq!(classify_status(code), Err(ErrorCode::NotFound), "{code}");
        }
        for code in [100, 199, 300, 304, 400, 401, 403, 418, 429, 500, 503] {
            assert_eq!(classify_status(code), Err(ErrorCode::ServerError), "{code}");
        }
    }

    fn hop(from: &str, location: Option<&str>, number: u32) -> Result<String, ErrorCode> {
        next_hop(&url(from), location, number).map(String::from)
    }

    #[test]
    fn redirects_resolve_against_the_request_url() {
        let from = "https://a.com/x/y.md?v=1";
        for (location, to) in [
            ("z.md", "https://a.com/x/z.md"),
            ("../z.md", "https://a.com/z.md"),
            ("/z.md", "https://a.com/z.md"),
            ("?v=2", "https://a.com/x/y.md?v=2"),
            ("//b.com/y.md", "https://b.com/y.md"),
            ("https://b.com:8443/y.md", "https://b.com:8443/y.md"),
            ("  z.md ", "https://a.com/x/z.md"),
        ] {
            assert_eq!(hop(from, Some(location), 1), Ok(to.into()), "{location}");
        }
    }

    #[test]
    fn redirects_stay_on_the_web_and_never_downgrade() {
        assert_eq!(hop("https://a.com/y.md", Some("https://b.com/y.md"), 1), Ok("https://b.com/y.md".into()));
        assert_eq!(hop("http://a.com/y.md", Some("https://b.com/y.md"), 1), Ok("https://b.com/y.md".into()));
        assert_eq!(hop("http://a.com/y.md", Some("http://b.com/y.md"), 1), Ok("http://b.com/y.md".into()));
        let from = "https://a.com/y.md";
        for location in
            ["http://a.com/y.md", "HTTP://a.com/y.md", "file:///etc/passwd", "ftp://a.com/y.md", "javascript:alert(1)"]
        {
            assert_eq!(hop(from, Some(location), 1), Err(ErrorCode::BadRedirect), "{location}");
        }
    }

    #[test]
    fn missing_empty_or_broken_locations_are_refused() {
        for location in [None, Some(""), Some("  "), Some("http://[bad")] {
            assert_eq!(hop("https://a.com/y.md", location, 1), Err(ErrorCode::BadRedirect), "{location:?}");
        }
    }

    #[test]
    fn redirects_stop_after_the_limit() {
        let from = "https://a.com/y.md";
        assert_eq!(hop(from, Some("z.md"), MAX_REDIRECTS), Ok("https://a.com/z.md".into()));
        assert_eq!(hop(from, Some("z.md"), MAX_REDIRECTS + 1), Err(ErrorCode::BadRedirect));
    }

    #[test]
    fn credentials_are_never_carried_by_a_redirect() {
        let from = "https://a.com/y.md";
        for location in ["https://user:pw@b.com/z.md", "https://user@b.com/z.md", "//user:pw@b.com/z.md"] {
            assert_eq!(hop(from, Some(location), 1), Ok("https://b.com/z.md".into()), "{location}");
        }
        assert_eq!(hop("https://u:p@a.com/x/y.md", Some("z.md"), 1), Ok("https://a.com/x/z.md".into()));
    }

    #[test]
    fn public_pages_cannot_redirect_into_the_local_network() {
        let from = "http://example.com/a.md";
        for location in [
            "http://127.0.0.1:6379/x.md",
            "http://0x7f.1/x.md",
            "http://2130706433/x.md",
            "http://localhost/x.md",
            "http://LOCALHOST./x.md",
            "http://admin.localhost/x.md",
            "http://10.0.0.1/x.md",
            "http://172.16.0.1/x.md",
            "http://192.168.1.1/x.md",
            "http://169.254.169.254/latest/meta-data",
            "http://0.0.0.0/x.md",
            "http://[::1]/x.md",
            "http://[::]/x.md",
            "http://[fd00::1]/x.md",
            "http://[fe80::1]/x.md",
            "http://[::ffff:127.0.0.1]/x.md",
        ] {
            assert_eq!(hop(from, Some(location), 1), Err(ErrorCode::BadRedirect), "{location}");
        }
    }

    #[test]
    fn local_pages_may_redirect_within_the_local_network() {
        assert_eq!(hop("http://localhost:8000/a.md", Some("/b.md"), 1), Ok("http://localhost:8000/b.md".into()));
        assert_eq!(hop("http://192.168.1.2/a.md", Some("http://10.0.0.1/b.md"), 1), Ok("http://10.0.0.1/b.md".into()));
        assert_eq!(
            hop("http://127.0.0.1/a.md", Some("https://example.com/b.md"), 1),
            Ok("https://example.com/b.md".into())
        );
    }

    /// q of each media range in a header like `a/b, c/d;q=0.5`, in order.
    fn weights(header: &str) -> Vec<(String, f32)> {
        header
            .split(',')
            .map(|range| {
                let mut parts = range.split(';').map(str::trim);
                let media = parts.next().expect("a media range").to_string();
                let q = parts.find_map(|p| p.strip_prefix("q=")).map_or(1.0, |q| q.parse().expect("a q value"));
                (media, q)
            })
            .collect()
    }

    #[test]
    fn accept_ranks_markdown_then_plain_text_then_anything() {
        let weights = weights(ACCEPT);
        let order: Vec<_> = weights.iter().map(|(media, _)| media.as_str()).collect();
        assert_eq!(order, ["text/markdown", "text/x-markdown", "text/plain", "*/*"]);
        assert!(weights.windows(2).all(|pair| pair[0].1 > pair[1].1), "{weights:?}");
    }
}
