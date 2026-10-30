//! Steps for features/frontmatter.feature. They check the front matter the session holds and the
//! blocks the page is built from, not where or in what type those are drawn.

use cucumber::{then, when};

use mdbrowser::content;
use mdbrowser::model::{Block, Document, Frame, Labels, assemble, frame};
use mdbrowser::session::{Event, Status};

use crate::world::Browser;

const LABELS: Labels = Labels { listen: content::LISTEN, previous: content::PREVIOUS, next: content::NEXT };

fn shown(w: &Browser) -> std::sync::Arc<Document> {
    match &w.state().status {
        Status::Ready(shown) => shown.document.clone(),
        other => panic!("{other:?}"),
    }
}

fn framed(w: &Browser) -> Frame {
    frame(&shown(w))
}

/// Every line of text on the page, front matter included.
fn page_lines(w: &Browser) -> Vec<String> {
    let page = Document { blocks: assemble(&shown(w), &LABELS), ..Document::default() };
    page.plain_text().lines().map(str::to_string).collect()
}

#[then(expr = "the byline reads {string}")]
fn byline_reads(w: &mut Browser, text: String) {
    assert_eq!(framed(w).byline.as_deref(), Some(text.as_str()));
    assert!(page_lines(w).contains(&text), "{:?}", page_lines(w));
}

#[then("there is no byline")]
fn no_byline(w: &mut Browser) {
    assert_eq!(framed(w).byline, None);
}

#[then(expr = "the description reads {string}")]
fn description_reads(w: &mut Browser, text: String) {
    assert_eq!(framed(w).description.as_deref(), Some(text.as_str()));
    assert!(page_lines(w).contains(&text), "{:?}", page_lines(w));
}

#[then(expr = "the text {string} appears once on the page")]
fn appears_once(w: &mut Browser, text: String) {
    assert_eq!(page_lines(w).iter().filter(|line| **line == text).count(), 1, "{:?}", page_lines(w));
}

#[then("the page offers Previous and Next")]
fn offers_both(w: &mut Browser) {
    offers(w, true, true);
}

#[then("the page offers Previous only")]
fn offers_previous(w: &mut Browser) {
    offers(w, true, false);
}

#[then("the page offers no Previous or Next")]
fn offers_neither(w: &mut Browser) {
    offers(w, false, false);
}

fn offers(w: &Browser, previous: bool, next: bool) {
    let found = framed(w);
    assert_eq!((found.previous.is_some(), found.next.is_some()), (previous, next), "{found:?}");
    let text = page_lines(w).join("\n");
    assert_eq!(text.contains(content::PREVIOUS), previous, "{text}");
    assert_eq!(text.contains(content::NEXT), next, "{text}");
}

#[when("I choose Previous")]
fn choose_previous(w: &mut Browser) {
    let href = framed(w).previous.expect("a Previous link");
    w.send(Event::Follow(href));
}

#[when("I choose Next")]
fn choose_next(w: &mut Browser) {
    let href = framed(w).next.expect("a Next link");
    w.send(Event::Follow(href));
}

#[then(expr = "the enclosure line reads {string}")]
fn enclosure_reads(w: &mut Browser, text: String) {
    assert!(page_lines(w).contains(&text), "{:?}", page_lines(w));
}

#[then("there is no enclosure line")]
fn no_enclosure(w: &mut Browser) {
    assert_eq!(framed(w).enclosure, None);
    assert!(!page_lines(w).iter().any(|line| line.starts_with(content::LISTEN)), "{:?}", page_lines(w));
}

#[when("I choose the enclosure")]
fn choose_enclosure(w: &mut Browser) {
    let url = framed(w).enclosure.expect("an enclosure").url;
    w.send(Event::Follow(url));
}

#[then("the page shows no images")]
fn no_images(w: &mut Browser) {
    let blocks = assemble(&shown(w), &LABELS);
    assert!(!blocks.iter().any(|block| matches!(block, Block::Image { .. })), "{blocks:?}");
}

#[then(expr = "History lists {string} titled {string}")]
fn history_lists_titled(w: &mut Browser, name: String, title: String) {
    let address = w.absolute(&name);
    assert!(w.state().visits.iter().any(|v| v.address == address && v.title == title), "{:?}", w.state().visits);
}
