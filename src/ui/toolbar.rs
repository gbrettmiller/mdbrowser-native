//! Back, forward, reload, the address bar, the font menu and the History toggle.

use std::fmt;

use iced::widget::{button, pick_list, row, text_input};
use iced::{Element, Fill};

use super::app::{ADDRESS_ID, Message};
use crate::content;
use crate::session::{Event, State};
use crate::theme::ReadingFont;

/// A reading font as the menu shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FontOption(pub ReadingFont);

impl fmt::Display for FontOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(content::font_name(self.0))
    }
}

const FONT_OPTIONS: [FontOption; 4] = [
    FontOption(ReadingFont::Literata),
    FontOption(ReadingFont::SourceSerif),
    FontOption(ReadingFont::Quattro),
    FontOption(ReadingFont::Inter),
];

pub fn toolbar<'a>(state: &'a State, draft: &'a str) -> Element<'a, Message> {
    let nav = &state.nav;
    let session = |event: Event| Message::Session(event);
    row![
        button(content::BACK).on_press_maybe(nav.can_go_back().then(|| session(Event::Back))),
        button(content::FORWARD).on_press_maybe(nav.can_go_forward().then(|| session(Event::Forward))),
        button(content::RELOAD).on_press_maybe(nav.current().is_some().then(|| session(Event::Reload))),
        text_input(content::ADDRESS_PLACEHOLDER, draft)
            .id(ADDRESS_ID)
            .on_input(Message::AddressChanged)
            .on_submit(Message::Submit)
            .width(Fill),
        pick_list(FONT_OPTIONS, Some(FontOption(state.settings.font)), |FontOption(font)| {
            Message::Session(Event::ChooseFont(font))
        }),
        button(content::HISTORY).on_press(Message::ToggleHistory),
    ]
    .spacing(8)
    .padding(8)
    .into()
}
