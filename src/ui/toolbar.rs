//! Back, forward, reload, Home, the address bar, and the History and Settings toggles.

use iced::widget::{row, text_input};
use iced::{Element, Fill};

use super::app::{ADDRESS_ID, Message};
use super::focus::button;
use crate::content;
use crate::session::{Event, State};

pub fn toolbar<'a>(state: &'a State, draft: &'a str) -> Element<'a, Message> {
    let nav = &state.nav;
    let session = |event: Event| Message::Session(event);
    row![
        button(content::BACK, nav.can_go_back().then(|| session(Event::Back))),
        button(content::FORWARD, nav.can_go_forward().then(|| session(Event::Forward))),
        button(content::RELOAD, nav.current().is_some().then(|| session(Event::Reload))),
        button(content::HOME, Some(session(Event::Home))),
        text_input(content::ADDRESS_PLACEHOLDER, draft)
            .id(ADDRESS_ID)
            .on_input(Message::AddressChanged)
            .on_submit(Message::Submit)
            .width(Fill),
        button(content::HISTORY, Some(Message::ToggleHistory)),
        button(content::SETTINGS, Some(Message::ToggleSettings)),
    ]
    .spacing(8)
    .padding(8)
    .into()
}
