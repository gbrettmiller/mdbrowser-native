//! What to ask a web server for, and what its answers mean. Pure: `services::web`
//! sends the request and follows these rules.

mod body;

pub use body::{body_policy, encoding_policy, image_policy, to_text};
use url::Url;

use super::error::ErrorCode;
use super::location::{Location, WEB_SCHEMES, without_credentials};
use super::network::{crosses_into_local_network, is_local_network};

/// Markdown first; plain text next, since many hosts serve `.md` files as `text/plain`.
pub const ACCEPT: &str = "text/markdown, text/x-markdown;q=0.9, text/plain;q=0.5, */*;q=0.1";

/// The formats we decode first (ADR-004), then any image, then anything at all.
pub const IMAGE_ACCEPT: &str = "image/png, image/jpeg, image/gif, image/webp, image/*;q=0.8, */*;q=0.1";

/// The Fetch standard's blocked ports, plus Redis (6379), which it doesn't list.
const BAD_PORTS: [u16; 82] = [
    1, 7, 9, 11, 13, 15, 17, 19, 20, 21, 22, 23, 25, 37, 42, 43, 53, 69, 77, 79, 87, 95, 101, 102, 103, 104, 109, 110,
    111, 113, 115, 117, 119, 123, 135, 137, 139, 143, 161, 179, 389, 427, 465, 512, 513, 514, 515, 526, 530, 531, 532,
    540, 548, 554, 556, 563, 587, 601, 636, 989, 990, 993, 995, 1719, 1720, 1723, 2049, 3659, 4045, 4190, 5060, 5061,
    6000, 6379, 6566, 6665, 6666, 6667, 6668, 6669, 6679, 6697,
];

/// What a fetch is for: it decides the `Accept` header and which types are taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Want {
    Page,
    Image,
}

impl Want {
    pub const fn accept(self) -> &'static str {
        match self {
            Self::Page => ACCEPT,
            Self::Image => IMAGE_ACCEPT,
        }
    }

    /// Whether a response of this type is taken, from `final_url` after redirects.
    pub fn check_type(self, content_type: Option<&str>, final_url: &Url) -> Result<(), FetchError> {
        match self {
            Self::Page => body_policy(content_type, final_url),
            Self::Image => image_policy(content_type),
        }
    }
}

/// Who asked for a fetch. A public page's links and images must not be steered into
/// the reader's own network by a name that resolves there; the reader's own
/// requests (typed, from History, from a local file or page) may go anywhere.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Origin {
    #[default]
    Trusted,
    Public,
}

impl Origin {
    /// Where a link or image found in the document at `current` comes from.
    pub fn of(current: &Location) -> Self {
        match current {
            Location::Web(url) if !is_local_network(url) => Origin::Public,
            Location::Welcome => Origin::Public,
            _ => Origin::Trusted,
        }
    }
}

/// Whether hop `hop` (from 1) to `url`, in a fetch that asked for `asked`, must not
/// connect to a local address. Written-local hosts were already allowed or refused
/// by the link and redirect rules. A public origin is always checked. The reader's
/// own request may redirect within the host they asked for (http to https, another
/// path); a redirect to any other host is the site's choice, so it is checked.
pub fn checks_resolved_address(origin: Origin, hop: u32, url: &Url, asked: &Url) -> bool {
    let site_chose = hop > 1 && url.host_str() != asked.host_str();
    (origin == Origin::Public || site_chose) && !is_local_network(url)
}

/// Why a fetch failed, before the caller says what that means to the reader.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchError {
    NotFound,
    Server,
    Unreachable,
    BadRedirect,
    /// Over the size cap, counted after decompression.
    TooLarge,
    /// A type this fetch doesn't take.
    WrongType,
    /// Cut off, badly compressed, or not readable as what was wanted.
    Undecodable,
    /// A public name that resolved into the reader's own machine or network.
    LocalAddress,
}

impl FetchError {
    /// What a page shows; the same codes as before images (phase 2a).
    pub const fn for_page(self) -> ErrorCode {
        match self {
            Self::NotFound => ErrorCode::NotFound,
            Self::Server => ErrorCode::ServerError,
            Self::Unreachable => ErrorCode::Unreachable,
            Self::BadRedirect => ErrorCode::BadRedirect,
            Self::TooLarge | Self::Undecodable => ErrorCode::Unreadable,
            Self::WrongType => ErrorCode::NotMarkdown,
            Self::LocalAddress => ErrorCode::LocalAddress,
        }
    }
}

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

pub fn classify_status(status: u16) -> Result<StatusClass, FetchError> {
    match status {
        200..=299 => Ok(StatusClass::Body),
        _ if REDIRECT_STATUSES.contains(&status) => Ok(StatusClass::Redirect),
        _ if NOT_FOUND_STATUSES.contains(&status) => Err(FetchError::NotFound),
        _ => Err(FetchError::Server),
    }
}

/// A port browsers block, where mail, chat and other servers listen.
pub fn is_blocked_port(url: &Url) -> bool {
    url.port().is_some_and(|port| BAD_PORTS.contains(&port))
}

