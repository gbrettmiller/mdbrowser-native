//! Golden layout snapshots over the corpus. Documents whose glyphs fall back to
//! host fonts (mixed scripts in typography.md, symbols in spec.md) are only
//! smoke-tested, since fallback faces differ between machines.

use super::Page;
use super::tests::page_at;
use crate::theme::ReadingFont;

const WIDTH: f32 = 720.0;

fn geometry(page: &Page) -> String {
    let header = format!("height {:.1}, {} fragments, anchors {:?}\n", page.height, page.fragments.len(), page.anchors);
    page.fragments.iter().fold(header, |out, f| {
        out + &format!("{:7.1} {:6.1} {:6.1} {:?}\n", f.bounds.y, f.bounds.x, f.bounds.width, f.text)
    })
}

fn summary(page: &Page) -> String {
    format!("height {:.1}, {} fragments, {} anchors", page.height, page.fragments.len(), page.anchors.len())
}

#[test]
fn index() {
    insta::assert_snapshot!(geometry(&page_at(include_str!("../../corpus/index.md"), ReadingFont::Literata, WIDTH)));
}

#[test]
fn lists() {
    insta::assert_snapshot!(geometry(&page_at(include_str!("../../corpus/lists.md"), ReadingFont::Literata, WIDTH)));
}

#[test]
fn tables_and_code() {
    let source = include_str!("../../corpus/tables-and-code.md");
    insta::assert_snapshot!(geometry(&page_at(source, ReadingFont::Literata, WIDTH)));
}

#[test]
fn images_as_placeholders() {
    insta::assert_snapshot!(geometry(&page_at(include_str!("../../corpus/images.md"), ReadingFont::Literata, WIDTH)));
}

/// Every corpus file with images lays out with images both shown and missing,
/// and every Markdown image in it becomes an image block.
#[test]
fn every_corpus_image_becomes_a_block_and_lays_out() {
    use crate::layout::{ImageNote, ImageView, layout};
    use crate::model::parse;
    use crate::theme::{Palette, reading};
    let mut files = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus")];
    let mut checked = 0;
    while let Some(path) = files.pop() {
        if path.is_dir() {
            files.extend(std::fs::read_dir(&path).unwrap().map(|entry| entry.unwrap().path()));
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        if path.extension().is_none_or(|ext| ext != "md") || !source.contains("![") {
            continue;
        }
        let document = parse(&source);
        let images = document.image_sources().len();
        let shown = |_: &str| ImageView::Shown { width: 640, height: 480 };
        let placeholder = |_: &str| ImageView::Placeholder {
            note: Some(ImageNote { text: "Loading image…".into(), clickable: false }),
        };
        for view in [&shown as &dyn Fn(&str) -> ImageView, &placeholder] {
            let page = layout(&document.blocks, &reading(ReadingFont::Literata, Palette::Light), WIDTH, view);
            assert!(page.height > 0.0, "{path:?}");
        }
        let page = layout(&document.blocks, &reading(ReadingFont::Literata, Palette::Light), WIDTH, &shown);
        assert_eq!(page.images.len(), images, "{path:?}");
        checked += 1;
    }
    assert!(checked >= 5, "checked {checked} files");
}

#[test]
fn long() {
    insta::assert_snapshot!(summary(&page_at(include_str!("../../corpus/long.md"), ReadingFont::Literata, WIDTH)));
}

#[test]
fn every_corpus_document_lays_out_in_every_font() {
    let corpus = [include_str!("../../corpus/typography.md"), include_str!("../../corpus/spec.md")];
    for font in ReadingFont::ALL {
        for source in corpus {
            let page = page_at(source, font, WIDTH);
            assert!(page.height > 0.0 && !page.fragments.is_empty() && !page.texts.is_empty());
        }
    }
}

#[test]
#[ignore = "timing report: cargo test --release -- --ignored --nocapture"]
fn spec_timing() {
    let source = include_str!("../../corpus/spec.md");
    page_at("warm up", ReadingFont::Literata, WIDTH);
    for width in [720.0, 719.0, 540.0] {
        let started = std::time::Instant::now();
        let page = page_at(source, ReadingFont::Literata, width);
        println!("spec.md @{width}: {} fragments in {:?}", page.fragments.len(), started.elapsed());
    }
}

fn framed_page(source: &str) -> Page {
    use crate::layout::{layout, no_images};
    use crate::model::{Labels, assemble, parse};
    use crate::theme::{Palette, reading};
    let labels = Labels { listen: "Listen:", previous: "← Previous", next: "Next →" };
    let document = parse(source);
    layout(&assemble(&document, &labels), &reading(ReadingFont::Literata, Palette::Light), WIDTH, &no_images)
}

/// Example 1 of the markdownWeb examples: a byline under no heading, and a previous/next row.
#[test]
fn a_post_with_front_matter() {
    let source = include_str!("../../corpus/markdownweb/2024/03/05/200550.md");
    insta::assert_snapshot!(geometry(&framed_page(source)));
}

/// Every markdownWeb post lays out with its byline and links, and shows them.
#[test]
fn every_markdownweb_post_shows_its_front_matter() {
    let mut files = vec![std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/markdownweb")];
    let mut checked = 0;
    while let Some(path) = files.pop() {
        if path.is_dir() {
            files.extend(std::fs::read_dir(&path).unwrap().map(|entry| entry.unwrap().path()));
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        if path.extension().is_none_or(|ext| ext != "md") || !source.starts_with("---\n") {
            continue;
        }
        let page = framed_page(&source);
        let text: String = page.fragments.iter().map(|f| f.text.as_str()).collect::<Vec<_>>().join("\n");
        assert!(page.height > 0.0 && text.contains("Dave Winer"), "{path:?}");
        assert!(source.contains("urlPrev") == text.contains("← Previous"), "{path:?}");
        assert!(source.contains("urlNext") == text.contains("Next →"), "{path:?}");
        checked += 1;
    }
    assert!(checked >= 9, "checked {checked} files");
}
