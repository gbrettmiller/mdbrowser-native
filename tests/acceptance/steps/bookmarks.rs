//! Steps for features/bookmarks.feature.

use cucumber::{given, then, when};
use mdbrowser::session::Event;
use mdbrowser::session::bookmarks::Move;

use super::local::viewing;
use crate::world::Browser;

fn titles(w: &Browser) -> Vec<String> {
    w.state().bookmarks.iter().map(|b| b.title.clone()).collect()
}

/// The address of the bookmark titled `title`.
fn address_of(w: &Browser, title: &str) -> String {
    let found = w.state().bookmarks.iter().find(|b| b.title == title);
    found.unwrap_or_else(|| panic!("no bookmark titled {title}: {:?}", titles(w))).address.clone()
}

#[given("I bookmarked this page")]
#[when("I bookmark this page")]
fn bookmark_this(w: &mut Browser) {
    w.send(Event::ToggleBookmark);
}

#[given(regex = r#"^I bookmarked "([^"]+)" then "([^"]+)"$"#)]
fn bookmarked_two(w: &mut Browser, first: String, second: String) {
    for name in [first, second] {
        viewing(w, name);
        bookmark_this(w);
    }
}

#[given(regex = r#"^I moved "([^"]+)" (up|down)$"#)]
#[when(regex = r#"^I move "([^"]+)" (up|down)$"#)]
fn move_bookmark(w: &mut Browser, title: String, way: String) {
    let by = if way == "up" { Move::Up } else { Move::Down };
    let address = address_of(w, &title);
    w.send(Event::MoveBookmark(address, by));
}

#[when(expr = "I remove the bookmark {string}")]
fn remove_bookmark(w: &mut Browser, title: String) {
    let address = address_of(w, &title);
    w.send(Event::RemoveBookmark(address));
}

#[when(expr = "I open the bookmark {string}")]
fn open_bookmark(w: &mut Browser, title: String) {
    let address = address_of(w, &title);
    w.send(Event::OpenBookmark(address));
}

#[then("this page is bookmarked")]
fn is_bookmarked(w: &mut Browser) {
    assert!(w.state().is_bookmarked());
}

#[then("this page is not bookmarked")]
fn is_not_bookmarked(w: &mut Browser) {
    assert!(!w.state().is_bookmarked());
}

#[then(expr = "Bookmarks lists {string}")]
#[then(expr = "Bookmarks lists only {string}")]
fn lists_one(w: &mut Browser, title: String) {
    assert_eq!(titles(w), [title]);
}

#[then(expr = "Bookmarks lists {string} then {string}")]
fn lists_two(w: &mut Browser, first: String, second: String) {
    assert_eq!(titles(w), [first, second]);
    assert_eq!(w.store.load_bookmarks(), w.state().bookmarks, "saved as shown");
}

#[then("Bookmarks is empty")]
fn is_empty(w: &mut Browser) {
    assert!(w.state().bookmarks.is_empty(), "{:?}", titles(w));
    assert!(w.store.load_bookmarks().is_empty());
}
