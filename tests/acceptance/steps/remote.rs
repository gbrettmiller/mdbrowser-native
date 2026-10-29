//! Steps for pages on the test server: features/remote.feature.

use std::fs;
use std::net::TcpListener;

use cucumber::{given, then, when};
use mdbrowser::content;
use mdbrowser::session::location::Location;
use mdbrowser::session::{Event, Status};

use super::local::see_error;
use super::{enter, shown_title, unescape};
use crate::http::Reply;
use crate::world::Browser;

#[when(expr = "I enter the URL of {string}")]
fn enter_url(w: &mut Browser, path: String) {
    let url = w.url(&path);
    enter(w, url);
}

#[given(expr = "I am viewing the URL of {string}")]
fn viewing_url(w: &mut Browser, path: String) {
    enter_url(w, path.clone());
    assert!(shown_title(w).is_some(), "{path}: {:?}", w.state().status);
}

#[given(expr = "I viewed the URL of {string} then the URL of {string}")]
fn viewed_two_urls(w: &mut Browser, first: String, second: String) {
    viewing_url(w, first);
    viewing_url(w, second);
}

#[given(expr = "I viewed {string} then the URL of {string}")]
fn viewed_file_then_url(w: &mut Browser, name: String, path: String) {
    let address = w.absolute(&name);
    enter(w, address);
    assert!(shown_title(w).is_some(), "{name}: {:?}", w.state().status);
    viewing_url(w, path);
}

#[given(expr = "a file {string} linking to the URL of {string}")]
fn file_linking_to_url(w: &mut Browser, name: String, path: String) {
    let url = w.url(&path);
    file_linking_to(w, name, url);
}

#[given(expr = "a file {string} linking to {string}")]
fn file_linking_to(w: &mut Browser, name: String, href: String) {
    fs::write(w.path(&name), format!("# Links\n\n[there]({href})\n")).unwrap();
}

#[when(expr = "I follow the URL of {string}")]
fn follow_url(w: &mut Browser, path: String) {
    let url = w.url(&path);
    w.send(Event::Follow(url));
}

#[when(expr = "I follow the file:\\/\\/ URL of {string}")]
fn follow_file_url(w: &mut Browser, name: String) {
    let url = w.file_url(&name);
    w.send(Event::Follow(url));
}

#[given(expr = "I entered the URL of {string} and see the error {string}")]
fn entered_url_and_failed(w: &mut Browser, path: String, code: String) {
    enter_url(w, path);
    see_error(w, code);
}

#[when("I enter a URL whose connection is refused")]
fn enter_refused(w: &mut Browser) {
    // Bound and dropped at once, so nothing is listening. Another scenario's server
    // could in theory take the port in between; that would fail loudly as notFound.
    let addr = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap();
    enter(w, format!("http://{addr}/a.md"));
}

#[when(expr = "I open the URL of {string} from History")]
fn open_url_from_history(w: &mut Browser, path: String) {
    let url = w.url(&path);
    w.send(Event::OpenHistory(url));
}

#[given(expr = "{string} now serves {string}")]
fn now_serves(w: &mut Browser, path: String, text: String) {
    w.routes.lock().expect("routes").insert(path, Reply::markdown(&unescape(&text)));
}

#[given(expr = "loading the URL of {string} is slow")]
fn slow_url(w: &mut Browser, path: String) {
    w.hold(&path);
}

#[when(expr = "{string} finishes loading")]
fn finishes_loading(w: &mut Browser, name: String) {
    w.release(&name);
}

#[then(expr = "I am still viewing the URL of {string}")]
fn still_viewing_url(w: &mut Browser, path: String) {
    assert_eq!(w.state().current_location(), Some(&w.web(&path)));
    assert!(shown_title(w).is_some(), "{:?}", w.state().status);
}

#[then(expr = "the system opened {string}")]
fn system_opened(w: &mut Browser, href: String) {
    assert_eq!(w.opened, [href]);
}

#[then(expr = "the address shows the URL of {string}")]
fn address_is_url(w: &mut Browser, path: String) {
    assert_eq!(w.state().address(), w.url(&path));
}

#[then(expr = "the error shows the URL of {string}")]
fn error_shows_url(w: &mut Browser, path: String) {
    assert!(matches!(w.state().status, Status::Failed(_)), "{:?}", w.state().status);
    assert_eq!(w.state().address(), w.url(&path));
}

/// The same choice the error view makes: a hint for pages on the web.
#[then("the error says what to try next")]
fn error_has_hint(w: &mut Browser) {
    let Status::Failed(code) = w.state().status else { panic!("{:?}", w.state().status) };
    let web = w.state().current_location().is_some_and(Location::is_web);
    let (_, hint) = content::failure(code, web);
    assert!(hint.is_some_and(|hint| !hint.is_empty()), "{code:?}");
}

#[then(expr = "History lists the URL of {string} titled {string}")]
fn history_lists_url(w: &mut Browser, path: String, title: String) {
    let url = w.url(&path);
    assert!(w.state().visits.iter().any(|v| v.address == url && v.title == title), "{:?}", w.state().visits);
}

#[then("the site was told this reader prefers Markdown")]
fn told_markdown_first(w: &mut Browser) {
    let requests = w.server.requests();
    let accept = &requests.last().expect("a request").headers["accept"];
    let weight = |range: &str| {
        range.split(';').map(str::trim).find_map(|p| p.strip_prefix("q=")).map_or(1.0, |q| q.parse().unwrap())
    };
    let (markdown, others): (Vec<_>, Vec<_>) = accept.split(',').partition(|r| r.trim().starts_with("text/markdown"));
    let top = weight(markdown.first().expect("text/markdown is asked for"));
    assert!(others.iter().all(|range| weight(range) < top), "{accept}");
}

#[then(regex = r#"^the site has seen (\d+) requests? for "([^"]+)"$"#)]
fn requests_seen(w: &mut Browser, count: usize, path: String) {
    let seen = w.server.requests().iter().filter(|request| request.path == path).count();
    assert_eq!(seen, count);
}
