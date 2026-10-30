//! The fetch loop shared by pages and images.

use std::io::{self, Read};
use std::sync::Arc;
use std::time::Instant;

use ureq::config::Config;
use ureq::http::{Response, Uri};
use ureq::unversioned::resolver::{DefaultResolver, ResolvedSocketAddrs, Resolver};
use ureq::unversioned::transport::{DefaultConnector, NextTimeout};
use ureq::{Agent, Body, Proxy};
use url::Url;

use super::Limits;
use crate::session::network::is_local_ip;
use crate::session::web::{self, FetchError, MAX_REDIRECTS, Origin, StatusClass, Want};

const USER_AGENT: &str = concat!("mdbrowser/", env!("CARGO_PKG_VERSION"));

/// One GET per hop, each a fresh request carrying only `Accept`: no cookies, and
/// no credentials beyond those typed for the first host (`next_hop` strips them).
/// Returns the final URL and the body, checked against `want`'s types.
pub fn fetch_bytes(url: &Url, want: Want, limits: &Limits, origin: Origin) -> Result<(Url, Vec<u8>), FetchError> {
    fetch_via(url, want, limits, origin, Arc::new(DefaultResolver::default()))
}

/// A refused response, with what a fallback needs from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    pub error: FetchError,
    /// Where the refusal came from, after redirects, when a response was read.
    pub url: Option<Url>,
    /// Its `Link` header, kept only for a wrong type.
    pub link: Option<String>,
}

impl From<FetchError> for Refused {
    fn from(error: FetchError) -> Self {
        Refused { error, url: None, link: None }
    }
}

/// `fetch_bytes` for a page, keeping the `Link` header of a response of the wrong type.
pub fn fetch_page_bytes(url: &Url, limits: &Limits, origin: Origin) -> Result<(Url, Vec<u8>), Refused> {
    fetch_response(url, Want::Page, limits, origin, Arc::new(DefaultResolver::default()))
}

/// `fetch_response`, with only the reason for a refusal.
fn fetch_via(
    url: &Url,
    want: Want,
    limits: &Limits,
    origin: Origin,
    resolver: Arc<dyn Resolver>,
) -> Result<(Url, Vec<u8>), FetchError> {
    fetch_response(url, want, limits, origin, resolver).map_err(|refused| refused.error)
}

/// The fetch loop, resolving names with `resolver` (tests swap in their own).
fn fetch_response(
    url: &Url,
    want: Want,
    limits: &Limits,
    origin: Origin,
    resolver: Arc<dyn Resolver>,
) -> Result<(Url, Vec<u8>), Refused> {
    // A deadline too far off to represent means none.
    let deadline = Instant::now().checked_add(limits.overall);
    let asked = url.clone();
    let mut url = url.clone();
    for hop in 1..=MAX_REDIRECTS + 1 {
        let check = web::checks_resolved_address(origin, hop, &url, &asked);
        let config = config(limits, check, Proxy::try_from_env());
        let agent = Agent::with_parts(config, DefaultConnector::new(), Guarded { inner: Arc::clone(&resolver), check });
        let remaining = deadline.map(|deadline| deadline.saturating_duration_since(Instant::now()));
        let mut response = agent
            .get(url.as_str())
            .header("Accept", want.accept())
            .config()
            .timeout_global(remaining)
            .build()
            .call()
            .map_err(request_failed)?;
        match web::classify_status(response.status().as_u16())? {
            StatusClass::Body => {
                let content_type = header(&response, "content-type");
                want.check_type(content_type, &url).map_err(|error| Refused {
                    error,
                    url: Some(url.clone()),
                    link: (error == FetchError::WrongType)
                        .then(|| header(&response, "link").map(String::from))
                        .flatten(),
                })?;
                web::encoding_policy(header(&response, "content-encoding"))?;
                let bytes = read_capped(response.body_mut(), limits.max_bytes)?;
                return Ok((url, bytes));
            }
            StatusClass::Redirect => url = web::next_hop(&url, header(&response, "location"), hop)?,
        }
    }
    Err(FetchError::BadRedirect.into()) // unreachable: next_hop refuses the hop past the limit
}

/// One hop's settings; each hop gets a fresh agent, so no connection, cookie or
/// credential outlives one request. A checked hop goes direct: a proxy would
/// resolve the name itself, out of the check's sight (ADR-004).
fn config(limits: &Limits, check: bool, env_proxy: Option<Proxy>) -> Config {
    Agent::config_builder()
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_connect(Some(limits.connect))
        .user_agent(USER_AGENT)
        .proxy(if check { None } else { env_proxy })
        .build()
}

/// A name resolved to a local address where that isn't allowed.
#[derive(Debug)]
struct LocalAddress;

