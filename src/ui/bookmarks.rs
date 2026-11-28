//! The Bookmarks side panel: the reader's own order, click to open, and each one's
//! move up, move down and remove buttons.

use iced::advanced::widget::Id;
use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Element, Fill, Length};

use super::app::Message;
use super::focus::{RING_REACH, pressable};
use super::icon;
use super::panels::PANEL_WIDTH;
use crate::content;
use crate::session::bookmarks::{Bookmark, Move};
use crate::session::{Event, State};
use crate::theme::icons::Icon;
use crate::theme::{self, chrome_text};

/// The moves a bookmark at `at` in a list `len` long can make: none past either end.
pub fn moves(at: usize, len: usize) -> (bool, bool) {
    (at > 0, at + 1 < len)
}

/// The id of a bookmark's move button, so the keyboard can follow the bookmark once it moves.
pub fn move_id(address: &str, by: Move) -> Id {
    let way = match by {
        Move::Up => "up",
        Move::Down => "down",
    };
    Id::from(format!("bookmark-{way}:{address}"))
}

/// Where the keyboard goes after `address` moved `by` by key: the same button on its new row,
/// or the other one when that end of the list turned it off. None when it can't move at all.
pub fn focus_after_move(bookmarks: &[Bookmark], address: &str, by: Move) -> Option<Id> {
    let at = bookmarks.iter().position(|b| b.address == address)?;
    let (up, down) = moves(at, bookmarks.len());
    let same = match by {
        Move::Up => up,
        Move::Down => down,
    };
    let other = match by {
        Move::Up => Move::Down,
        Move::Down => Move::Up,
    };
    match (same, up || down) {
        (true, _) => Some(move_id(address, by)),
        (false, true) => Some(move_id(address, other)),
        (false, false) => None,
    }
}

/// A move button: a click moves the bookmark, and Enter or Space also keeps the keyboard on it.
fn move_button(address: &str, by: Move, enabled: bool) -> Element<'_, Message> {
    let glyph = match by {
        Move::Up => Icon::MoveUp,
        Move::Down => Icon::MoveDown,
    };
    let click = enabled.then(|| Message::Session(Event::MoveBookmark(address.to_string(), by)));
    let key = enabled.then(|| Message::MoveBookmarkByKey(address.to_string(), by));
    icon::button_with_id(glyph, click, key, move_id(address, by))
}

fn entry(bookmark: &Bookmark, at: usize, len: usize, muted: iced::Color) -> Element<'_, Message> {
    let label = column![
        text(bookmark.title.as_str()).size(chrome_text::BODY),
        text(bookmark.address.as_str()).size(chrome_text::NOTE).style(move |_| text::Style { color: Some(muted) }),
    ]
    .spacing(2);
    let session = |event| Some(Message::Session(event));
    let open = session(Event::OpenBookmark(bookmark.address.clone()));
    let (up, down) = moves(at, len);
    row![
        pressable(button(label).width(Fill).style(button::text).on_press_maybe(open.clone()), open),
        move_button(&bookmark.address, Move::Up, up),
        move_button(&bookmark.address, Move::Down, down),
        icon::button(Icon::Remove, session(Event::RemoveBookmark(bookmark.address.clone()))),
    ]
    .spacing(4)
    .align_y(iced::alignment::Vertical::Center)
    .into()
}

pub fn panel(state: &State, palette: theme::Palette) -> Element<'_, Message> {
    let reader = theme::reading(theme::Look { font: state.settings.font, palette, ..theme::Look::default() });
    let muted = reader.muted;
    let len = state.bookmarks.len();
    let entries: Element<'_, Message> = if state.bookmarks.is_empty() {
        text(content::BOOKMARKS_EMPTY).size(chrome_text::LABEL).into()
    } else {
        column(state.bookmarks.iter().enumerate().map(|(at, bookmark)| entry(bookmark, at, len, muted)))
            .spacing(2)
            .into()
    };
    container(
        column![
            text(content::BOOKMARKS).size(chrome_text::HEADING),
            scrollable(container(entries).padding(RING_REACH)).height(Fill)
        ]
        .spacing(12),
    )
    .width(Length::Fixed(PANEL_WIDTH))
    .height(Fill)
    .padding(12)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn after_a_move_by_key_the_keyboard_follows_the_bookmark() {
        let list = |addresses: &[&str]| {
            addresses.iter().map(|a| Bookmark { address: (*a).into(), title: (*a).into() }).collect::<Vec<_>>()
        };
        let moved_up = list(&["a", "c", "b"]);
        assert_eq!(focus_after_move(&moved_up, "c", Move::Up), Some(move_id("c", Move::Up)));
        let to_the_top = list(&["c", "a", "b"]);
        assert_eq!(
            focus_after_move(&to_the_top, "c", Move::Up),
            Some(move_id("c", Move::Down)),
            "Up is off at the top"
        );
        let to_the_bottom = list(&["b", "a"]);
        assert_eq!(focus_after_move(&to_the_bottom, "a", Move::Down), Some(move_id("a", Move::Up)));
        assert_eq!(focus_after_move(&list(&["a"]), "a", Move::Down), None, "a lone bookmark can't move");
        assert_eq!(focus_after_move(&list(&["a"]), "z", Move::Up), None);
        assert_ne!(move_id("a", Move::Up), move_id("a", Move::Down));
        assert_ne!(move_id("a", Move::Up), move_id("b", Move::Up));
    }

    #[test]
    fn the_first_cannot_move_up_and_the_last_cannot_move_down() {
        assert_eq!(moves(0, 3), (false, true));
        assert_eq!(moves(1, 3), (true, true));
        assert_eq!(moves(2, 3), (true, false));
        assert_eq!(moves(0, 1), (false, false));
    }
}
