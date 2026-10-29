//! Steps for local files: features/browse.feature and features/reading.feature.

use std::collections::BTreeSet;
use std::fs;

use cucumber::{given, then, when};
use iced::advanced::graphics::text::font_system;
use mdbrowser::content;
use mdbrowser::layout;
use mdbrowser::session::location::Location;
use mdbrowser::session::visits::Visit;
use mdbrowser::session::{Event, Status};
use mdbrowser::theme::{self, ReadingFont, fonts};

use crate::world::Browser;

use super::{enter, shown_title, unescape};

// ── Files and fixtures ──────────────────────────────────────────────────────

#[given(expr = "a file {string} containing {string}")]
fn a_file_containing(w: &mut Browser, name: String, content: String) {
    fs::write(w.path(&name), unescape(&content)).unwrap();
}

#[given(expr = "{string} exists with invalid UTF-8 content")]
fn invalid_utf8(w: &mut Browser, name: String) {
    fs::write(w.path(&name), [0xff, 0xfe, 0x00, 0x80]).unwrap();
}

#[when(expr = "{string} is changed to have title {string}")]
fn retitled(w: &mut Browser, name: String, title: String) {
    fs::write(w.path(&name), format!("---\ntitle: {title}\n---\n# Welcome\n")).unwrap();
}

#[given(expr = "{string} is deleted")]
#[when(expr = "{string} is deleted")]
fn deleted(w: &mut Browser, name: String) {
    fs::remove_file(w.path(&name)).unwrap();
}

#[when(expr = "{string} is created")]
fn created(w: &mut Browser, name: String) {
    fs::write(w.path(&name), "later").unwrap();
}

#[given("the system opener fails")]
fn opener_fails(w: &mut Browser) {
    w.opener_fails = true;
}

#[given(expr = "loading {string} is slow")]
fn slow(w: &mut Browser, name: String) {
    w.hold(&name);
}

// ── Entering addresses ──────────────────────────────────────────────────────

#[when(expr = "I enter the absolute path of {string}")]
#[when(expr = "I view {string}")]
#[when(expr = "I enter the absolute path of {string} before it finishes")]
fn enter_absolute(w: &mut Browser, name: String) {
    let address = w.absolute(&name);
    enter(w, address);
}

#[when(regex = r#"^I enter the home-relative ~/ path of "([^"]*)"$"#)]
fn enter_home_relative(w: &mut Browser, name: String) {
    enter(w, format!("~/{name}"));
}

#[when(expr = "I enter the file:\\/\\/ URL of {string}")]
fn enter_file_url(w: &mut Browser, name: String) {
    let address = w.file_url(&name);
    enter(w, address);
}

#[when(expr = "I enter {string}")]
fn enter_literal(w: &mut Browser, input: String) {
    enter(w, input);
}

#[given(expr = "I entered the absolute path of {string} and see the error {string}")]
fn entered_and_failed(w: &mut Browser, name: String, code: String) {
    enter_absolute(w, name);
    see_error(w, code);
}

#[given(expr = "I am viewing {string}")]
#[given(expr = "I viewed {string}")]
fn viewing(w: &mut Browser, name: String) {
    enter_absolute(w, name.clone());
    still_viewing(w, name);
}

#[given(regex = r#"^I viewed "([^"]+)" then "([^"]+)"( and went back)?$"#)]
fn viewed_two(w: &mut Browser, first: String, second: String, went_back: String) {
    viewing(w, first);
    viewing(w, second);
    if !went_back.is_empty() {
        w.send(Event::Back);
    }
}

#[given(regex = r#"^I viewed "([^"]+)" then "([^"]+)" then "([^"]+)"$"#)]
fn viewed_three(w: &mut Browser, first: String, second: String, third: String) {
    [first, second, third].into_iter().for_each(|name| viewing(w, name));
}

// ── Links, navigation, reload ───────────────────────────────────────────────

#[when(expr = "I follow {string}")]
fn follow(w: &mut Browser, href: String) {
    w.send(Event::Follow(href));
}