impl std::fmt::Display for LocalAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("a public name resolved to a local address")
    }
}

impl std::error::Error for LocalAddress {}

/// Resolves names, and when `check` is set refuses any that lead to a local address.
/// ureq connects only to the addresses returned here, so what was checked is what
/// is used: the name is never resolved twice.
#[derive(Debug)]
struct Guarded {
    inner: Arc<dyn Resolver>,
    check: bool,
}

impl Resolver for Guarded {
    fn resolve(&self, uri: &Uri, config: &Config, timeout: NextTimeout) -> Result<ResolvedSocketAddrs, ureq::Error> {
        let addresses = self.inner.resolve(uri, config, timeout)?;
        if self.check && addresses.iter().any(|address| is_local_ip(address.ip())) {
            return Err(ureq::Error::Other(Box::new(LocalAddress)));
        }
        Ok(addresses)
    }
}

/// A header's value; one that isn't valid text counts as absent.
fn header<'a, B>(response: &'a Response<B>, name: &str) -> Option<&'a str> {
    response.headers().get(name).and_then(|value| value.to_str().ok())
}

/// The decoded (decompressed) body, refused past `max_bytes`. ureq's own limit
/// counts the compressed bytes, so a small gzip body could inflate without bound.
fn read_capped(body: &mut Body, max_bytes: u64) -> Result<Vec<u8>, FetchError> {
    let mut bytes = Vec::new();
    body.as_reader().take(max_bytes.saturating_add(1)).read_to_end(&mut bytes).map_err(|error| {
        if timed_out(&error) { FetchError::Unreachable } else { FetchError::Undecodable } // cut off, or bad gzip
    })?;
    if bytes.len() as u64 > max_bytes { Err(FetchError::TooLarge) } else { Ok(bytes) }
}

/// ureq wraps its own timeout in an `io::Error` of kind `Other` when reading a body.
fn timed_out(error: &io::Error) -> bool {
    let wrapped = error.get_ref().and_then(|inner| inner.downcast_ref::<ureq::Error>());
    error.kind() == io::ErrorKind::TimedOut || matches!(wrapped, Some(ureq::Error::Timeout(_)))
}

/// Failures before any response: DNS, refused or reset connections, TLS, timeouts,
/// and the resolver's refusal.
fn request_failed(error: ureq::Error) -> FetchError {
    match error {
        ureq::Error::Other(inner) if inner.is::<LocalAddress>() => FetchError::LocalAddress,
        _ => FetchError::Unreachable,
    }
}

