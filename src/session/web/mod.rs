//! What to ask a web server for, and what its answers mean. Pure: `services::web`
//! sends the request and follows these rules.

mod body;

pub use body::{body_policy, encoding_policy, to_text};
use url::Url;

use super::error::ErrorCode;
use super::location::{WEB_SCHEMES, without_credentials};
use super::network::is_local_network;

/// Markdown first; plain text next, since many hosts serve `.md` files as `text/plain`.
pub const ACCEPT: &str = "text/markdown, text/x-markdown;q=0.9, text/plain;q=0.5, */*;q=0.1";

/// Redirects followed before giving up.
pub const MAX_REDIRECTS: u32 = 5;

const REDIRECT_STATUSES: [u16; 5] = [301, 302, 303, 307, 308];
const NOT_FOUND_STATUSES: [u16; 2] = [404, 410];

/// A status that isn't a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusClass {
    Body,
    Redirect,
}

pub fn classify_status(status: u16) -> Result<StatusClass, ErrorCode> {
    match status {
        200..=299 => Ok(StatusClass::Body),
        _ if REDIRECT_STATUSES.contains(&status) => Ok(StatusClass::Redirect),
        _ if NOT_FOUND_STATUSES.contains(&status) => Err(ErrorCode::NotFound),
        _ => Err(ErrorCode::ServerError),
    }
}

/// Where redirect number `hop` (from 1) leads. Never to another scheme than
/// http(s), never from https down to http, never from a public host into the
/// reader's own machine or network, and never carrying credentials.
pub fn next_hop(from: &Url, location_header: Option<&str>, hop: u32) -> Result<Url, ErrorCode> {
    let location = location_header.map(str::trim).filter(|location| !location.is_empty());
    if hop > MAX_REDIRECTS {
        return Err(ErrorCode::BadRedirect);
    }
    let to = from.join(location.ok_or(ErrorCode::BadRedirect)?).map_err(|_| ErrorCode::BadRedirect)?;
    let downgrade = from.scheme() == "https" && to.scheme() == "http";
    // Links and redirects share this refusal (ADR-003): a public page must not steer
    // the app into the reader's own machine or network.
    let into_local_network = is_local_network(&to) && !is_local_network(from);
    if !WEB_SCHEMES.contains(&to.scheme()) || downgrade || into_local_network {
        return Err(ErrorCode::BadRedirect);
    }
    Ok(without_credentials(to))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    #[test]
    fn every_2xx_has_a_body_to_read() {
        for code in [200, 203, 204, 206, 299] {
            assert_eq!(classify_status(code), Ok(StatusClass::Body), "{code}");
        }
    }

    #[test]
    fn five_3xx_statuses_redirect() {
        for code in [301, 302, 303, 307, 308] {
            assert_eq!(classify_status(code), Ok(StatusClass::Redirect), "{code}");
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
