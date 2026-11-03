//! Steps for features/about.feature.

use cucumber::then;

use crate::world::Browser;

#[then(expr = "the address bar shows {string}")]
fn address_bar_shows(w: &mut Browser, address: String) {
    assert_eq!(w.state().address(), address);
}