#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::time::Duration;

    use super::*;
    use crate::services::test_server::{Reply, Server};

    /// Resolves every name to the test server, as rebinding DNS would.
    #[derive(Debug)]
    struct Everywhere(SocketAddr);

    impl Resolver for Everywhere {
        fn resolve(&self, _: &Uri, _: &Config, _: NextTimeout) -> Result<ResolvedSocketAddrs, ureq::Error> {
            let mut addresses = self.empty();
            addresses.push(self.0);
            Ok(addresses)
        }
    }

    fn limits() -> Limits {
        Limits { connect: Duration::from_secs(2), overall: Duration::from_secs(5), max_bytes: 1024 }
    }

    /// `http://<name>:<port><path>`, which `Everywhere` sends to `server`.
    fn named(server: &Server, name: &str, path: &str) -> Url {
        let port = Url::parse(&server.url("/")).unwrap().port().unwrap();
        Url::parse(&format!("http://{name}:{port}{path}")).unwrap()
    }

    fn fetch_at(server: &Server, url: &Url, want: Want, origin: Origin) -> Result<(Url, Vec<u8>), FetchError> {
        let address = server.url("/").trim_start_matches("http://").trim_end_matches('/').parse().unwrap();
        fetch_via(url, want, &limits(), origin, Arc::new(Everywhere(address)))
    }

    #[test]
    fn a_public_origin_never_connects_to_a_name_that_resolves_locally() {
        let server = Server::start(|_| Reply::markdown("# secret"));
        let url = named(&server, "public.test", "/a.md");
        assert_eq!(fetch_at(&server, &url, Want::Page, Origin::Public), Err(FetchError::LocalAddress));
        assert!(server.requests().is_empty(), "nothing was sent");
    }

    #[test]
    fn the_readers_own_request_may_reach_an_intranet_name() {
        let server = Server::start(|_| Reply::markdown("# wiki"));
        let url = named(&server, "wiki.example.com", "/a.md");
        assert_eq!(fetch_at(&server, &url, Want::Page, Origin::Trusted).map(|(_, body)| body), Ok(b"# wiki".to_vec()));
    }

    #[test]
    fn a_redirect_never_lands_on_a_name_that_resolves_locally() {
        let server = Server::start(|request| match request.path.as_str() {
            "/a.md" => Reply::redirect(302, "http://other.test/b.md"),
            _ => Reply::markdown("# b"),
        });
        let url = named(&server, "wiki.example.com", "/a.md");
        assert_eq!(fetch_at(&server, &url, Want::Page, Origin::Trusted), Err(FetchError::LocalAddress));
        assert_eq!(server.requests().len(), 1, "only the first hop was sent");
    }

    #[test]
    fn the_readers_own_address_may_redirect_within_its_host() {
        let server = Server::start(|request| match request.path.as_str() {
            "/a.md" => Reply::redirect(301, "/b.md"),
            _ => Reply::markdown("# b"),
        });
        let url = named(&server, "wiki.example.com", "/a.md");
        assert_eq!(fetch_at(&server, &url, Want::Page, Origin::Trusted).map(|(_, body)| body), Ok(b"# b".to_vec()));
    }

    #[test]
    fn checked_hops_go_direct_and_others_may_use_a_proxy() {
        let proxy = Proxy::new("http://127.0.0.1:3128").ok();
        assert!(config(&limits(), true, proxy.clone()).proxy().is_none());
        assert!(config(&limits(), false, proxy).proxy().is_some());
    }

    #[test]
    fn written_local_addresses_are_left_to_the_link_rules() {
        let server = Server::start(|_| Reply::markdown("# here"));
        let url = Url::parse(&server.url("/a.md")).unwrap();
        assert!(fetch_at(&server, &url, Want::Page, Origin::Public).is_ok());
    }

    #[test]
    fn images_ask_for_images_and_take_only_image_types() {
        let png = Server::start(|_| Reply::new(200).header("Content-Type", "image/png").body(vec![1, 2, 3]));
        let url = Url::parse(&png.url("/a.png")).unwrap();
        assert_eq!(fetch_at(&png, &url, Want::Image, Origin::Trusted).map(|(_, body)| body), Ok(vec![1, 2, 3]));
        let headers = &png.requests()[0].headers;
        assert_eq!(headers["accept"], web::IMAGE_ACCEPT);
        assert!(!headers.contains_key("referer") && !headers.contains_key("cookie"), "{headers:?}");
        let html = Server::start(|_| Reply::new(200).header("Content-Type", "text/html").body("<p>"));
        let url = Url::parse(&html.url("/a.png")).unwrap();
        assert_eq!(fetch_at(&html, &url, Want::Image, Origin::Trusted), Err(FetchError::WrongType));
    }

    /// Resolves to the test server and counts the lookups.
    #[derive(Debug)]
    struct Counting(SocketAddr, std::sync::atomic::AtomicUsize);

    impl Resolver for Counting {
        fn resolve(&self, _: &Uri, _: &Config, _: NextTimeout) -> Result<ResolvedSocketAddrs, ureq::Error> {
            self.1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let mut addresses = self.empty();
            addresses.push(self.0);
            Ok(addresses)
        }
    }

    #[test]
    fn a_hop_resolves_once_and_connects_to_the_address_it_got() {
        let server = Server::start(|_| Reply::markdown("# wiki"));
        let address = server.url("/").trim_start_matches("http://").trim_end_matches('/').parse().unwrap();
        let resolver = Arc::new(Counting(address, Default::default()));
        let url = named(&server, "wiki.example.com", "/a.md");
        let result = fetch_via(&url, Want::Page, &limits(), Origin::Trusted, resolver.clone());
        assert_eq!(result.map(|(_, body)| body), Ok(b"# wiki".to_vec()), "it reached the resolved address");
        assert_eq!(resolver.1.load(std::sync::atomic::Ordering::SeqCst), 1, "one lookup");
    }

    #[test]
    fn a_redirect_to_a_blocked_port_is_refused_before_it_is_followed() {
        let server = Server::start(|_| Reply::redirect(302, "http://127.0.0.1:6667/x.md"));
        let url = Url::parse(&server.url("/a.md")).unwrap();
        assert_eq!(fetch_at(&server, &url, Want::Page, Origin::Trusted), Err(FetchError::BadRedirect));
        assert_eq!(server.requests().len(), 1);
    }

    #[test]
    fn a_body_over_the_cap_is_too_large() {
        let server = Server::start(|_| Reply::new(200).header("Content-Type", "image/png").body(vec![0; 1025]));
        let url = Url::parse(&server.url("/a.png")).unwrap();
        assert_eq!(fetch_at(&server, &url, Want::Image, Origin::Trusted), Err(FetchError::TooLarge));
    }
}
