//! Acceptance suite: runs the Gherkin scenarios in features/ against the session.

mod steps;
mod world;

use cucumber::World as _;

fn main() {
    let features = concat!(env!("CARGO_MANIFEST_DIR"), "/features");
    futures::executor::block_on(world::Browser::cucumber().fail_on_skipped().run_and_exit(features));
}
