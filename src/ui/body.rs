//! Everything the document area can show besides a document: welcome, loading,
//! errors, and the notices above it.

use iced::widget::{button, column, container, row, text};
use iced::{Element, Fill};

use super::app::Message;
use crate::content;
use crate::session::error::ErrorCode;
use crate::session::location::Location;
use crate::session::{Event, Notice, State, Status};
use crate::theme;

pub fn message<'a>(message: impl text::IntoFragment<'a>) -> Element<'a, Message> {
    container(text(message).size(16)).padding(32).into()
}

/// The failure, a hint for pages on the web, the address it was for, and the
/// system browser when the session offers it.
pub fn error<'a>(code: ErrorCode, state: &State, theme: &theme::Theme) -> Element<'a, Message> {
    let muted = theme.muted;
    let (message, hint) = content::failure(code, state.current_location().is_some_and(Location::is_web));
    let (address, browser_offered) = (state.address(), state.browser_offer().is_some());
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

/// The document's title, for windows that have no title bar to show it: only
/// when a document is on screen.
pub fn title_line(status: &Status) -> Option<&str> {
    match status {
        Status::Ready(shown) => Some(shown.title.as_str()),
        _ => None,
    }
}

pub fn title_bar<'a>(title: &'a str, theme: &theme::Theme) -> Element<'a, Message> {
    let muted = theme.muted;
    container(text(title).size(13).style(move |_| text::Style { color: Some(muted) }).wrapping(text::Wrapping::None))
        .padding([2, 8])
        .width(Fill)
        .center_x(Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::model::parse;
    use crate::session::Shown;

    #[test]
    fn the_title_shows_only_with_a_document() {
        let shown = Shown { document: Arc::new(parse("# Hi")), title: "Hi".into() };
        assert_eq!(title_line(&Status::Ready(Arc::new(shown))), Some("Hi"));
        for status in [Status::Idle, Status::Loading, Status::Failed(ErrorCode::NotFound)] {
            assert_eq!(title_line(&status), None);
        }
    }
}
