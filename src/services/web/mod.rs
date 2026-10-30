//! Fetching over http(s). I/O only: `session::web` decides what to ask for, which
//! redirects to follow, and which responses to take. Pages and images share one
//! fetch loop (`fetch::fetch_bytes`).

mod fetch;

use std::time::Duration;

pub use fetch::fetch_bytes;
use fetch::{Refused, fetch_page_bytes};
use url::Url;

use super::files::MAX_BYTES;
use crate::session::error::ErrorCode;
use crate::session::web::{self, Origin};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// For the whole fetch, every redirect included.
const OVERALL_TIMEOUT: Duration = Duration::from_secs(30);

/// How long and how much one fetch may take, redirects included.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    pub connect: Duration,
    pub overall: Duration,
    pub max_bytes: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits { connect: CONNECT_TIMEOUT, overall: OVERALL_TIMEOUT, max_bytes: MAX_BYTES }
    }
}

/// A Markdown page and the URL it was finally found at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    pub url: Url,
    pub source: String,
    /// The address asked for, when this was found in its place (ADR-005).
    pub fallback_from: Option<Url>,
}

/// A Markdown page, with the failure codes pages have always had (phase 2a). When
/// the server answers with something else, Markdown versions of the same page are
/// tried (ADR-005); the result says which address was found.
pub fn fetch(url: &Url, limits: &Limits, origin: Origin) -> Result<Fetched, ErrorCode> {
    fallback(url, limits, origin)
}

fn page(url: &Url, limits: &Limits, origin: Origin) -> Result<Fetched, Refused> {
    let (url, bytes) = fetch_page_bytes(url, limits, origin)?;
    let source = web::to_text(bytes).map_err(Refused::from)?;
    Ok(Fetched { url, source, fallback_from: None })
}

/// `url`, or when it isn't Markdown, the first of its `Link` alternate and `.md`
/// siblings that is. One level only: a candidate's own refusal is never followed,
/// and any other failure (404, timeout, bad body) never starts a search.
fn fallback(url: &Url, limits: &Limits, origin: Origin) -> Result<Fetched, ErrorCode> {
    let refused = match page(url, limits, origin) {
        Ok(found) => return Ok(found),
        Err(refused) => refused,
    };
    if refused.error != web::FetchError::WrongType {
        return Err(refused.error.for_page());
    }
    let asked = refused.url.as_ref().unwrap_or(url);
    web::candidates(asked, refused.link.as_deref())
        .iter()
        .find_map(|candidate| page(candidate, limits, origin).ok())
        .map(|found| Fetched { fallback_from: Some(url.clone()), ..found })
        .ok_or_else(|| {
            if has_llms_txt(asked, limits, origin) { ErrorCode::NotMarkdownWithLlms } else { ErrorCode::NotMarkdown }
        })
}

