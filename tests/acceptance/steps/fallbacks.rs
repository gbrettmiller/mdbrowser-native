//! Steps for features/fallbacks.feature.

use cucumber::{then, when};

use mdbrowser::session::{Event, Status};

use crate::world::Browser;

#[then(expr = "the page says it was found instead of the URL of {string}")]
fn found_instead(w: &mut Browser, path: String) {
    let original = w.url(&path);
    let Status::Ready(shown) = &w.state().status else { panic!("{:?}", w.state().status) };
    assert_eq!(shown.fallback_from.as_deref(), Some(original.as_str()));
}

#[then("the page does not say it was found instead")]
fn not_found_instead(w: &mut Browser) {
    let Status::Ready(shown) = &w.state().status else { panic!("{:?}", w.state().status) };
    assert_eq!(shown.fallback_from, None);
}

#[then(expr = "the site was not asked for {string}")]
fn not_asked(w: &mut Browser, path: String) {
    assert!(w.server.requests().iter().all(|request| request.path != path), "{path} was requested");
}

#[when("I choose the site's llms.txt")]
fn choose_llms(w: &mut Browser) {
    let offer = w.state().llms_offer().expect("llms.txt offered");
    w.send(Event::Enter(offer));
}

#[then("I am not offered the site's llms.txt")]
fn no_llms_offer(w: &mut Browser) {
    assert_eq!(w.state().llms_offer(), None);
}
