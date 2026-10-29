//! Back, forward, reload, the address bar, the font menu and the History toggle.

use std::fmt;

use iced::widget::{button, column, pick_list, row, text_input};
use iced::{Element, Fill};

use super::app::{ADDRESS_ID, Message};
use crate::content;
use crate::session::images::ImageLoading;
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

/// A web-image mode as the menu shows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImagesOption(pub ImageLoading);

impl fmt::Display for ImagesOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(content::image_loading(self.0))
    }
}

const FONT_OPTIONS: [FontOption; 4] = [
    FontOption(ReadingFont::Literata),
    FontOption(ReadingFont::SourceSerif),
    FontOption(ReadingFont::Quattro),
    FontOption(ReadingFont::Inter),
];

/// Narrower than this, the menus move to a second row so the address bar and
/// History always keep their place.
const ONE_ROW_MIN_WIDTH: f32 = 900.0;

pub fn toolbar<'a>(state: &'a State, draft: &'a str, window_width: f32) -> Element<'a, Message> {
    let nav = &state.nav;
    let session = |event: Event| Message::Session(event);
    let navigation = row![
        button(content::BACK).on_press_maybe(nav.can_go_back().then(|| session(Event::Back))),
        button(content::FORWARD).on_press_maybe(nav.can_go_forward().then(|| session(Event::Forward))),
        button(content::RELOAD).on_press_maybe(nav.current().is_some().then(|| session(Event::Reload))),
        text_input(content::ADDRESS_PLACEHOLDER, draft)
            .id(ADDRESS_ID)
            .on_input(Message::AddressChanged)
            .on_submit(Message::Submit)
            .width(Fill),
    ]
    .spacing(8);
    let menus = row![
        pick_list(FONT_OPTIONS, Some(FontOption(state.settings.font)), |FontOption(font)| {
            Message::Session(Event::ChooseFont(font))
        }),
        pick_list(
            ImageLoading::ALL.map(ImagesOption),
            Some(ImagesOption(state.settings.images)),
            |ImagesOption(mode)| { Message::Session(Event::ChooseImageLoading(mode)) }
        ),
    ]
    .spacing(8);
    let history = button(content::HISTORY).on_press(Message::ToggleHistory);
    if window_width >= ONE_ROW_MIN_WIDTH {
        row![navigation, menus, history].spacing(8).padding(8).into()
    } else {
        column![row![navigation, history].spacing(8), menus].spacing(8).padding(8).into()
    }
}
