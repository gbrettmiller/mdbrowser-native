//! Fetching a Markdown page over http(s). I/O only: `session::web` decides what
//! to ask for, which redirects to follow, and which responses are Markdown.

use std::io::{self, Read};
use std::time::{Duration, Instant};

use ureq::Agent;
use ureq::Body;
use ureq::http::Response;
use url::Url;

use super::files::MAX_BYTES;
use crate::session::error::ErrorCode;
use crate::session::web::{self, ACCEPT, MAX_REDIRECTS, Reply};

const USER_AGENT: &str = concat!("mdbrowser/", env!("CARGO_PKG_VERSION"));

/// How long and how much one fetch may take, redirects included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub connect: Duration,
    pub overall: Duration,
    pub max_bytes: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { connect: Duration::from_secs(10), overall: Duration::from_secs(30), max_bytes: MAX_BYTES }
    }
}

/// A Markdown page and the URL it was finally found at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    pub url: Url,
    pub source: String,
}

/// One GET per hop, each a fresh request carrying only `Accept`: no cookies, and
/// no credentials beyond those typed for the first host (`next_hop` strips them).
pub fn fetch(url: &Url, limits: &Limits) -> Result<Fetched, ErrorCode> {
    let agent = agent(limits);
    // A deadline too far off to represent means none.
    let deadline = Instant::now().checked_add(limits.overall);
    let mut url = url.clone();
    for hop in 1..=MAX_REDIRECTS + 1 {
        let remaining = deadline.map(|deadline| deadline.saturating_duration_since(Instant::now()));
        let mut response = agent
            .get(url.as_str())
            .header("Accept", ACCEPT)
            .config()
            .timeout_global(remaining)
            .build()
            .call()
            .map_err(code)?;
        match web::classify_status(response.status().as_u16())? {
            Reply::Body => {
                web::body_policy(header(&response, "content-type"), &url)?;
                web::encoding_policy(header(&response, "content-encoding"))?;
                let bytes = read_capped(response.body_mut(), limits.max_bytes)?;
                return Ok(Fetched { source: web::decode(bytes)?, url });
            }
            Reply::Redirect => url = web::next_hop(&url, header(&response, "location"), hop)?,
        }
    }
    Err(ErrorCode::BadRedirect) // unreachable: next_hop refuses the hop past the limit
}

fn agent(limits: &Limits) -> Agent {
    Agent::config_builder()
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_connect(Some(limits.connect))
        .user_agent(USER_AGENT)
        .build()
        .into()
}

/// A header's value; one that isn't valid text counts as absent.
fn header<'a, B>(response: &'a Response<B>, name: &str) -> Option<&'a str> {
    response.headers().get(name).and_then(|value| value.to_str().ok())
}

/// The decoded (decompressed) body, refused past `max_bytes`. ureq's own limit
/// counts the compressed bytes, so a small gzip body could inflate without bound.
fn read_capped(body: &mut Body, max_bytes: u64) -> Result<Vec<u8>, ErrorCode> {
    let mut bytes = Vec::new();
    body.as_reader().take(max_bytes.saturating_add(1)).read_to_end(&mut bytes).map_err(|error| {
        if timed_out(&error) { ErrorCode::Unreachable } else { ErrorCode::Unreadable } // cut off, or bad gzip
    })?;
    if bytes.len() as u64 > max_bytes { Err(ErrorCode::Unreadable) } else { Ok(bytes) }
}

/// ureq wraps its own timeout in an `io::Error` of kind `Other` when reading a body.
fn timed_out(error: &io::Error) -> bool {
    let wrapped = error.get_ref().and_then(|inner| inner.downcast_ref::<ureq::Error>());
    error.kind() == io::ErrorKind::TimedOut || matches!(wrapped, Some(ureq::Error::Timeout(_)))
}

/// Failures before any response: DNS, refused or reset connections, TLS, timeouts.
fn code(_error: ureq::Error) -> ErrorCode {
    ErrorCode::Unreachable
}