/// Where redirect number `hop` (from 1) leads. Never to another scheme than
/// http(s), never from https down to http, never from a public host into the
/// reader's own machine or network, never to a port browsers block, and never
/// carrying credentials.
pub fn next_hop(from: &Url, location_header: Option<&str>, hop: u32) -> Result<Url, FetchError> {
    let location = location_header.map(str::trim).filter(|location| !location.is_empty());
    if hop > MAX_REDIRECTS {
        return Err(FetchError::BadRedirect);
    }
    let to = from.join(location.ok_or(FetchError::BadRedirect)?).map_err(|_| FetchError::BadRedirect)?;
    let downgrade = from.scheme() == "https" && to.scheme() == "http";
    // Links, images and redirects share this refusal (ADR-003): a public page must
    // not steer the app into the reader's own machine or network.
    let into_local_network = crosses_into_local_network(from, &to);
    let bad_port = is_blocked_port(&to);
    if !WEB_SCHEMES.contains(&to.scheme()) || downgrade || into_local_network || bad_port {
        return Err(FetchError::BadRedirect);
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
    fn the_welcome_page_is_a_public_origin() {
        assert_eq!(Origin::of(&Location::Welcome), Origin::Public);
        assert_eq!(Origin::of(&Location::file("/d/a.md")), Origin::Trusted);
    }

    #[test]
    fn each_want_takes_its_own_types() {
        let page = url("https://x.org/a.md");
        assert_eq!(Want::Page.check_type(Some("text/markdown"), &page), Ok(()));
        assert_eq!(Want::Image.check_type(Some("image/png"), &page), Ok(()));
        assert_eq!(Want::Image.check_type(Some("text/markdown"), &page), Err(FetchError::WrongType));
    }

    #[test]
    fn blocked_ports_are_named_by_the_url() {
        assert!(is_blocked_port(&url("http://x.org:25/")));
        assert!(!is_blocked_port(&url("http://x.org:8080/")));
        assert!(!is_blocked_port(&url("http://x.org/")));
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
            assert_eq!(classify_status(code), Err(FetchError::NotFound), "{code}");
        }
        for code in [100, 199, 300, 304, 400, 401, 403, 418, 429, 500, 503] {
            assert_eq!(classify_status(code), Err(FetchError::Server), "{code}");
        }
    }

    fn hop(from: &str, location: Option<&str>, number: u32) -> Result<String, FetchError> {
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
            assert_eq!(hop(from, Some(location), 1), Err(FetchError::BadRedirect), "{location}");
        }
    }

    #[test]
    fn missing_empty_or_broken_locations_are_refused() {
        for location in [None, Some(""), Some("  "), Some("http://[bad")] {
            assert_eq!(hop("https://a.com/y.md", location, 1), Err(FetchError::BadRedirect), "{location:?}");
        }
    }

    #[test]
    fn redirects_to_ports_browsers_block_are_refused() {
        assert_eq!(hop("https://a.com/x.md", Some("http://b.com:6667/x.md"), 1), Err(FetchError::BadRedirect));
        assert_eq!(hop("https://a.com/x.md", Some("https://b.com:25/x.md"), 1), Err(FetchError::BadRedirect));
        assert_eq!(hop("https://a.com/x.md", Some("https://b.com:8443/x.md"), 1), Ok("https://b.com:8443/x.md".into()));
    }

    #[test]
    fn origins_follow_where_a_reference_was_found() {
        assert_eq!(Origin::of(&Location::web(url("https://x.org/a.md"))), Origin::Public);
        assert_eq!(Origin::of(&Location::web(url("http://127.0.0.1:8000/a.md"))), Origin::Trusted);
        assert_eq!(Origin::of(&Location::web(url("http://router/a.md"))), Origin::Trusted);
        assert_eq!(Origin::of(&Location::file("/x/a.md")), Origin::Trusted);
    }

    #[test]
    fn resolved_addresses_are_checked_for_public_origins_and_redirects_elsewhere() {
        let asked = url("http://wiki.example.com/a.md");
        let public = url("https://x.org/a.md");
        assert!(checks_resolved_address(Origin::Public, 1, &public, &public));
        assert!(!checks_resolved_address(Origin::Trusted, 1, &asked, &asked));
        // The reader's own address may redirect within its host (http to https, a path).
        assert!(!checks_resolved_address(Origin::Trusted, 2, &url("https://wiki.example.com/b.md"), &asked));
        // A redirect to another host is the site's choice, so it is checked.
        assert!(checks_resolved_address(Origin::Trusted, 2, &public, &asked));
        let written_local = url("http://192.168.1.2/a.md");
        assert!(!checks_resolved_address(Origin::Public, 1, &written_local, &written_local));
        assert!(!checks_resolved_address(Origin::Trusted, 3, &written_local, &asked));
    }

    #[test]
    fn page_failures_keep_their_2a_codes() {
        for (error, code) in [
            (FetchError::NotFound, ErrorCode::NotFound),
            (FetchError::Server, ErrorCode::ServerError),
            (FetchError::Unreachable, ErrorCode::Unreachable),
            (FetchError::BadRedirect, ErrorCode::BadRedirect),
            (FetchError::TooLarge, ErrorCode::Unreadable),
            (FetchError::WrongType, ErrorCode::NotMarkdown),
            (FetchError::Undecodable, ErrorCode::Unreadable),
            (FetchError::LocalAddress, ErrorCode::LocalAddress),
        ] {
            assert_eq!(error.for_page(), code, "{error:?}");
        }
    }

    #[test]
    fn each_kind_of_fetch_asks_for_its_own_types() {
        assert_eq!(Want::Page.accept(), ACCEPT);
        assert!(Want::Image.accept().starts_with("image/png, image/jpeg, image/gif, image/webp"));
    }

    #[test]
    fn redirects_stop_after_the_limit() {
        let from = "https://a.com/y.md";
        assert_eq!(hop(from, Some("z.md"), MAX_REDIRECTS), Ok("https://a.com/z.md".into()));
        assert_eq!(hop(from, Some("z.md"), MAX_REDIRECTS + 1), Err(FetchError::BadRedirect));
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
            assert_eq!(hop(from, Some(location), 1), Err(FetchError::BadRedirect), "{location}");
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
