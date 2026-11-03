//! Steps for features/source-view.feature.

use std::fs;

use cucumber::{given, then, when};
use mdbrowser::model::Block;
use mdbrowser::session::{Event, Status, View};

use crate::world::Browser;

use super::unescape;

fn source(w: &Browser) -> String {
    match &w.state().view {
        View::Source(document) => match document.blocks.as_slice() {
            [Block::Code { text, .. }] => text.clone(),
            other => panic!("the source is one code block, got {other:?}"),
        },
        View::Page => panic!("not in the source view"),
    }
}

#[when("I view the source")]
#[given("I viewed the source")]
fn view_source(w: &mut Browser) {
    w.send(Event::ToggleSource);
}

#[when("I view the page")]
#[given("I viewed the page")]
fn view_page(w: &mut Browser) {
    assert!(matches!(w.state().view, View::Source(_)), "not in the source view");
    w.send(Event::ToggleSource);
}

#[then("I see the source view")]
fn sees_source(w: &mut Browser) {
    assert!(matches!(w.state().view, View::Source(_)), "view: {:?}", w.state().view);
}

#[then("I see the rendered page")]
fn sees_page(w: &mut Browser) {
    assert_eq!(w.state().view, View::Page);
    assert!(matches!(w.state().status, Status::Ready(_)), "status: {:?}", w.state().status);
}

#[then("I am not in the source view")]
fn not_in_source(w: &mut Browser) {
    assert_eq!(w.state().view, View::Page);
}

#[then(expr = "the source is the text of {string}")]
fn source_is_file(w: &mut Browser, name: String) {
    assert_eq!(source(w), fs::read_to_string(w.path(&name)).unwrap());
}

#[then(expr = "the source starts with the line {string}")]
fn source_starts_with(w: &mut Browser, line: String) {
    assert_eq!(source(w).lines().next(), Some(line.as_str()));
}

#[then(expr = "the source has no line {string}")]
fn source_lacks_line(w: &mut Browser, line: String) {
    assert!(!source(w).lines().any(|l| l == line), "source: {:?}", source(w));
}

#[then("the source is empty")]
fn source_empty(w: &mut Browser) {
    assert_eq!(source(w), "");
}

#[then(expr = "the source is {string}")]
fn source_is(w: &mut Browser, expected: String) {
    assert_eq!(source(w), unescape(&expected));
}

#[when(expr = "the file {string} changes to {string}")]
fn file_changes(w: &mut Browser, name: String, content: String) {
    fs::write(w.path(&name), unescape(&content)).unwrap();
}

#[given(expr = "a file {string} with Windows line endings and a byte order mark")]
fn crlf_file_with_bom(w: &mut Browser, name: String) {
    fs::write(w.path(&name), "\u{feff}# Title\r\n\r\nline one\r\nline two\r\n").unwrap();
}

#[then("the saved settings are the session's")]
fn saved_settings_match(w: &mut Browser) {
    assert_eq!(w.store.load_settings(), w.state().settings);
}
