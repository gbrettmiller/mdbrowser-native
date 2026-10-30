//! The History side panel: newest first, click to reopen, one button to clear.

use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Fill, Length};

use super::app::Message;
use super::focus::pressable;
use crate::content;
use crate::session::{Event, State};
use crate::theme::{self, chrome_text};

const WIDTH: f32 = 300.0;

pub fn panel(state: &State, palette: theme::Palette) -> Element<'_, Message> {
    let reader = theme::reading(theme::Look { font: state.settings.font, palette, ..theme::Look::default() });
    let muted = reader.muted;
    let entries: Element<'_, Message> = if state.visits.is_empty() {
        text(content::HISTORY_EMPTY).size(chrome_text::LABEL).into()
    } else {
        column(state.visits.iter().map(|visit| {
            let label = column![
                text(visit.title.as_str()).size(chrome_text::BODY),
                text(visit.address.as_str()).size(chrome_text::NOTE).style(move |_| text::Style { color: Some(muted) }),
            ]
            .spacing(2);
            let open = Message::Session(Event::OpenHistory(visit.address.clone()));
            pressable(button(label).width(Fill).style(button::text).on_press(open.clone()), Some(open))
        }))
        .spacing(2)
        .into()
    };
    let clearing = (!state.visits.is_empty()).then_some(Message::Session(Event::ClearHistory));
    let clear =
        pressable(button(content::CLEAR_HISTORY).style(button::secondary).on_press_maybe(clearing.clone()), clearing);
    container(
        column![text(content::HISTORY).size(chrome_text::HEADING), clear, scrollable(entries).height(Fill)].spacing(12),
    )
    .width(Length::Fixed(WIDTH))
    .height(Fill)
    .padding(12)
    .into()
}
