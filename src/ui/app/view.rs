//! The window's contents: toolbar, notices, the page or a message, the History
//! panel, and the status line.

use std::sync::Arc;

use iced::widget::{column, container, responsive, row, scrollable, text};
use iced::{Element, Fill};

use super::{App, Message, PAGE_ID, SCROLL_ID, SCROLLBAR};
use crate::content;
use crate::session::location::Location;
use crate::session::{Event, Shown, Status};
use crate::theme::{self, chrome_text};
use crate::ui::document::{Document, Handlers};
use crate::ui::{body, history, images, settings, toolbar};

impl App {
    pub(super) fn title(&self) -> String {
        match &self.session().status {
            Status::Ready(shown) => format!("{} — {}", shown.title, content::APP_NAME),
            _ => content::APP_NAME.into(),
        }
    }

    fn document_view(&self, shown: Arc<Shown>) -> Element<'_, Message> {
        let reader = theme::reading(self.look());
        responsive(move |size| {
            let width = size.width - SCROLLBAR;
            self.viewport.sized(size.height);
            let (page, settling) = self.page(&shown.document, width);
            self.viewport.laid_out(&shown.document, width, settling);
            let pictures = images::pictures(&page, &self.session().images);
            let on = Handlers {
                follow: Box::new(|href| Message::Session(Event::Follow(href))),
                load_image: Box::new(|src| Message::Session(Event::LoadImage(src))),
                hover: Box::new(Message::Hover),
                reveal: Box::new(Message::Reveal),
                scroll: Box::new(Message::Scroll),
            };
            let document = Document::new(Arc::as_ptr(&shown.document) as usize, page, pictures, reader, on).id(PAGE_ID);
            scrollable(document)
                .id(SCROLL_ID)
                .on_scroll(|viewport| Message::Scrolled(viewport.absolute_offset().y))
                .into()
        })
        .into()
    }

    pub(super) fn view(&self) -> Element<'_, Message> {
        let state = self.session();
        let reader = theme::reading(self.look());
        let main: Element<'_, Message> = match &state.status {
            Status::Idle => body::message(""),
            Status::Loading => body::message(content::loading(state.current_location().and_then(Location::host))),
            Status::Failed(code) => body::error(*code, state, &reader),
            Status::Ready(shown) => self.document_view(shown.clone()),
        };
        let background = reader.background;
        let page = container(main)
            .width(Fill)
            .height(Fill)
            .style(move |_| container::Style { background: Some(background.into()), ..container::Style::default() });
        let panels = row![page]
            .push(self.panels.history.then(|| history::panel(state, self.palette())))
            .push(self.panels.settings.then(|| settings::panel(state, &self.home_draft, &self.colour_drafts)));
        let content: Element<'_, Message> = panels.into();
        let status = self.hover.clone().unwrap_or_default();
        column![toolbar::toolbar(state, &self.draft)]
            .push(body::title_line(&state.status).map(|title| body::title_bar(title, &reader)))
            .push(body::fallback_banner(&state.status))
            .push(body::notices(&state.notices))
            .push(content)
            .push(container(text(status).size(chrome_text::NOTE)).padding([2, 8]))
            .into()
    }
}
