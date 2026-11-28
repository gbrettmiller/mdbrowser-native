//! Icon buttons (ADR-016): a Font Awesome glyph, a tooltip naming the action and its shortcut,
//! and the same name for the status line while the button has the keyboard.

use iced::Element;
use iced::advanced::widget::Id;
use iced::widget::{text, tooltip};

use super::app::Message;
use super::focus;
use crate::content;
use crate::theme::chrome_text;
use crate::theme::icons::Icon;

fn glyph<'a>(icon: Icon) -> Element<'a, Message> {
    text(icon.glyph()).font(icon.font()).size(chrome_text::BODY).into()
}

fn name(icon: Icon) -> String {
    content::icon_name(icon, cfg!(target_os = "macos"))
}

fn tipped<'a>(stop: Element<'a, Message>, icon: Icon) -> Element<'a, Message> {
    tooltip(stop, text(name(icon)), tooltip::Position::Bottom).into()
}

/// An icon button; a disabled one (no message) is skipped by Tab.
pub fn button<'a>(icon: Icon, on_press: Option<Message>) -> Element<'a, Message> {
    tipped(focus::named_button(glyph(icon), on_press, name(icon)), icon)
}

/// An icon button that `operation::focus(id)` can reach, whose Enter or Space sends `on_key`.
pub fn button_with_id<'a>(
    icon: Icon,
    on_press: Option<Message>,
    on_key: Option<Message>,
    id: Id,
) -> Element<'a, Message> {
    tipped(focus::button_with_id(glyph(icon), on_press, on_key, id, name(icon)), icon)
}
