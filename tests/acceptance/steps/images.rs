//! Steps for images: features/images.feature.

use std::fs;

use cucumber::{given, then, when};
use mdbrowser::content;
use mdbrowser::layout::{self, ImageView, Page};
use mdbrowser::model::Block;
use mdbrowser::session::images::{ImageLoading, Note, View};
use mdbrowser::session::{Event, Status};
use mdbrowser::theme;

use super::unescape;
use crate::fixtures;
use crate::http::Reply;
use crate::world::Browser;

const WIDTH: f32 = 720.0;

/// Every image block in the document, however deeply nested.
fn image_blocks(blocks: &[Block]) -> Vec<&Block> {
    blocks
        .iter()
        .flat_map(|block| match block {
            Block::Image { .. } => vec![block],
            Block::Quote(inner) => image_blocks(inner),
            Block::List { items, .. } => items.iter().flat_map(|item| image_blocks(&item.blocks)).collect(),
            _ => vec![],
        })
        .collect()
}

#[then("no image is shown")]
fn no_image_shown(w: &mut Browser) {
    let sources = image_sources(w);
    assert!(sources.iter().all(|src| !matches!(w.state().images.view(src), View::Shown(_))), "{sources:?}");
}

/// The mode whose menu entry reads "Web images: <name>".
fn mode_named(name: &str) -> ImageLoading {
    let label = format!("Web images: {name}");
    ImageLoading::ALL.into_iter().find(|mode| content::image_loading(*mode) == label).expect("a Web images menu entry")
}

#[given(expr = "I set web images to {string}")]
#[when(expr = "I set web images to {string}")]
fn set_web_images(w: &mut Browser, name: String) {
    w.send(Event::ChooseImageLoading(mode_named(&name)));
}

#[then(expr = "web images are set to {string}")]
fn web_images_set_to(w: &mut Browser, name: String) {
    assert_eq!(w.state().settings.images, mode_named(&name));
}

fn image_sources(w: &Browser) -> Vec<String> {
    let Status::Ready(shown) = &w.state().status else { panic!("no document: {:?}", w.state().status) };
    image_blocks(&shown.document.blocks)
        .into_iter()
        .filter_map(|block| match block {
            Block::Image { src, .. } => Some(src.clone()),
            _ => None,
        })
        .collect()
}

/// The source of the image described as `alt`, or of the only image when `alt` is empty.
fn source_of(w: &Browser, alt: &str) -> String {
    let Status::Ready(shown) = &w.state().status else { panic!("no document: {:?}", w.state().status) };
    let images: Vec<(String, String)> = image_blocks(&shown.document.blocks)
        .into_iter()
        .filter_map(|block| match block {
            Block::Image { src, alt, .. } => Some((src.clone(), alt.clone())),
            _ => None,
        })
        .collect();
    match images.iter().find(|(_, description)| description == alt) {
        Some((src, _)) => src.clone(),
        None if alt.is_empty() && images.len() == 1 => images[0].0.clone(),
        None => panic!("no image described {alt:?} in {images:?}"),
    }
}

/// The page as the app lays it out: notes in the words `content` gives them.
fn laid_page(w: &Browser) -> Page {
    let Status::Ready(shown) = &w.state().status else { panic!("no document: {:?}", w.state().status) };
    let images = &w.state().images;
    let view = |src: &str| match images.view(src) {
        View::Shown((width, height)) => ImageView::Shown { width, height },
        View::Note(note, clickable) => ImageView::Placeholder { note: Some((content::image_note(note), clickable)) },
    };
    theme::fonts::install();
    layout::layout(&shown.document.blocks, &theme::reading(w.state().settings.font), WIDTH, &view)
}

/// Feature files write the shortcut with Ctrl; macOS says Cmd.
fn on_this_platform(note: &str) -> String {
    let modifier = if content::image_note(Note::Off).contains("Cmd+") { "Cmd+" } else { "Ctrl+" };
    note.replace("Ctrl+", modifier)
}

#[given(expr = "an image file {string} of {int} by {int} pixels")]
fn image_file(w: &mut Browser, name: String, width: u32, height: u32) {
    fs::write(w.path(&name), fixtures::png(width, height)).unwrap();
}

