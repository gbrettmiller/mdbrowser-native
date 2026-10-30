//! Steps for features/keyboard-access.feature: the keyboard rules the widgets use, without a window.

use std::fs;

use cucumber::{given, then, when};
use iced::keyboard::{Key, Modifiers, key::Named};
use mdbrowser::layout::{Direction, Page, Reach, Target, reach};
use mdbrowser::session::Event;
use mdbrowser::ui::app::MIN_WINDOW;
use mdbrowser::ui::keys;
use mdbrowser::ui::panels::{self, Panel};
use mdbrowser::ui::viewport::Viewport;

use super::reader::{WIDTH, look, page_at};
use crate::world::Browser;

fn write(w: &Browser, name: &str, text: &str) {
    fs::write(w.path(name), text).unwrap();
}

// ── Documents ───────────────────────────────────────────────────────────────

#[given(expr = "a document {string} with {int} links")]
fn with_links(w: &mut Browser, name: String, count: usize) {
    let body: String = (1..=count).map(|i| format!("Link [number {i}](l{i}.md).\n\n")).collect();
    write(w, &name, &body);
    (1..=count).for_each(|i| write(w, &format!("l{i}.md"), &format!("# Linked {i}\n")));
}

#[given(expr = "a document {string} with no links")]
fn without_links(w: &mut Browser, name: String) {
    write(w, &name, "Just words.\n\n# And a heading\n");
}

#[given(expr = "a document {string} whose only link wraps")]
fn wrapped(w: &mut Browser, name: String) {
    write(w, &name, &format!("[{}](wrapped-target.md)\n", "word ".repeat(60)));
}

#[given(expr = "a document {string} with a link at the top and another far below")]
fn far_apart(w: &mut Browser, name: String) {
    let filler = "Filler text.\n\n".repeat(80);
    write(w, &name, &format!("[first](a.md)\n\n{filler}[last](b.md)\n"));
}

#[given(expr = "a document {string} that is {int} paragraphs long")]
fn long(w: &mut Browser, name: String, paragraphs: usize) {
    write(w, &name, &"A paragraph of some words.\n\n".repeat(paragraphs));
}

// ── The keyboard ────────────────────────────────────────────────────────────

fn page(w: &Browser) -> Page {
    page_at(w, look(w))
}

/// The line step and page step the theme gives at the reader's settings.
fn line(w: &Browser) -> f32 {
    let theme = mdbrowser::theme::reading(look(w));
    theme.body_size * theme.line_ratio
}

fn window_height(w: &Browser) -> f32 {
    w.keys.window.1
}

fn viewport(w: &Browser) -> Viewport {
    let mut viewport = Viewport::new(WIDTH, window_height(w));
    viewport.scrolled(w.keys.scroll);
    viewport
}

fn named(key: &str) -> Key {
    Key::Named(match key {
        "Tab" => Named::Tab,
        "Enter" => Named::Enter,
        "Space" => Named::Space,
        "PageUp" => Named::PageUp,
        "PageDown" => Named::PageDown,
        "Home" => Named::Home,
        "End" => Named::End,
        "Up" => Named::ArrowUp,
        "Down" => Named::ArrowDown,
        other => panic!("no key named {other}"),
    })
}

#[given(expr = "the window is {int} pixels tall")]
fn window_tall(w: &mut Browser, height: f32) {
    w.keys.window.1 = height;
}

#[given(expr = "the window is {int} pixels wide")]
fn window_wide(w: &mut Browser, width: f32) {
    w.keys.window.0 = width;
}

#[given(expr = "the focus is on link {int}")]
fn focus_on_link(w: &mut Browser, number: usize) {
    w.keys.focus = Some(number - 1);
    w.keys.page_focused = true;
}

#[given("the focus is on the address bar")]
#[when("the focus is on the address bar")]
fn focus_on_address(w: &mut Browser) {
    w.keys.page_focused = false;
}

#[when(expr = "I press {string}")]
fn press(w: &mut Browser, chord: String) {
    let (shift, key) = chord.strip_prefix("Shift+").map_or((false, chord.as_str()), |key| (true, key));
    let modifiers = if shift { Modifiers::SHIFT } else { Modifiers::empty() };
    let key = named(key);
    match key {
        Key::Named(Named::Tab) => tab(w, if shift { Direction::Back } else { Direction::Forward }),
        Key::Named(Named::Enter) => enter(w),
        _ => {
            let Some(by) = keys::scroll(key.as_ref(), modifiers, w.keys.page_focused) else { return };
            let content = page(w).height;
            let line = line(w);
            if let Some(y) = viewport(w).key_scroll(by, content, line) {
                w.keys.scroll = y;
            }
        }
    }
}