#[cfg(test)]
#[path = "../../tests/support/http.rs"]
mod test_server;

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    use flate2::Compression;
    use flate2::write::GzEncoder;

    use super::test_server::{Reply as Canned, Server};
    use super::*;

    const LIMIT: u64 = 1024;

    fn limits() -> Limits {
        Limits { connect: Duration::from_secs(2), overall: Duration::from_secs(5), max_bytes: LIMIT }
    }

    fn get(url: &str) -> Result<Fetched, ErrorCode> {
        fetch(&Url::parse(url).unwrap(), &limits())
    }

    fn source(url: &str) -> Result<String, ErrorCode> {
        get(url).map(|fetched| fetched.source)
    }

    fn serve(reply: Canned) -> Server {
        Server::start(move |_| reply.clone())
    }

    fn gzip(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn asks_for_markdown_and_returns_the_text_and_its_url() {
        let server = serve(Canned::markdown("# Hi"));
        let fetched = get(&server.url("/a.md")).unwrap();
        assert_eq!(fetched.source, "# Hi");
        assert_eq!(fetched.url.as_str(), server.url("/a.md"));
        let headers = &server.requests()[0].headers;
        assert_eq!(headers["accept"], ACCEPT);
        assert!(headers["user-agent"].starts_with("mdbrowser/"), "{headers:?}");
    }

    #[test]
    fn follows_redirects_and_reports_where_it_ended() {
        let server = Server::start(|request| match request.path.as_str() {
            "/old.md" => Canned::redirect(301, "/new.md"),
            _ => Canned::markdown("new"),
        });
        let fetched = get(&server.url("/old.md")).unwrap();
        assert_eq!((fetched.source.as_str(), fetched.url.as_str()), ("new", server.url("/new.md").as_str()));
    }

    #[test]
    fn a_redirect_loop_stops_one_request_past_the_limit() {
        let server = serve(Canned::redirect(302, "/loop.md"));
        assert_eq!(source(&server.url("/loop.md")), Err(ErrorCode::BadRedirect));
        assert_eq!(server.requests().len() as u32, MAX_REDIRECTS + 1);
    }

    #[test]
    fn the_overall_limit_covers_every_hop() {
        let server = Server::start(|request| {
            std::thread::sleep(Duration::from_millis(150));
            Canned::redirect(302, &format!("{}x", request.path))
        });
        let quick = Limits { overall: Duration::from_millis(400), ..limits() };
        let started = Instant::now();
        let result = fetch(&Url::parse(&server.url("/a.md")).unwrap(), &quick).map(|f| f.source);
        assert_eq!(result, Err(ErrorCode::Unreachable));
        assert!(server.requests().len() < 4, "stopped by the deadline, not the hop limit");
        assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    }

    #[test]
    fn redirects_without_a_readable_location_are_refused() {
        assert_eq!(source(&serve(Canned::new(302)).url("/a.md")), Err(ErrorCode::BadRedirect));
        let garbled = serve(Canned::new(302).raw_header("Location", b"/\xff\xfe.md"));
        assert_eq!(source(&garbled.url("/a.md")), Err(ErrorCode::BadRedirect));
    }

    #[test]
    fn failing_statuses() {
        for (status, code) in [(404, ErrorCode::NotFound), (410, ErrorCode::NotFound), (500, ErrorCode::ServerError)] {
            assert_eq!(source(&serve(Canned::new(status)).url("/a.md")), Err(code), "{status}");
        }
    }

    #[test]
    fn bodies_over_the_limit_are_unreadable() {
        let at_limit = "x".repeat(LIMIT as usize);
        assert_eq!(source(&serve(Canned::markdown(&at_limit)).url("/a.md")), Ok(at_limit.clone()));
        let over = at_limit + "x";
        assert_eq!(source(&serve(Canned::markdown(&over)).url("/a.md")), Err(ErrorCode::Unreadable));
    }

    #[test]
    fn the_limit_applies_after_decompression() {
        let small = Canned::markdown("").header("Content-Encoding", "gzip").body(gzip(b"# zipped"));
        assert_eq!(source(&serve(small).url("/a.md")), Ok("# zipped".into()));
        let bomb = Canned::markdown("").header("Content-Encoding", "gzip").body(gzip(&vec![b'x'; 64 * 1024]));
        assert_eq!(source(&serve(bomb).url("/a.md")), Err(ErrorCode::Unreadable));
        let corrupt = Canned::markdown("").header("Content-Encoding", "gzip").body("not gzip at all");
        assert_eq!(source(&serve(corrupt).url("/a.md")), Err(ErrorCode::Unreadable));
    }

    #[test]
    fn only_strict_utf8_is_readable() {
        let invalid = Canned::markdown("").body(b"caf\xe9".to_vec());
        assert_eq!(source(&serve(invalid).url("/a.md")), Err(ErrorCode::Unreadable));
    }

    #[test]
    fn a_declared_charset_other_than_utf8_is_unreadable() {
        let latin1 =
            Canned::new(200).header("Content-Type", "text/markdown; charset=iso-8859-1").body(b"caf\xe9".to_vec());
        assert_eq!(source(&serve(latin1).url("/a.md")), Err(ErrorCode::Unreadable));
    }

    #[test]
    fn a_leading_byte_order_mark_is_dropped() {
        assert_eq!(source(&serve(Canned::markdown("\u{feff}# Hi")).url("/a.md")), Ok("# Hi".into()));
    }

    #[test]
    fn the_response_type_and_path_decide_what_is_markdown() {
        let html = Canned::new(200).header("Content-Type", "text/html").body("<p>hi</p>");
        assert_eq!(source(&serve(html).url("/a.md")), Err(ErrorCode::NotMarkdown));
        let untyped = serve(Canned::new(200).body("# Hi"));
        assert_eq!(source(&untyped.url("/a.md")), Ok("# Hi".into()));
        assert_eq!(source(&untyped.url("/a")), Err(ErrorCode::NotMarkdown));
        assert_eq!(source(&serve(Canned::new(204)).url("/empty.md")), Ok(String::new()));
    }

    #[test]
    fn a_silent_server_times_out_as_unreachable() {
        let silent = TcpListener::bind("127.0.0.1:0").unwrap(); // connects, never answers
        let url = Url::parse(&format!("http://{}/a.md", silent.local_addr().unwrap())).unwrap();
        let quick = Limits { overall: Duration::from_millis(300), ..limits() };
        let started = Instant::now();
        assert_eq!(fetch(&url, &quick).map(|f| f.source), Err(ErrorCode::Unreachable));
        let waited = started.elapsed();
        assert!(waited >= Duration::from_millis(250) && waited < Duration::from_secs(3), "{waited:?}");
    }

    #[test]
    fn a_body_that_stalls_past_the_deadline_is_unreachable() {
        let stalled = serve(Canned::markdown("# Hi").cut_short(100, Duration::from_secs(5)));
        let quick = Limits { overall: Duration::from_millis(300), ..limits() };
        let started = Instant::now();
        let result = fetch(&Url::parse(&stalled.url("/a.md")).unwrap(), &quick).map(|f| f.source);
        assert_eq!(result, Err(ErrorCode::Unreachable));
        assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    }

    #[test]
    fn a_body_cut_off_by_the_server_is_unreadable() {
        let cut = serve(Canned::markdown("# Hi").cut_short(100, Duration::ZERO));
        assert_eq!(source(&cut.url("/a.md")), Err(ErrorCode::Unreadable));
    }

    #[test]
    fn encodings_ureq_does_not_decode_are_unreadable() {
        for encoding in ["br", "deflate", "x-gzip", "GZIP"] {
            let encoded = Canned::markdown("# Hi").header("Content-Encoding", encoding);
            assert_eq!(source(&serve(encoded).url("/a.md")), Err(ErrorCode::Unreadable), "{encoding}");
        }
        let identity = Canned::markdown("# Hi").header("Content-Encoding", "identity");
        assert_eq!(source(&serve(identity).url("/a.md")), Ok("# Hi".into()));
    }

    #[test]
    fn a_refused_connection_is_unreachable() {
        // Dropped at once. A parallel test could in theory be given the same port
        // in between; the window is a few microseconds.
        let addr = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
        assert_eq!(source(&format!("http://{addr}/a.md")), Err(ErrorCode::Unreachable));
    }

    #[test]
    /// ureq's `cookies` feature is off; this guards against it being turned on.
    fn cookies_are_never_sent_back() {
        let server = Server::start(|request| match request.path.as_str() {
            "/a.md" => Canned::redirect(302, "/b.md").header("Set-Cookie", "session=1; Path=/"),
            _ => Canned::markdown("b"),
        });
        assert_eq!(source(&server.url("/a.md")), Ok("b".into()));
        assert!(server.requests().iter().all(|request| !request.headers.contains_key("cookie")));
    }

    #[test]
    fn typed_credentials_never_follow_a_redirect() {
        let target = serve(Canned::markdown("b"));
        let target_url = target.url("/b.md");
        let origin = Server::start(move |_| Canned::redirect(302, &target_url));
        let typed = origin.url("/a.md").replace("http://", "http://user:pw@");
        assert_eq!(source(&typed), Ok("b".into()));
        assert!(origin.requests()[0].headers.contains_key("authorization"), "the typed host gets them");
        assert!(!target.requests()[0].headers.contains_key("authorization"));
    }
}
