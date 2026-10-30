//! The Settings side panel: font, web images, appearance and the home page.
//! Choices apply at once and are saved by the session.

use std::fmt;

use iced::widget::{button, column, container, pick_list, row, text, text_input};
use iced::{Element, Length};

use super::app::Message;
use crate::content;
use crate::session::images::ImageLoading;
use crate::session::{Event, State, TextStep};
use crate::theme::{Appearance, ReadingFont, Spacing};

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

/// Smaller, the size in pixels, larger. The buttons are disabled at the ends of the scale.
fn text_size<'a>(state: &State) -> Element<'a, Message> {
    let size = state.settings.text_size;
    let step = |label: &'static str, step, enabled: bool| {
        button(text(label).size(14).center())
            .on_press_maybe(enabled.then_some(Message::Session(Event::StepTextSize(step))))
    };
    column![
        text(content::TEXT_SIZE_LABEL).size(14),
        row![
            step(content::SMALLER_TEXT, TextStep::Smaller, size != size.smaller()),
            text(content::text_size(size)).size(14).width(Length::Fill).center(),
            step(content::LARGER_TEXT, TextStep::Larger, size != size.larger()),
        ]
        .spacing(8)
        .align_y(iced::Center),
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
            text_size(state),
            choice(content::SPACING_LABEL, &Spacing::ALL, settings.spacing, content::spacing, Event::ChooseSpacing),
            button(text(content::RESET_READER_CONTROLS).size(14))
                .on_press(Message::Session(Event::ResetReaderControls)),
            home,
        ]
        .spacing(16),
    )
    .width(Length::Fixed(WIDTH))
    .height(Length::Fill)
    .padding(12)
    .into()
}
