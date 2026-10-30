//! Step definitions. Steps assert on session state, recorded effects and the
//! requests the test server saw, never on pixels.

mod fallbacks;
mod frontmatter;
mod images;
mod local;
mod reader;
mod remote;
mod settings;

use mdbrowser::session::{Event, Status};

use crate::world::Browser;

fn unescape(text: &str) -> String {
    text.replace("\\n", "\n")
}

fn enter(w: &mut Browser, input: String) {
    w.send(Event::Enter(input));
}

fn shown_title(w: &Browser) -> Option<String> {
    match &w.state().status {
        Status::Ready(shown) => Some(shown.title.clone()),
        _ => None,
    }
}
