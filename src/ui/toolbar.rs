//! Back, forward, reload, the source toggle, Home, the address bar, the bookmark star, and the
//! Bookmarks, History and Settings toggles.

use iced::widget::{row, text, text_input, tooltip};
use iced::{Element, Fill};

use super::app::{ADDRESS_ID, Message};
use super::focus::button;
use crate::content;
use crate::session::{Event, State, Status, View};

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

/// The source button's label, and whether it can be pressed: only with a page shown.
pub fn source_button(state: &State) -> (&'static str, bool) {
    let label = if matches!(state.view, View::Source(_)) { content::PAGE } else { content::SOURCE };
    (label, matches!(state.status, Status::Ready(_)))
}

/// The star's label, and whether it can be pressed: only on a page that can be bookmarked.
pub fn bookmark_button(state: &State) -> (&'static str, bool) {
    let label = if state.is_bookmarked() { content::BOOKMARKED } else { content::BOOKMARK };
    (label, state.bookmarkable().is_some())
}

fn stop<'a>(state: &'a State, draft: &'a str, at: Stop) -> Element<'a, Message> {
    let nav = &state.nav;
    let session = |event: Event| Message::Session(event);
    match at {
        Stop::Back => button(content::BACK, nav.can_go_back().then(|| session(Event::Back))),
        Stop::Forward => button(content::FORWARD, nav.can_go_forward().then(|| session(Event::Forward))),
        Stop::Reload => button(content::RELOAD, nav.current().is_some().then(|| session(Event::Reload))),
        Stop::Source => {
            let (label, enabled) = source_button(state);
            let tip = content::source_tooltip(cfg!(target_os = "macos"));
            tooltip(button(label, enabled.then(|| session(Event::ToggleSource))), text(tip), tooltip::Position::Bottom)
                .into()
        }
        Stop::Home => button(content::HOME, Some(session(Event::Home))),
        Stop::Address => text_input(content::ADDRESS_PLACEHOLDER, draft)
            .id(ADDRESS_ID)
            .on_input(Message::AddressChanged)
            .on_submit(Message::Submit)
            .width(Fill)
            .into(),
        Stop::Bookmark => {
            let (label, enabled) = bookmark_button(state);
            let tip = content::bookmark_tooltip(state.is_bookmarked(), cfg!(target_os = "macos"));
            tooltip(
                button(label, enabled.then(|| session(Event::ToggleBookmark))),
                text(tip),
                tooltip::Position::Bottom,
            )
            .into()
        }
        Stop::Bookmarks => button(content::BOOKMARKS, Some(Message::ToggleBookmarks)),
        Stop::History => button(content::HISTORY, Some(Message::ToggleHistory)),
        Stop::Settings => button(content::SETTINGS, Some(Message::ToggleSettings)),
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
    fn the_star_follows_the_address_bar_and_shows_whether_the_page_is_kept() {
        let at = |stop| ORDER.iter().position(|&s| s == stop).expect("in the toolbar");
        assert_eq!(at(Stop::Bookmark), at(Stop::Address) + 1);
        let page = ready();
        assert_eq!(bookmark_button(&page), (content::BOOKMARK, true));
        let (kept, _) = step(page, Event::ToggleBookmark);
        assert_eq!(bookmark_button(&kept), (content::BOOKMARKED, true));
        assert_eq!(bookmark_button(&State::new(native("/home/r"))), (content::BOOKMARK, false), "nothing shown");
    }

    #[test]
    fn the_source_button_names_the_other_view_and_needs_a_page() {
        let page = ready();
        assert_eq!(source_button(&page), (content::SOURCE, true));
        let (source, _) = step(page, Event::ToggleSource);
        assert_eq!(source_button(&source), (content::PAGE, true));
        assert_eq!(source_button(&State::new(native("/home/r"))), (content::SOURCE, false), "nothing shown");
        let (loading, _) = step(State::new(native("/home/r")), Event::Enter(native("/d/a.md")));
        assert_eq!(source_button(&loading), (content::SOURCE, false));
    }
}