#[when("I go back")]
fn go_back(w: &mut Browser) {
    w.send(Event::Back);
}

#[when("I go forward")]
fn go_forward(w: &mut Browser) {
    w.send(Event::Forward);
}

#[when("I reload")]
fn reload(w: &mut Browser) {
    w.send(Event::Reload);
}

#[when("I restart the app")]
fn restart(w: &mut Browser) {
    w.restart();
}

#[then(expr = "{string} does not replace it when it finishes")]
fn late_load_ignored(w: &mut Browser, name: String) {
    let before = shown_title(w);
    w.release(&name);
    assert_eq!(shown_title(w), before);
}

// ── What the reader sees ────────────────────────────────────────────────────

#[then(expr = "I see the document titled {string}")]
fn see_title(w: &mut Browser, title: String) {
    assert_eq!(shown_title(w).as_deref(), Some(title.as_str()), "status: {:?}", w.state().status);
}

#[then(expr = "I see the error {string}")]
pub(super) fn see_error(w: &mut Browser, code: String) {
    match &w.state().status {
        Status::Failed(actual) => assert_eq!(actual.key(), code),
        other => panic!("expected error {code}, got {other:?}"),
    }
}

#[then(expr = "the error shows the address {string}")]
fn error_address(w: &mut Browser, input: String) {
    assert_eq!(w.state().address(), input);
}

#[then(expr = "the address shows the absolute path of {string}")]
fn address_is(w: &mut Browser, name: String) {
    assert_eq!(w.state().address(), w.absolute(&name));
}

#[then("nothing was read from disk")]
fn nothing_read(w: &mut Browser) {
    assert!(w.reads.is_empty(), "read {:?}", w.reads);
}

#[then(expr = "the document text is {string}")]
fn text_is(w: &mut Browser, expected: String) {
    let Status::Ready(shown) = &w.state().status else { panic!("no document: {:?}", w.state().status) };
    assert_eq!(shown.document.plain_text(), unescape(&expected));
}

#[then(expr = "I am still viewing {string}")]
fn still_viewing(w: &mut Browser, name: String) {
    let expected = Location::file(w.path(&name));
    assert_eq!(w.state().current_location(), Some(&expected));
    assert!(matches!(w.state().status, Status::Ready(_)), "status: {:?}", w.state().status);
}

#[then(expr = "the pending fragment is {string}")]
fn pending_fragment(w: &mut Browser, fragment: String) {
    assert_eq!(w.state().fragment.as_deref(), Some(fragment.as_str()));
}

#[then(regex = r"^(back|forward) is (enabled|disabled)$")]
fn button_state(w: &mut Browser, button: String, enabled: String) {
    let nav = &w.state().nav;
    let actual = if button == "back" { nav.can_go_back() } else { nav.can_go_forward() };
    assert_eq!(actual, enabled == "enabled", "{button}");
}

#[then(expr = "I am told {string}")]
fn told(w: &mut Browser, message: String) {
    let notices: Vec<_> = w.state().notices.iter().map(|n| content::notice(n.kind)).collect();
    assert!(notices.contains(&message.as_str()), "notices: {notices:?}");
}

#[then(expr = "{string} is opened externally")]
fn opened(w: &mut Browser, url: String) {
    assert_eq!(w.opened, [url]);
}

#[then("nothing is opened externally")]
fn nothing_opened(w: &mut Browser) {
    assert!(w.opened.is_empty(), "opened {:?}", w.opened);
}

// ── History ─────────────────────────────────────────────────────────────────

fn distinct_documents(count: usize) -> Vec<Visit> {
    (0..count)
        .map(|i| Visit {
            address: format!("/nowhere/doc-{i}.md"),
            title: format!("Doc {i}"),
            visited_at: (count - i) as u64,
        })
        .collect()
}

#[given(regex = r#"^History already holds 500 distinct documents( including "([^"]+)")?$"#)]
fn history_full(w: &mut Browser, _including: String, name: String) {
    let mut visits = distinct_documents(500);
    if !name.is_empty() {
        visits[250] = Visit { address: w.absolute(&name), title: "Home".into(), visited_at: 250 };
    }
    w.seed_history(visits);
}

