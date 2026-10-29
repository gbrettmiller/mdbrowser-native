//! Acceptance suite: runs the Gherkin scenarios in features/ against the session.

mod fixtures;
#[path = "../support/http.rs"]
mod http;
mod steps;
mod world;

use cucumber::World as _;

fn main() {
    // The guard every load passes through: check it before trusting it.
    let outside = url::Url::parse("https://example.com/a.md").unwrap();
    assert!(!world::on_test_server(&outside) && world::on_test_server(&"http://127.0.0.1:1/a".parse().unwrap()));
    let features = concat!(env!("CARGO_MANIFEST_DIR"), "/features");
    futures::executor::block_on(world::Browser::cucumber().fail_on_skipped().run_and_exit(features));
}
