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
