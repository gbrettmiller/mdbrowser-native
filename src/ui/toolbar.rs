//! Back, forward, reload, Home, the address bar, and the History and Settings toggles.

use iced::widget::{button, row, text_input};
use iced::{Element, Fill};

use super::app::{ADDRESS_ID, Message};
use crate::content;
use crate::session::{Event, State};

pub fn toolbar<'a>(state: &'a State, draft: &'a str) -> Element<'a, Message> {
    let nav = &state.nav;
    let session = |event: Event| Message::Session(event);
    row![
        button(content::BACK).on_press_maybe(nav.can_go_back().then(|| session(Event::Back))),
        button(content::FORWARD).on_press_maybe(nav.can_go_forward().then(|| session(Event::Forward))),
        button(content::RELOAD).on_press_maybe(nav.current().is_some().then(|| session(Event::Reload))),
        button(content::HOME).on_press(session(Event::Home)),
        text_input(content::ADDRESS_PLACEHOLDER, draft)
            .id(ADDRESS_ID)
            .on_input(Message::AddressChanged)
            .on_submit(Message::Submit)
            .width(Fill),
        button(content::HISTORY).on_press(Message::ToggleHistory),
        button(content::SETTINGS).on_press(Message::ToggleSettings),
    ]
    .spacing(8)
    .padding(8)
    .into()
}
