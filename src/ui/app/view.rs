//! The window's contents: toolbar, notices, the page or a message, the History
//! panel, and the status line.

use std::sync::Arc;

use iced::widget::{column, container, responsive, row, scrollable, text};
use iced::{Element, Fill};

use super::{App, Message, SCROLL_ID, SCROLLBAR};
use crate::content;
use crate::session::location::Location;
use crate::session::{Event, Shown, Status};
use crate::theme;
use crate::ui::document::{Document, Handlers};
use crate::ui::{body, history, images, toolbar};

impl App {
    pub(super) fn title(&self) -> String {
        match &self.session().status {
            Status::Ready(shown) => format!("{} — {}", shown.title, content::APP_NAME),
            _ => content::APP_NAME.into(),
        }
    }

    fn document_view(&self, shown: Arc<Shown>) -> Element<'_, Message> {
        let font = self.session().settings.font;
        responsive(move |size| {
            let width = size.width - SCROLLBAR;
            let (page, settling) = self.page(&shown.document, width);
            self.viewport.laid_out(&shown.document, width, settling);
            let pictures = images::pictures(&page, &self.session().images);
            let on = Handlers {
                follow: Box::new(|href| Message::Session(Event::Follow(href))),
                load_image: Box::new(|src| Message::Session(Event::LoadImage(src))),
                hover: Box::new(Message::Hover),
            };
            let document = Document::new(page, pictures, theme::reading(font), on);
            scrollable(document)
                .id(SCROLL_ID)
                .on_scroll(|viewport| Message::Scrolled(viewport.absolute_offset().y))
                .into()
        })
        .into()
    }

    pub(super) fn view(&self) -> Element<'_, Message> {
        let state = self.session();
        let reader = theme::reading(state.settings.font);
        let main: Element<'_, Message> = match &state.status {
            Status::Idle => body::message(content::WELCOME),
            Status::Loading => body::message(content::loading(state.current_location().and_then(Location::host))),
            Status::Failed(code) => body::error(*code, state, &reader),
            Status::Ready(shown) => self.document_view(shown.clone()),
        };
        let background = reader.background;
        let page = container(main)
            .width(Fill)
            .height(Fill)
            .style(move |_| container::Style { background: Some(background.into()), ..container::Style::default() });
        let content: Element<'_, Message> =
            if self.history_open { row![page, history::panel(state)].into() } else { page.into() };
        let status = self.hover.clone().unwrap_or_default();
        column![
            toolbar::toolbar(state, &self.draft, self.window_width),
            body::notices(&state.notices),
            content,
            container(text(status).size(12)).padding([2, 8]),
        ]
        .into()
    }
}
