//! The Settings side panel: font, web images, appearance and the home page.
//! Choices apply at once and are saved by the session.

use std::fmt;

use iced::widget::{column, container, pick_list, text, text_input};
use iced::{Element, Length};

use super::app::Message;
use crate::content;
use crate::session::images::ImageLoading;
use crate::session::{Event, State};
use crate::theme::{Appearance, ReadingFont};

const WIDTH: f32 = 300.0;

/// A choice as its pick list shows it.
#[derive(Clone, Copy)]
struct Shown<T>(T, fn(T) -> &'static str);

impl<T: PartialEq> PartialEq for Shown<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Copy> fmt::Display for Shown<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((self.1)(self.0))
    }
}

fn choice<'a, T: Copy + PartialEq + 'static>(
    label: &'static str,
    all: &[T],
    current: T,
    name: fn(T) -> &'static str,
    event: fn(T) -> Event,
) -> Element<'a, Message> {
    let options: Vec<_> = all.iter().map(|&value| Shown(value, name)).collect();
    column![
        text(label).size(14),
        pick_list(options, Some(Shown(current, name)), move |Shown(value, _)| Message::Session(event(value)))
            .width(Length::Fill),
    ]
    .spacing(4)
    .into()
}

pub fn panel<'a>(state: &'a State, home_draft: &'a str) -> Element<'a, Message> {
    let settings = &state.settings;
    let home = column![
        text(content::HOME_PAGE_LABEL).size(14),
        text_input(content::HOME_PAGE_PLACEHOLDER, home_draft)
            .on_input(Message::HomeDraftChanged)
            .on_submit(Message::SubmitHome),
    ]
    .spacing(4);
    container(
        column![
            text(content::SETTINGS).size(18),
            choice(content::FONT_LABEL, &ReadingFont::ALL, settings.font, content::font_name, Event::ChooseFont),
            choice(
                content::IMAGES_LABEL,
                &ImageLoading::ALL,
                settings.images,
                content::image_loading,
                Event::ChooseImageLoading
            ),
            choice(
                content::APPEARANCE_LABEL,
                &Appearance::ALL,
                settings.appearance,
                content::appearance,
                Event::ChooseAppearance
            ),
            home,
        ]
        .spacing(16),
    )
    .width(Length::Fixed(WIDTH))
    .height(Length::Fill)
    .padding(12)
    .into()
}
