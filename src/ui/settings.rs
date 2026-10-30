//! The Settings side panel: font, web images, appearance and the home page.
//! Choices apply at once and are saved by the session.

use std::fmt;

use iced::widget::{column, container, pick_list, row, scrollable, text, text_input};
use iced::{Element, Length};

use super::app::Message;
use super::focus::{self, button, neighbour};
use crate::content;
use crate::session::images::ImageLoading;
use crate::session::{Event, State, TextStep};
use crate::theme::colours::hex;
use crate::theme::{Appearance, Colours, ReadingFont, Spacing, Which};

const WIDTH: f32 = 300.0;
/// Space beside the controls so the panel's scrollbar doesn't cover them.
const SCROLLBAR_ROOM: f32 = 16.0;

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
    let step = |forward| neighbour(all, current, forward).map(|value| Message::Session(event(value)));
    column![
        text(label).size(14),
        focus::choice(
            pick_list(options, Some(Shown(current, name)), move |Shown(value, _)| Message::Session(event(value)))
                .width(Length::Fill),
            step(false),
            step(true),
        ),
    ]
    .spacing(4)
    .into()
}

/// Smaller, the size in pixels, larger. The buttons are disabled at the ends of the scale.
fn text_size<'a>(state: &State) -> Element<'a, Message> {
    let size = state.settings.text_size;
    let step = |label: &'static str, step, enabled: bool| {
        button(text(label).size(14).center(), enabled.then_some(Message::Session(Event::StepTextSize(step))))
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

/// What the three colour fields show: the saved colours until the reader types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColourDrafts([String; 3]);

impl ColourDrafts {
    pub fn of(colours: Colours) -> Self {
        ColourDrafts(Which::ALL.map(|which| hex(colours.get(which))))
    }

    pub fn get(&self, which: Which) -> &str {
        &self.0[which.index()]
    }

    /// The drafts with `which` as the reader typed it.
    pub fn typed(mut self, which: Which, text: String) -> Self {
        self.0[which.index()] = text;
        self
    }
}

/// The three hex fields, each applied on Enter. They show only for the Custom appearance.
fn colour_fields<'a>(drafts: &'a ColourDrafts) -> Element<'a, Message> {
    let field = |which| -> Element<'a, Message> {
        column![
            text(content::colour_name(which)).size(14),
            text_input(content::COLOUR_PLACEHOLDER, drafts.get(which))
                .on_input(move |typed| Message::ColourDraftChanged(which, typed))
                .on_submit(Message::SubmitColour(which)),
        ]
        .spacing(4)
        .into()
    };
    column![text(content::COLOURS_LABEL).size(14)].extend(Which::ALL.map(field)).spacing(8).into()
}

pub fn panel<'a>(state: &'a State, home_draft: &'a str, colours: &'a ColourDrafts) -> Element<'a, Message> {
    let settings = &state.settings;
    let home = column![
        text(content::HOME_PAGE_LABEL).size(14),
        text_input(content::HOME_PAGE_PLACEHOLDER, home_draft)
            .on_input(Message::HomeDraftChanged)
            .on_submit(Message::SubmitHome),
    ]
    .spacing(4);
    let fields = column![
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
    ]
    // Right under the Appearance choice that reveals them.
    .push((settings.appearance == Appearance::Custom).then(|| colour_fields(colours)))
    .push(text_size(state))
    .push(choice(content::SPACING_LABEL, &Spacing::ALL, settings.spacing, content::spacing, Event::ChooseSpacing))
    .push(button(text(content::RESET_READER_CONTROLS).size(14), Some(Message::Session(Event::ResetReaderControls))))
    .push(home)
    .spacing(16)
    .padding(iced::Padding::ZERO.right(SCROLLBAR_ROOM));
    // The panel is taller than a small window once the colour fields show.
    container(scrollable(fields)).width(Length::Fixed(WIDTH)).height(Length::Fill).padding(12).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_drafts_start_as_the_saved_colours_in_hex() {
        let drafts =
            ColourDrafts::of(Colours { page: [0xfd, 0xf6, 0xe3], text: [0x33, 0x33, 0x33], link: [0x0b, 0x5f, 0xa5] });
        assert_eq!(
            (drafts.get(Which::Page), drafts.get(Which::Text), drafts.get(Which::Link)),
            ("#fdf6e3", "#333333", "#0b5fa5")
        );
    }

    #[test]
    fn typing_changes_one_draft_and_leaves_the_others() {
        let drafts = ColourDrafts::of(Colours::default());
        let typed = drafts.clone().typed(Which::Text, "#3".into());
        assert_eq!(typed.get(Which::Text), "#3");
        for other in [Which::Page, Which::Link] {
            assert_eq!(typed.get(other), drafts.get(other));
        }
    }
}
