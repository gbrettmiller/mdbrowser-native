//! Back, forward, reload, the source toggle, Home, the address bar, the bookmark star, and the
//! Bookmarks, History and Settings toggles, all as icons but the address bar (ADR-016).

use iced::advanced::widget::Id;
use iced::widget::{row, text_input};
use iced::{Element, Fill};

use super::app::{ADDRESS_ID, Message};
use super::icon;
use crate::content;
use crate::session::{Event, State, Status, View};
use crate::theme::icons::Icon;

/// The toolbar's stops, in the order Tab visits them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stop {
    Back,
    Forward,
    Reload,
    Source,
    Home,
    Address,
    Bookmark,
    Bookmarks,
    History,
    Settings,
}

const ORDER: [Stop; 10] = [
    Stop::Back,
    Stop::Forward,
    Stop::Reload,
    Stop::Source,
    Stop::Home,
    Stop::Address,
    Stop::Bookmark,
    Stop::Bookmarks,
    Stop::History,
    Stop::Settings,
];

/// The source toggle's icon, naming the view it switches to, and whether it can be pressed:
/// only with a page shown.
pub fn source_button(state: &State) -> (Icon, bool) {
    let icon = if matches!(state.view, View::Source(_)) { Icon::Page } else { Icon::Source };
    (icon, matches!(state.status, Status::Ready(_)))
}

/// The star, filled when the page is kept, and whether it can be pressed: only on a page that
/// can be bookmarked.
pub fn bookmark_button(state: &State) -> (Icon, bool) {
    let icon = if state.is_bookmarked() { Icon::Bookmarked } else { Icon::Bookmark };
    (icon, state.bookmarkable().is_some())
}

/// The source button, so a keyboard press can keep the keyboard on it.
pub const SOURCE_ID: &str = "source";

/// What a click and a key press on the source button send. Both switch the view; the key
/// press also keeps the keyboard on the button, where a click (or Ctrl+U) puts it on the page.
fn source_messages(enabled: bool) -> (Option<Message>, Option<Message>) {
    (enabled.then_some(Message::Session(Event::ToggleSource)), enabled.then_some(Message::ToggleSourceByKey))
}

fn stop<'a>(state: &'a State, draft: &'a str, at: Stop) -> Element<'a, Message> {
    let nav = &state.nav;
    let session = |event: Event| Message::Session(event);
    match at {
        Stop::Back => icon::button(Icon::Back, nav.can_go_back().then(|| session(Event::Back))),
        Stop::Forward => icon::button(Icon::Forward, nav.can_go_forward().then(|| session(Event::Forward))),
        Stop::Reload => icon::button(Icon::Reload, nav.current().is_some().then(|| session(Event::Reload))),
        Stop::Source => {
            let (view, enabled) = source_button(state);
            let (click, key) = source_messages(enabled);
            icon::button_with_id(view, click, key, Id::new(SOURCE_ID))
        }
        Stop::Home => icon::button(Icon::Home, Some(session(Event::Home))),
        Stop::Address => text_input(content::ADDRESS_PLACEHOLDER, draft)
            .id(ADDRESS_ID)
            .on_input(Message::AddressChanged)
            .on_submit(Message::Submit)
            .width(Fill)
            .into(),
        Stop::Bookmark => {
            let (star, enabled) = bookmark_button(state);
            icon::button(star, enabled.then(|| session(Event::ToggleBookmark)))
        }
        Stop::Bookmarks => icon::button(Icon::Bookmarks, Some(Message::ToggleBookmarks)),
        Stop::History => icon::button(Icon::History, Some(Message::ToggleHistory)),
        Stop::Settings => icon::button(Icon::Settings, Some(Message::ToggleSettings)),
    }
}

pub fn toolbar<'a>(state: &'a State, draft: &'a str) -> Element<'a, Message> {
    row(ORDER.map(|at| stop(state, draft, at))).spacing(8).padding(8).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::address::native;
    use crate::session::location::Location;
    use crate::session::{Loaded, State, step};

    fn ready() -> State {
        let state = State::new(native("/home/r"));
        let (state, _) = step(state, Event::Enter(native("/d/a.md")));
        let loaded = Loaded { source: "# A".into(), location: Location::file(native("/d/a.md")), fallback_from: None };
        step(state, Event::Loaded { request_id: 1, result: Ok(loaded), now: 1 }).0
    }

    #[test]
    fn the_source_button_follows_reload_in_tab_order() {
        let at = |stop| ORDER.iter().position(|&s| s == stop).expect("in the toolbar");
        assert_eq!(at(Stop::Source), at(Stop::Reload) + 1);
        assert_eq!(ORDER.len(), 10, "every stop appears once");
    }

    #[test]
    fn a_key_press_on_the_source_button_keeps_the_keyboard_there_and_a_click_does_not() {
        let (click, key) = source_messages(true);
        assert!(matches!(click, Some(Message::Session(Event::ToggleSource))));
        assert!(matches!(key, Some(Message::ToggleSourceByKey)));
        assert!(matches!(source_messages(false), (None, None)), "no page, no stop");
    }

    #[test]
    fn the_star_follows_the_address_bar_and_shows_whether_the_page_is_kept() {
        let at = |stop| ORDER.iter().position(|&s| s == stop).expect("in the toolbar");
        assert_eq!(at(Stop::Bookmark), at(Stop::Address) + 1);
        let page = ready();
        assert_eq!(bookmark_button(&page), (Icon::Bookmark, true));
        let (kept, _) = step(page, Event::ToggleBookmark);
        assert_eq!(bookmark_button(&kept), (Icon::Bookmarked, true));
        assert_eq!(bookmark_button(&State::new(native("/home/r"))), (Icon::Bookmark, false), "nothing shown");
    }

    #[test]
    fn the_source_button_names_the_other_view_and_needs_a_page() {
        let page = ready();
        assert_eq!(source_button(&page), (Icon::Source, true));
        let (source, _) = step(page, Event::ToggleSource);
        assert_eq!(source_button(&source), (Icon::Page, true));
        assert_eq!(source_button(&State::new(native("/home/r"))), (Icon::Source, false), "nothing shown");
        let (loading, _) = step(State::new(native("/home/r")), Event::Enter(native("/d/a.md")));
        assert_eq!(source_button(&loading), (Icon::Source, false));
    }
}
