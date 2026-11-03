//! Steps for features/about.feature.

use cucumber::then;
use mdbrowser::session::{Status, about::Facts};

use crate::world::Browser;

#[then(expr = "the address bar shows {string}")]
fn address_bar_shows(w: &mut Browser, address: String) {
    assert_eq!(w.state().address(), address);
}

fn about_text(w: &Browser) -> String {
    match &w.state().status {
        Status::Ready(shown) => shown.source.to_string(),
        other => panic!("expected a page, got {other:?}"),
    }
}

#[then("the page shows the version from Cargo.toml")]
fn shows_version(w: &mut Browser) {
    let version = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .unwrap()
        .lines()
        .find_map(|line| line.strip_prefix("version = ").map(|v| v.trim_matches('"').to_string()))
        .expect("a version in Cargo.toml");
    assert!(about_text(w).contains(&format!("mdBrowser {version},")));
}

#[then("the page shows this platform")]
fn shows_platform(w: &mut Browser) {
    assert!(about_text(w).contains(&Facts::this_build().platform));
}
