//! The History side panel.

use iced::widget::{column, text};
use iced::{Element, Length};

use super::app::Message;
use crate::session::State;

pub fn panel(_state: &State) -> Element<'_, Message> {
    column![text("")].width(Length::Fixed(0.0)).into()
}
