//! Steps for images: features/images.feature.

use cucumber::{given, then, when};
use mdbrowser::content;
use mdbrowser::model::Block;
use mdbrowser::session::images::ImageLoading;
use mdbrowser::session::{Event, Status};

use crate::world::Browser;

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

/// Until images are drawn (phase 2b, step 8), "shown" means the document has an image block.
#[then("no image is shown")]
fn no_image_shown(w: &mut Browser) {
    let Status::Ready(shown) = &w.state().status else { panic!("no document: {:?}", w.state().status) };
    assert!(image_blocks(&shown.document.blocks).is_empty(), "{:?}", shown.document.blocks);
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
