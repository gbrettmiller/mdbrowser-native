//! Everything the document area can show besides a document: welcome, loading,
//! errors, and the notices above it.

use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill};

use super::app::Message;
use crate::content;
use crate::session::error::ErrorCode;
use crate::session::{Event, Notice};
use crate::theme;

pub fn message<'a>(message: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    container(text(message).size(16)).padding(32).into()
}

/// The failure, a hint for pages on the web, the address it was for, and the
/// system browser when the session offers it.
pub fn error<'a>(
    code: ErrorCode,
    address: String,
    web: bool,
    browser_offered: bool,
    theme: &theme::Theme,
) -> Element<'a, Message> {
    let muted = theme.muted;
    let (message, hint) = content::failure(code, web);
    let open = button(content::OPEN_IN_BROWSER).on_press(Message::Session(Event::OpenInBrowser));
    column![text(message).size(20)]
        .push(hint.map(|hint| text(hint).size(16)))
        .push(text(address).size(14).style(move |_| text::Style { color: Some(muted) }))
        .push(browser_offered.then_some(open))
        .spacing(8)
        .padding(32)
        .into()
}

pub fn notices(notices: &[Notice]) -> Element<'_, Message> {
    column(notices.iter().map(|notice| {
        row![
            text(content::notice(notice.kind)).width(Fill),
            button(content::DISMISS).on_press(Message::Session(Event::DismissNotice(notice.id))),
        ]
        .spacing(8)
        .padding([4, 8])
        .into()
    }))
    .into()
}