/// Whether the site answers for `/llms.txt` the way a click on it would be read, so
/// the offer is never made for a page that would only fail.
fn has_llms_txt(asked: &Url, limits: &Limits, origin: Origin) -> bool {
    asked.join(web::LLMS_TXT).is_ok_and(|llms| page(&llms, limits, origin).is_ok())
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::net::TcpListener;
    use std::time::{Duration, Instant};

    use flate2::Compression;
    use flate2::write::GzEncoder;

    use super::*;
    use crate::services::test_server::{Reply, Server};
    use crate::session::web::{ACCEPT, MAX_REDIRECTS};

    const LIMIT: u64 = 1024;

    fn limits() -> Limits {
        Limits { connect: Duration::from_secs(2), overall: Duration::from_secs(5), max_bytes: LIMIT }
    }

    fn get(url: &str) -> Result<Fetched, ErrorCode> {
        fetch(&Url::parse(url).unwrap(), &limits(), Origin::Trusted)
    }

    fn source(url: &str) -> Result<String, ErrorCode> {
        get(url).map(|fetched| fetched.source)
    }

    fn serve(reply: Reply) -> Server {
        Server::start(move |_| reply.clone())
    }

    fn gzip(bytes: &[u8]) -> Vec<u8> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(bytes).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn asks_for_markdown_and_returns_the_text_and_its_url() {
        let server = serve(Reply::markdown("# Hi"));
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
            "/old.md" => Reply::redirect(301, "/new.md"),
            _ => Reply::markdown("new"),
        });
        let fetched = get(&server.url("/old.md")).unwrap();
        assert_eq!((fetched.source.as_str(), fetched.url.as_str()), ("new", server.url("/new.md").as_str()));
    }

    #[test]
    fn a_redirect_loop_stops_one_request_past_the_limit() {
        let server = serve(Reply::redirect(302, "/loop.md"));
        assert_eq!(source(&server.url("/loop.md")), Err(ErrorCode::BadRedirect));
        assert_eq!(server.requests().len() as u32, MAX_REDIRECTS + 1);
    }

    #[test]
    fn the_overall_limit_covers_every_hop() {
        let server = Server::start(|request| {
            std::thread::sleep(Duration::from_millis(150));
            Reply::redirect(302, &format!("{}x", request.path))
        });
        let quick = Limits { overall: Duration::from_millis(400), ..limits() };
        let started = Instant::now();
        let result = fetch(&Url::parse(&server.url("/a.md")).unwrap(), &quick, Origin::Trusted).map(|f| f.source);
        assert_eq!(result, Err(ErrorCode::Unreachable));
        assert!(server.requests().len() < 4, "stopped by the deadline, not the hop limit");
        assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    }

    #[test]
    fn redirects_without_a_readable_location_are_refused() {
        assert_eq!(source(&serve(Reply::new(302)).url("/a.md")), Err(ErrorCode::BadRedirect));
        let garbled = serve(Reply::new(302).raw_header("Location", b"/\xff\xfe.md"));
        assert_eq!(source(&garbled.url("/a.md")), Err(ErrorCode::BadRedirect));
    }

    #[test]
    fn failing_statuses() {
        for (status, code) in [(404, ErrorCode::NotFound), (410, ErrorCode::NotFound), (500, ErrorCode::ServerError)] {
            assert_eq!(source(&serve(Reply::new(status)).url("/a.md")), Err(code), "{status}");
        }
    }

    #[test]
    fn bodies_over_the_limit_are_unreadable() {
        let at_limit = "x".repeat(LIMIT as usize);
        assert_eq!(source(&serve(Reply::markdown(&at_limit)).url("/a.md")), Ok(at_limit.clone()));
        let over = at_limit + "x";
        assert_eq!(source(&serve(Reply::markdown(&over)).url("/a.md")), Err(ErrorCode::Unreadable));
    }

    #[test]
    fn the_limit_applies_after_decompression() {
        let small = Reply::markdown("").header("Content-Encoding", "gzip").body(gzip(b"# zipped"));
        assert_eq!(source(&serve(small).url("/a.md")), Ok("# zipped".into()));
        let bomb = Reply::markdown("").header("Content-Encoding", "gzip").body(gzip(&vec![b'x'; 64 * 1024]));
        assert_eq!(source(&serve(bomb).url("/a.md")), Err(ErrorCode::Unreadable));
        let corrupt = Reply::markdown("").header("Content-Encoding", "gzip").body("not gzip at all");
        assert_eq!(source(&serve(corrupt).url("/a.md")), Err(ErrorCode::Unreadable));
    }

    #[test]
    fn only_strict_utf8_is_readable() {
        let invalid = Reply::markdown("").body(b"caf\xe9".to_vec());
        assert_eq!(source(&serve(invalid).url("/a.md")), Err(ErrorCode::Unreadable));
    }

    #[test]
    fn a_declared_charset_other_than_utf8_is_unreadable() {
        let latin1 =
            Reply::new(200).header("Content-Type", "text/markdown; charset=iso-8859-1").body(b"caf\xe9".to_vec());
        assert_eq!(source(&serve(latin1).url("/a.md")), Err(ErrorCode::Unreadable));
    }

    #[test]
    fn a_leading_byte_order_mark_is_dropped() {
        assert_eq!(source(&serve(Reply::markdown("\u{feff}# Hi")).url("/a.md")), Ok("# Hi".into()));
    }

    #[test]
    fn the_response_type_and_path_decide_what_is_markdown() {
        let html = Reply::new(200).header("Content-Type", "text/html").body("<p>hi</p>");
        assert_eq!(source(&serve(html).url("/a.md")), Err(ErrorCode::NotMarkdown));
        let untyped =
            Server::start(
                |request| if request.path == "/a.md" { Reply::new(200).body("# Hi") } else { Reply::new(404) },
            );
        assert_eq!(source(&untyped.url("/a.md")), Ok("# Hi".into()));
        let only_a =
            Server::start(|request| if request.path == "/a" { Reply::new(200).body("# Hi") } else { Reply::new(404) });
        assert_eq!(source(&only_a.url("/a")), Err(ErrorCode::NotMarkdown));
        assert_eq!(source(&serve(Reply::new(204)).url("/empty.md")), Ok(String::new()));
    }

    fn html() -> Reply {
        Reply::new(200).header("Content-Type", "text/html").body("<p>hi</p>")
    }

    /// `/post` is HTML; `/post.md` is Markdown.
    fn site(page: Reply) -> Server {
        Server::start(move |request| match request.path.as_str() {
            "/post.md" => Reply::markdown("# Found"),
            "/raw/post.md" => Reply::markdown("# Alternate"),
            "/post" => page.clone(),
            _ => Reply::new(404),
        })
    }

    #[test]
    fn an_html_page_falls_back_to_its_md_sibling() {
        let server = site(html());
        let found = fallback(&Url::parse(&server.url("/post")).unwrap(), &limits(), Origin::Trusted).unwrap();
        assert_eq!((found.url.path(), found.source.as_str()), ("/post.md", "# Found"));
    }

    #[test]
    fn a_link_alternate_wins_over_the_sibling() {
        let linked = html().header("Link", r#"</raw/post.md>; rel="alternate"; type="text/markdown""#);
        let found = fallback(&Url::parse(&site(linked).url("/post")).unwrap(), &limits(), Origin::Trusted).unwrap();
        assert_eq!((found.url.path(), found.source.as_str()), ("/raw/post.md", "# Alternate"));
    }

    #[test]
    fn a_link_to_another_host_is_never_followed() {
        let elsewhere = html().header("Link", r#"<https://evil.example/p.md>; rel="alternate"; type="text/markdown""#);
        let found = fallback(&Url::parse(&site(elsewhere).url("/post")).unwrap(), &limits(), Origin::Trusted).unwrap();
        assert_eq!(found.url.path(), "/post.md");
    }

    #[test]
    fn nothing_found_keeps_the_original_error() {
        let server = Server::start(|request| if request.path == "/post" { html() } else { Reply::new(404) });
        let result = fallback(&Url::parse(&server.url("/post")).unwrap(), &limits(), Origin::Trusted);
        assert_eq!(result, Err(ErrorCode::NotMarkdown));
    }

    fn with_llms(llms: Reply) -> Server {
        Server::start(move |request| match request.path.as_str() {
            "/post" => html(),
            "/llms.txt" => llms.clone(),
            _ => Reply::new(404),
        })
    }

    #[test]
    fn a_site_with_an_llms_txt_says_so() {
        let plain = Reply::new(200).header("Content-Type", "text/plain; charset=utf-8").body("# Site");
        for llms in [Reply::markdown("# Site"), plain] {
            let result = fallback(&Url::parse(&with_llms(llms).url("/post")).unwrap(), &limits(), Origin::Trusted);
            assert_eq!(result, Err(ErrorCode::NotMarkdownWithLlms));
        }
    }

    #[test]
    fn a_missing_or_unreadable_llms_txt_is_not_offered() {
        for llms in [Reply::new(404), Reply::new(500), html()] {
            let result = fallback(&Url::parse(&with_llms(llms).url("/post")).unwrap(), &limits(), Origin::Trusted);
            assert_eq!(result, Err(ErrorCode::NotMarkdown));
        }
    }

    #[test]
    fn llms_txt_is_only_looked_for_when_nothing_else_was_found() {
        let server = site(html());
        fallback(&Url::parse(&server.url("/post")).unwrap(), &limits(), Origin::Trusted).unwrap();
        assert!(server.requests().iter().all(|request| request.path != "/llms.txt"));
    }

    #[test]
    fn a_fallback_that_is_html_too_does_not_chain() {
        let server = Server::start(|request| match request.path.as_str() {
            "/post" => html(),
            "/post.md" => html().header("Link", r#"</x.md>; rel="alternate"; type="text/markdown""#),
            "/x.md" => Reply::markdown("# Chained"),
            _ => Reply::new(404),
        });
        let result = fallback(&Url::parse(&server.url("/post")).unwrap(), &limits(), Origin::Trusted);
        assert_eq!(result, Err(ErrorCode::NotMarkdown));
        assert!(server.requests().iter().all(|request| request.path != "/x.md"));
    }

    #[test]
    fn only_a_wrong_type_starts_a_fallback() {
        for status in [404, 500] {
            let server = Server::start(move |request| match request.path.as_str() {
                "/post.md" => Reply::markdown("# Found"),
                _ => Reply::new(status),
            });
            let result = fallback(&Url::parse(&server.url("/post")).unwrap(), &limits(), Origin::Trusted);
            assert!(result.is_err(), "{status}");
            assert!(server.requests().iter().all(|request| request.path != "/post.md"), "{status}");
        }
    }

    #[test]
    fn markdown_is_returned_as_it_is() {
        let server = Server::start(|_| Reply::markdown("# Here"));
        let found = fallback(&Url::parse(&server.url("/post")).unwrap(), &limits(), Origin::Trusted).unwrap();
        assert_eq!((found.url.path(), found.source.as_str()), ("/post", "# Here"));
        assert_eq!(server.requests().len(), 1);
    }

    #[test]
    fn a_silent_server_times_out_as_unreachable() {
        let silent = TcpListener::bind("127.0.0.1:0").unwrap(); // connects, never answers
        let url = Url::parse(&format!("http://{}/a.md", silent.local_addr().unwrap())).unwrap();
        let quick = Limits { overall: Duration::from_millis(300), ..limits() };
        let started = Instant::now();
        assert_eq!(fetch(&url, &quick, Origin::Trusted).map(|f| f.source), Err(ErrorCode::Unreachable));
        let waited = started.elapsed();
        assert!(waited >= Duration::from_millis(250) && waited < Duration::from_secs(3), "{waited:?}");
    }

    #[test]
    fn a_body_that_stalls_past_the_deadline_is_unreachable() {
        let stalled = serve(Reply::markdown("# Hi").cut_short(100, Duration::from_secs(5)));
        let quick = Limits { overall: Duration::from_millis(300), ..limits() };
        let started = Instant::now();
        let result = fetch(&Url::parse(&stalled.url("/a.md")).unwrap(), &quick, Origin::Trusted).map(|f| f.source);
        assert_eq!(result, Err(ErrorCode::Unreachable));
        assert!(started.elapsed() < Duration::from_secs(3), "{:?}", started.elapsed());
    }

    #[test]
    fn a_body_cut_off_by_the_server_is_unreadable() {
        let cut = serve(Reply::markdown("# Hi").cut_short(100, Duration::ZERO));
        assert_eq!(source(&cut.url("/a.md")), Err(ErrorCode::Unreadable));
    }

    #[test]
    fn encodings_ureq_does_not_decode_are_unreadable() {
        for encoding in ["br", "deflate", "x-gzip", "GZIP"] {
            let encoded = Reply::markdown("# Hi").header("Content-Encoding", encoding);
            assert_eq!(source(&serve(encoded).url("/a.md")), Err(ErrorCode::Unreadable), "{encoding}");
        }
        let identity = Reply::markdown("# Hi").header("Content-Encoding", "identity");
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
            "/a.md" => Reply::redirect(302, "/b.md").header("Set-Cookie", "session=1; Path=/"),
            _ => Reply::markdown("b"),
        });
        assert_eq!(source(&server.url("/a.md")), Ok("b".into()));
        assert!(server.requests().iter().all(|request| !request.headers.contains_key("cookie")));
    }

    #[test]
    fn typed_credentials_never_follow_a_redirect() {
        let target = serve(Reply::markdown("b"));
        let target_url = target.url("/b.md");
        let origin = Server::start(move |_| Reply::redirect(302, &target_url));
        let typed = origin.url("/a.md").replace("http://", "http://user:pw@");
        assert_eq!(source(&typed), Ok("b".into()));
        assert!(origin.requests()[0].headers.contains_key("authorization"), "the typed host gets them");
        assert!(!target.requests()[0].headers.contains_key("authorization"));
    }
}