fn tab(w: &mut Browser, direction: Direction) {
    let page = page(w);
    let stops = page.stops();
    match reach(stops.len(), w.keys.focus, direction) {
        Reach::Stop(i) => {
            w.keys.focus = Some(i);
            w.keys.left = None;
            let (top, bottom) = stops[i].span();
            if let Some(y) = viewport(w).reveal(top, bottom) {
                w.keys.scroll = y;
            }
        }
        Reach::Leave => {
            w.keys.focus = None;
            w.keys.left = Some(direction);
        }
    }
}

fn enter(w: &mut Browser) {
    let stops = page(w).stops();
    let stop = stops.get(w.keys.focus.expect("a focused link")).expect("that link exists");
    w.send(match &stop.target {
        Target::Link(href) => Event::Follow(href.clone()),
        Target::Image(src) => Event::LoadImage(src.clone()),
    });
}

#[then(expr = "the focus is on link {int}")]
fn on_link(w: &mut Browser, number: usize) {
    assert_eq!(w.keys.focus, Some(number - 1));
}

#[then("the focus is on the next control after the page")]
fn after_page(w: &mut Browser) {
    assert_eq!((w.keys.focus, w.keys.left), (None, Some(Direction::Forward)));
}

#[then("the focus is on the previous control before the page")]
fn before_page(w: &mut Browser) {
    assert_eq!((w.keys.focus, w.keys.left), (None, Some(Direction::Back)));
}

#[then(expr = "there is {int} link stop")]
fn stop_count(w: &mut Browser, count: usize) {
    assert_eq!(page(w).stops().len(), count);
}

#[then("the ring covers several lines")]
fn ring_lines(w: &mut Browser) {
    let stops = page(w).stops();
    assert!(stops[0].rects.len() > 1, "{} rectangles", stops[0].rects.len());
}

#[then(expr = "link {int} is in view")]
fn in_view(w: &mut Browser, number: usize) {
    let stops = page(w).stops();
    let rect = stops[number - 1].rects[0];
    let (top, height) = (w.keys.scroll, window_height(w));
    assert!(rect.y >= top && rect.y + rect.height <= top + height, "{rect:?} not within {top}..{}", top + height);
}

// ── Scroll results ──────────────────────────────────────────────────────────

fn end(w: &Browser) -> f32 {
    page(w).height - window_height(w)
}

fn page_step(w: &Browser) -> f32 {
    window_height(w) - 3.0 * line(w)
}

#[then("the page is scrolled one page down")]
fn one_page_down(w: &mut Browser) {
    assert_close(w.keys.scroll, page_step(w));
}

#[then("the page is scrolled one line down")]
fn one_line_down(w: &mut Browser) {
    assert_close(w.keys.scroll, line(w));
}

#[then("the page is at the end")]
fn to_the_end(w: &mut Browser) {
    assert_close(w.keys.scroll, end(w));
}

#[then("the page is scrolled two pages up from the end")]
fn two_pages_up_from_end(w: &mut Browser) {
    assert_close(w.keys.scroll, end(w) - 2.0 * page_step(w));
}

#[then("the page is scrolled one page up from the end")]
fn one_page_up_from_end(w: &mut Browser) {
    assert_close(w.keys.scroll, end(w) - page_step(w));
}

#[then("the page is at the top")]
fn at_top(w: &mut Browser) {
    assert_close(w.keys.scroll, 0.0);
}

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.5, "scrolled to {actual}, expected {expected}");
}

// ── Window and panels ───────────────────────────────────────────────────────

#[then(expr = "the smallest window is {int} by {int}")]
fn smallest_window(_w: &mut Browser, width: f32, height: f32) {
    assert_eq!(MIN_WINDOW, (width, height));
}

fn panel(name: &str) -> Panel {
    match name {
        "History" => Panel::History,
        "Settings" => Panel::Settings,
        other => panic!("no panel named {other}"),
    }
}

#[given(expr = "the {word} panel is open")]
#[when(expr = "I open the {word} panel")]
fn open_panel(w: &mut Browser, name: String) {
    w.keys.panels = panels::toggled(w.keys.panels, panel(&name), w.keys.window.0);
}

#[then(expr = "the {word} panel is open")]
fn panel_open(w: &mut Browser, name: String) {
    assert!(w.keys.panels.has(panel(&name)));
}

#[then(expr = "the {word} panel is closed")]
fn panel_closed(w: &mut Browser, name: String) {
    assert!(!w.keys.panels.has(panel(&name)));
}

#[then("both panels are open")]
fn both_open(w: &mut Browser) {
    assert!(w.keys.panels.history && w.keys.panels.settings);
}