#[given(expr = "History lists {string}")]
fn history_seeded(w: &mut Browser, name: String) {
    let visit = Visit { address: w.absolute(&name), title: "Note A".into(), visited_at: 1 };
    w.seed_history(vec![visit]);
}

#[when(expr = "I open the History entry for {string}")]
fn open_history(w: &mut Browser, name: String) {
    let address = w.absolute(&name);
    w.send(Event::OpenHistory(address));
}

#[when("I clear History")]
fn clear_history(w: &mut Browser) {
    w.send(Event::ClearHistory);
}

#[then(expr = "History lists {string} then {string}")]
fn history_order(w: &mut Browser, first: String, second: String) {
    assert_eq!(w.history_titles(), [first, second]);
}

#[then(expr = "History does not list {string}")]
fn history_excludes(w: &mut Browser, name: String) {
    assert!(w.state().visits.iter().all(|v| !v.address.ends_with(&name)), "{:?}", w.state().visits);
}

#[then(expr = "History holds {int} entries")]
fn history_size(w: &mut Browser, count: usize) {
    assert_eq!(w.state().visits.len(), count);
}

#[then("the oldest document is not listed")]
fn oldest_evicted(w: &mut Browser) {
    assert!(!w.history_titles().contains(&"Doc 499".to_string()));
}

#[then(expr = "{string} is first in History")]
fn first_in_history(w: &mut Browser, title: String) {
    assert_eq!(w.history_titles().first(), Some(&title));
}

#[then("History is empty")]
fn history_empty(w: &mut Browser) {
    assert!(w.state().visits.is_empty());
    assert!(w.store.load_visits().is_empty());
}

// ── Reading font ────────────────────────────────────────────────────────────

/// The font whose menu label is `name`.
fn font_named(name: &str) -> ReadingFont {
    ReadingFont::ALL.into_iter().find(|font| content::font_name(*font) == name).expect("a font menu label")
}

#[when(expr = "I choose the reading font {string}")]
#[given(expr = "I chose the reading font {string}")]
fn choose_font(w: &mut Browser, name: String) {
    w.send(Event::ChooseFont(font_named(&name)));
}

#[given(expr = "the stored reading font is {string}")]
fn stored_font(w: &mut Browser, key: String) {
    let path = w.home.path().join(".mdbrowser/settings.json");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, format!(r#"{{"font":"{key}"}}"#)).unwrap();
}

#[then(expr = "the reading font is {string}")]
fn reading_font_is(w: &mut Browser, name: String) {
    assert_eq!(w.state().settings.font, font_named(&name));
}

/// The family name inside each bundled font file, independent of the app's own mapping.
fn family_in_file(menu_label: &str) -> &'static str {
    match menu_label {
        "Literata" => "Literata",
        "Source Serif 4" => "Source Serif 4",
        "iA Writer Quattro" => "iA Writer Quattro S",
        "Inter" => "Inter",
        other => panic!("no such font: {other}"),
    }
}

/// Checks the faces that actually shaped the page's glyphs, so a wrong
/// mapping or a silent fallback to another family fails.
#[then(expr = "the document is set in {string}")]
fn set_in(w: &mut Browser, name: String) {
    let Status::Ready(shown) = &w.state().status else { panic!("no document: {:?}", w.state().status) };
    fonts::install();
    let page =
        layout::layout(&shown.document.blocks, &theme::reading(w.state().settings.font), 720.0, &layout::no_images);
    let mut system = font_system().write().unwrap();
    let db = system.raw().db();
    let families: BTreeSet<String> = page
        .texts
        .iter()
        .flat_map(|text| {
            text.buffer.layout_runs().flat_map(|run| run.glyphs.iter().map(|g| g.font_id)).collect::<Vec<_>>()
        })
        .filter_map(|id| db.face(id).map(|face| face.families[0].0.clone()))
        .filter(|family| family != theme::MONO_FAMILY)
        .collect();
    assert_eq!(families, BTreeSet::from([family_in_file(&name).to_string()]));
}
