//! Steps for features/reader-controls.feature: text size, spacing and custom colours.

use cucumber::{given, then, when};
use mdbrowser::content;
use mdbrowser::layout::{self, Page};
use mdbrowser::session::{Event, NoticeKind, Status, TextStep};
use mdbrowser::theme::colours::{hex, parse_hex};
use mdbrowser::theme::{self, Colours, Look, Palette, Spacing, TextSize, Which};

use crate::world::Browser;

pub(super) const WIDTH: f32 = 600.0;

fn spacing_named(name: &str) -> Spacing {
    Spacing::ALL.into_iter().find(|s| content::spacing(*s) == name).expect("a spacing menu label")
}

fn colour_named(name: &str) -> Which {
    Which::ALL.into_iter().find(|w| content::colour_name(*w) == name).expect("a colour field label")
}

/// The open document laid out at `look`, with no images.
pub(super) fn page_at(w: &Browser, look: Look) -> Page {
    let Status::Ready(shown) = &w.state().status else { panic!("no document: {:?}", w.state().status) };
    theme::fonts::install();
    layout::layout(&shown.document.blocks, &theme::reading(look), WIDTH, &layout::no_images)
}

/// How the reader has the page now.
pub(super) fn look(w: &Browser) -> Look {
    let settings = &w.state().settings;
    Look { font: settings.font, palette: w.state().palette(), size: settings.text_size, spacing: settings.spacing }
}

fn body_size(page: &Page) -> f32 {
    page.fragments.iter().map(|f| f.size).fold(f32::MAX, f32::min)
}

#[then(expr = "the text size is {string}")]
fn text_size_is(w: &mut Browser, size: String) {
    assert_eq!(content::text_size(w.state().settings.text_size), size);
}

#[then(expr = "the spacing is {string}")]
fn spacing_is(w: &mut Browser, name: String) {
    assert_eq!(content::spacing(w.state().settings.spacing), name);
}

#[when("I make the text larger")]
#[given("I made the text larger")]
fn larger(w: &mut Browser) {
    w.send(Event::StepTextSize(TextStep::Larger));
}

#[when(expr = "I make the text larger {int} times")]
fn larger_times(w: &mut Browser, times: u32) {
    (0..times).for_each(|_| larger(w));
}

#[when(expr = "I make the text smaller {int} times")]
fn smaller_times(w: &mut Browser, times: u32) {
    (0..times).for_each(|_| w.send(Event::StepTextSize(TextStep::Smaller)));
}

#[when("I reset the text size")]
fn reset_size(w: &mut Browser) {
    w.send(Event::StepTextSize(TextStep::Reset));
}

#[when("I reset the reader controls")]
fn reset_controls(w: &mut Browser) {
    w.send(Event::ResetReaderControls);
}

#[then("the text size is the largest step")]
fn largest(w: &mut Browser) {
    let largest = (0..100).fold(TextSize::default(), |size, _| size.larger());
    assert_eq!(w.state().settings.text_size, largest);
}

#[then("the text size is the smallest step")]
fn smallest(w: &mut Browser) {
    let smallest = (0..100).fold(TextSize::default(), |size, _| size.smaller());
    assert_eq!(w.state().settings.text_size, smallest);
}

#[then("the document is set at a larger size")]
fn set_larger(w: &mut Browser) {
    let (now, default) = (page_at(w, look(w)), page_at(w, Look { size: TextSize::default(), ..look(w) }));
    assert!(body_size(&now) > body_size(&default), "{} vs {}", body_size(&now), body_size(&default));
}

#[when(expr = "I choose the spacing {string}")]
#[given(expr = "I chose the spacing {string}")]
fn choose_spacing(w: &mut Browser, name: String) {
    w.send(Event::ChooseSpacing(spacing_named(&name)));
}

#[then(expr = "the page is shorter than at {string}")]
fn shorter(w: &mut Browser, name: String) {
    let (now, other) =
        (page_at(w, look(w)).height, page_at(w, Look { spacing: spacing_named(&name), ..look(w) }).height);
    assert!(now < other, "{now} vs {other}");
}

#[then(expr = "the page is taller than at {string}")]
fn taller(w: &mut Browser, name: String) {
    let (now, other) =
        (page_at(w, look(w)).height, page_at(w, Look { spacing: spacing_named(&name), ..look(w) }).height);
    assert!(now > other, "{now} vs {other}");
}

#[when(expr = "I set the custom {string} colour to {string}")]
#[given(expr = "I set the custom {string} colour to {string}")]
fn set_colour(w: &mut Browser, which: String, typed: String) {
    w.colours_before = Some(w.state().settings.colours());
    w.send(Event::ChooseCustomColour(colour_named(&which), typed));
}

#[then(expr = "the custom {string} colour is {string}")]
fn colour_is(w: &mut Browser, which: String, typed: String) {
    let expected = parse_hex(&typed).unwrap_or_else(|| panic!("{typed:?} should be a colour"));
    let actual = w.state().settings.colours().get(colour_named(&which));
    assert_eq!(hex(actual), hex(expected));
}

#[then(expr = "the custom {string} colour is what it was")]
fn colour_unchanged(w: &mut Browser, which: String) {
    let before = w.colours_before.expect("a colour was set first");
    assert_eq!(w.state().settings.colours().get(colour_named(&which)), before.get(colour_named(&which)));
}

#[then("the custom page colour is the dark page colour")]
fn page_is_dark_page(w: &mut Browser) {
    assert_eq!(w.state().settings.colours().page, Colours::of(Palette::Dark).page);
}

#[then(expr = "I am told that the {word} colour is too close to the page colour")]
fn told_too_close(w: &mut Browser, which: String) {
    let kind = match which.as_str() {
        "text" => NoticeKind::UnreadableText,
        "link" => NoticeKind::UnreadableLink,
        other => panic!("no notice for the {other} colour"),
    };
    let kinds: Vec<_> = w.state().notices.iter().map(|n| n.kind).collect();
    assert_eq!(kinds, [kind], "notices: {kinds:?}");
}