#[given(expr = "the site serves {string} containing {string}")]
fn site_serves(w: &mut Browser, path: String, text: String) {
    w.routes.lock().expect("routes").insert(path, Reply::markdown(&unescape(&text)));
}

#[given(expr = "the site serves {string} containing an image from the file:\\/\\/ URL of {string}")]
fn site_serves_file_image(w: &mut Browser, path: String, name: String) {
    let markdown = format!("![secret]({})", w.file_url(&name));
    w.routes.lock().expect("routes").insert(path, Reply::markdown(&markdown));
}

#[given(expr = "the site serves {string} containing an image from {string}")]
fn site_serves_image_from(w: &mut Browser, path: String, src: String) {
    w.routes.lock().expect("routes").insert(path, Reply::markdown(&format!("![secret](<{src}>)")));
}

#[given(expr = "a file {string} linking an image to the URL of {string}")]
fn file_with_web_image(w: &mut Browser, name: String, path: String) {
    let description = path.rsplit('/').next().unwrap_or_default().to_string();
    fs::write(w.path(&name), format!("![{description}]({})", w.url(&path))).unwrap();
}

#[then(expr = "the image {string} is shown at {int} by {int}")]
fn image_shown_at(w: &mut Browser, alt: String, width: u32, height: u32) {
    let src = source_of(w, &alt);
    assert_eq!(w.state().images.view(&src), View::Shown((width, height)), "{src}");
}

#[then(expr = "the image is shown at {int} by {int}")]
fn only_image_shown_at(w: &mut Browser, width: u32, height: u32) {
    image_shown_at(w, String::new(), width, height);
}

#[given(expr = "the image {string} is not shown and says {string}")]
#[then(expr = "the image {string} is not shown and says {string}")]
fn image_not_shown(w: &mut Browser, alt: String, note: String) {
    let src = source_of(w, &alt);
    match w.state().images.view(&src) {
        View::Note(shown, _) => assert_eq!(content::image_note(shown), on_this_platform(&note), "{src}"),
        View::Shown(size) => panic!("{src} is shown at {size:?}"),
    }
}

#[then(expr = "the image is not shown and says {string}")]
fn only_image_not_shown(w: &mut Browser, note: String) {
    image_not_shown(w, String::new(), note);
}

#[then(expr = "the image {string} is as wide as the column, with its aspect ratio kept")]
fn image_fills_column(w: &mut Browser, alt: String) {
    let src = source_of(w, &alt);
    let page = laid_page(w);
    let laid = page.images.iter().find(|laid| laid.src == src).expect("a laid-out image");
    let reader = theme::reading(w.state().settings.font);
    let column = ((WIDTH - 2.0 * reader.gutter).min(reader.measure).max(80.0)).floor();
    assert_eq!(laid.bounds.width, column);
    let View::Shown((own_width, own_height)) = w.state().images.view(&src) else { panic!("{src} isn't shown") };
    let expected = (own_height as f32 * column / own_width as f32).round();
    assert_eq!(laid.bounds.height, expected);
}

#[then(expr = "the site was asked for an image at {string} with no cookie and no referrer")]
fn asked_for_image(w: &mut Browser, path: String) {
    let requests = w.server.requests();
    let request = requests.iter().find(|request| request.path == path).expect("a request");
    assert!(request.headers["accept"].starts_with("image/"), "{:?}", request.headers);
    assert!(!request.headers.contains_key("cookie") && !request.headers.contains_key("referer"));
}

#[when(expr = "I click the note under {string}")]
fn click_note(w: &mut Browser, alt: String) {
    let src = source_of(w, &alt);
    let page = laid_page(w);
    let note = page.fragments.iter().find(|fragment| fragment.action.as_deref() == Some(&src));
    assert!(note.is_some(), "{src}'s note can't be clicked");
    let location = w.state().images.location(&src).cloned().expect("a location");
    w.send(Event::LoadImage(location));
}

#[when("I select everything and copy")]
fn select_and_copy(w: &mut Browser) {
    let page = laid_page(w);
    let (first, last) = page.everything().expect("some text");
    w.copied = Some(page.selected_text(first, last));
}

#[then(expr = "the copied text is {string}")]
fn copied_text(w: &mut Browser, expected: String) {
    assert_eq!(w.copied.as_deref(), Some(unescape(&expected).as_str()));
}
