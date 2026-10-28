//! Walking skeleton: open the file named on the command line and render it.

use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use iced::widget::{column, container, responsive, scrollable, text};
use iced::{Element, Fill, Task};

use super::document::Document;
use crate::layout::{self, Page};
use crate::model::{self, Document as Doc};
use crate::services::files;
use crate::theme::{self, ReadingFont, fonts};

const SCROLLBAR: f32 = 12.0;

#[derive(Debug, Clone)]
pub enum Message {
    Follow(String),
    Hover(Option<String>),
}

struct App {
    path: PathBuf,
    document: Result<Doc, String>,
    theme: theme::Theme,
    /// Layout is a function of (document, theme, width); keep the last one.
    page: RefCell<Option<(f32, Arc<Page>)>>,
    status: String,
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let path = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "corpus/index.md".into()));
        let font = std::env::var("MDBROWSER_FONT").ok().and_then(|key| ReadingFont::from_key(&key)).unwrap_or_default();
        let document =
            files::read(&path).map(|source| model::parse(&source)).map_err(|e| format!("{}: {e}", path.display()));
        let app = App { path, document, theme: theme::reading(font), page: RefCell::new(None), status: String::new() };
        (app, Task::none())
    }

    fn title(&self) -> String {
        let name = self.path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        self.document.as_ref().map_or(name.clone(), |doc| doc.title(&name))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Follow(href) => self.status = format!("Following links arrives with the session: {href}"),
            Message::Hover(href) => self.status = href.unwrap_or_default(),
        }
        Task::none()
    }

    fn page_for(&self, doc: &Doc, width: f32) -> Arc<Page> {
        let mut cached = self.page.borrow_mut();
        if let Some((_, page)) = cached.as_ref().filter(|(w, _)| (w - width).abs() < 0.5) {
            return page.clone();
        }
        let started = Instant::now();
        let page = Arc::new(layout::layout(&doc.blocks, &self.theme, width));
        eprintln!("layout @{width}px: {} fragments in {:?}", page.fragments.len(), started.elapsed());
        *cached = Some((width, page.clone()));
        page
    }

    fn view(&self) -> Element<'_, Message> {
        let body: Element<'_, Message> = match &self.document {
            Err(error) => container(text(error.clone())).padding(32).into(),
            Ok(doc) => responsive(move |size| {
                let page = self.page_for(doc, size.width - SCROLLBAR);
                scrollable(Document::new(page, self.theme, Message::Follow, Message::Hover)).into()
            })
            .into(),
        };
        let background = self.theme.background;
        let page = container(body)
            .width(Fill)
            .height(Fill)
            .style(move |_| container::Style { background: Some(background.into()), ..container::Style::default() });
        column![page, container(text(self.status.clone()).size(12)).padding([2, 8])].into()
    }
}

pub fn run() -> iced::Result {
    fonts::install();
    iced::application(App::boot, App::update, App::view).title(App::title).window_size((960.0, 1000.0)).run()
}
