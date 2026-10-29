//! Steps for images: features/images.feature.

use cucumber::then;
use mdbrowser::model::Block;
use mdbrowser::session::Status;

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
