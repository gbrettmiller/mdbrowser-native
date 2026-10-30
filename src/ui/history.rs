//! The History side panel: newest first, click to reopen, one button to clear.

use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Fill, Length};

use super::app::Message;
use crate::content;
use crate::session::{Event, State};
use crate::theme;

const WIDTH: f32 = 300.0;

pub fn panel(state: &State, palette: theme::Palette) -> Element<'_, Message> {
    let reader = theme::reading(theme::Look { font: state.settings.font, palette, ..theme::Look::default() });
    let muted = reader.muted;
    let entries: Element<'_, Message> = if state.visits.is_empty() {
        text(content::HISTORY_EMPTY).size(14).into()
    } else {
        column(state.visits.iter().map(|visit| {
            let label = column![
                text(visit.title.as_str()).size(15),
                text(visit.address.as_str()).size(12).style(move |_| text::Style { color: Some(muted) }),
            ]
            .spacing(2);
            button(label)
                .width(Fill)
                .style(button::text)
                .on_press(Message::Session(Event::OpenHistory(visit.address.clone())))
                .into()
        }))
        .spacing(2)
        .into()
    };
    let clear = button(content::CLEAR_HISTORY)
        .style(button::secondary)
        .on_press_maybe((!state.visits.is_empty()).then_some(Message::Session(Event::ClearHistory)));
    container(column![text(content::HISTORY).size(18), clear, scrollable(entries).height(Fill)].spacing(12))
        .width(Length::Fixed(WIDTH))
        .height(Fill)
        .padding(12)
        .into()
}
