//! Keyboard shortcuts: which key chord does what. Pure, so each chord is tested.

use iced::keyboard::{Key, Modifiers, key::Named};

use crate::session::{Event, Neighbour, TextStep};

/// A move of the page from the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scroll {
    PageUp,
    PageDown,
    LineUp,
    LineDown,
    Top,
    Bottom,
}

/// The scroll `key` asks for. PageUp and PageDown scroll wherever focus is; the other keys
/// (Space, the arrows, Home and End) only when the page has it, because a text field needs them.
pub fn scroll(key: Key<&str>, modifiers: Modifiers, page_focused: bool) -> Option<Scroll> {
    if modifiers.command() || modifiers.alt() {
        return None;
    }
    let scroll = match key {
        Key::Named(Named::PageUp) if !modifiers.shift() => Scroll::PageUp,
        Key::Named(Named::PageDown) if !modifiers.shift() => Scroll::PageDown,
        Key::Named(Named::Space) if page_focused => {
            if modifiers.shift() {
                Scroll::PageUp
            } else {
                Scroll::PageDown
            }
        }
        Key::Named(Named::ArrowUp) if page_focused && !modifiers.shift() => Scroll::LineUp,
        Key::Named(Named::ArrowDown) if page_focused && !modifiers.shift() => Scroll::LineDown,
        Key::Named(Named::Home) if page_focused && !modifiers.shift() => Scroll::Top,
        Key::Named(Named::End) if page_focused && !modifiers.shift() => Scroll::Bottom,
        _ => return None,
    };
    Some(scroll)
}

/// What a shortcut asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Session(Event),
    FocusAddress,
    ToggleHistory,
    ToggleSettings,
    /// Tab and Shift+Tab, when nothing on the page used them: on to the next or previous control.
    FocusNext,
    FocusPrevious,
    Scroll(Scroll),
}

/// The command for `key` pressed with `modifiers`, if it is a shortcut.
pub fn command(key: Key<&str>, modifiers: Modifiers) -> Option<Command> {
    let (primary, shift, alt) = (modifiers.command(), modifiers.shift(), modifiers.alt());
    if let Some(scroll) = scroll(key.clone(), modifiers, false) {
        return Some(Command::Scroll(scroll));
    }
    let command = match key {
        Key::Named(Named::ArrowLeft) if alt => Command::Session(Event::Back),
        Key::Named(Named::ArrowRight) if alt => Command::Session(Event::Forward),
        Key::Named(Named::Tab) if !primary && !alt => {
            if shift {
                Command::FocusPrevious
            } else {
                Command::FocusNext
            }
        }
        Key::Named(Named::F5) => Command::Session(Event::Reload),
        Key::Character("r") if primary => Command::Session(Event::Reload),
        Key::Character("l") if primary => Command::FocusAddress,
        Key::Character("h") if primary => Command::ToggleHistory,
        Key::Character(",") if primary => Command::ToggleSettings,
        // The previous and next post of a blog. Not bare keys: those type into the address bar.
        // `=` and `+` are one key; Shift makes the second on most layouts. Neither is a bare key.
        Key::Character("=" | "+") if primary => Command::Session(Event::StepTextSize(TextStep::Larger)),
        Key::Character("-") if primary => Command::Session(Event::StepTextSize(TextStep::Smaller)),
        Key::Character("0") if primary => Command::Session(Event::StepTextSize(TextStep::Reset)),
        Key::Character("[") if primary => Command::Session(Event::Neighbour(Neighbour::Previous)),
        Key::Character("]") if primary => Command::Session(Event::Neighbour(Neighbour::Next)),
        Key::Named(Named::Home) if alt => Command::Session(Event::Home),
        // The only keyboard route to the button; the session ignores it unless offered.
        Key::Character("o" | "O") if primary && shift => Command::Session(Event::OpenInBrowser),
        // The keyboard route to every image note's click, and to images that are off.
        Key::Character("i" | "I") if primary && shift => Command::Session(Event::LoadPageImages),
        _ => return None,
    };
    Some(command)
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn primary() -> Modifiers {
        if cfg!(target_os = "macos") { Modifiers::LOGO } else { Modifiers::CTRL }
    }

    #[test]
    fn each_chord_maps_to_its_command() {
        let shifted = primary().union(Modifiers::SHIFT);
        assert_eq!(command(Key::Character("I"), shifted), Some(Command::Session(Event::LoadPageImages)));
        assert_eq!(command(Key::Character("O"), shifted), Some(Command::Session(Event::OpenInBrowser)));
        assert_eq!(command(Key::Character("l"), primary()), Some(Command::FocusAddress));
        assert_eq!(command(Key::Character("h"), primary()), Some(Command::ToggleHistory));
        assert_eq!(command(Key::Character(","), primary()), Some(Command::ToggleSettings));
        assert_eq!(command(Key::Named(Named::Home), Modifiers::ALT), Some(Command::Session(Event::Home)));
        assert_eq!(command(Key::Named(Named::F5), Modifiers::empty()), Some(Command::Session(Event::Reload)));
        assert_eq!(command(Key::Named(Named::ArrowLeft), Modifiers::ALT), Some(Command::Session(Event::Back)));
        let size = |step| Some(Command::Session(Event::StepTextSize(step)));
        assert_eq!(command(Key::Character("="), primary()), size(TextStep::Larger));
        assert_eq!(command(Key::Character("+"), primary().union(Modifiers::SHIFT)), size(TextStep::Larger));
        assert_eq!(command(Key::Character("-"), primary()), size(TextStep::Smaller));
        assert_eq!(command(Key::Character("0"), primary()), size(TextStep::Reset));
        let neighbour = |which| Some(Command::Session(Event::Neighbour(which)));
        assert_eq!(command(Key::Character("["), primary()), neighbour(Neighbour::Previous));
        assert_eq!(command(Key::Character("]"), primary()), neighbour(Neighbour::Next));
    }

    #[test]
    fn tab_moves_focus_on_and_shift_tab_back_but_not_with_other_modifiers() {
        let tab = Key::Named(Named::Tab);
        assert_eq!(command(tab.clone(), Modifiers::empty()), Some(Command::FocusNext));
        assert_eq!(command(tab.clone(), Modifiers::SHIFT), Some(Command::FocusPrevious));
        assert_eq!(command(tab.clone(), primary()), None);
        assert_eq!(command(tab, Modifiers::ALT), None);
    }

    fn named(key: Named) -> Key<&'static str> {
        Key::Named(key)
    }

    #[test]
    fn page_up_and_page_down_scroll_wherever_focus_is() {
        for focused in [false, true] {
            assert_eq!(scroll(named(Named::PageDown), Modifiers::empty(), focused), Some(Scroll::PageDown));
            assert_eq!(scroll(named(Named::PageUp), Modifiers::empty(), focused), Some(Scroll::PageUp));
        }
        assert_eq!(command(named(Named::PageDown), Modifiers::empty()), Some(Command::Scroll(Scroll::PageDown)));
    }

    #[test]
    fn the_other_scroll_keys_need_the_page_to_have_focus() {
        let keys = [
            (Named::Space, Modifiers::empty(), Scroll::PageDown),
            (Named::Space, Modifiers::SHIFT, Scroll::PageUp),
            (Named::ArrowDown, Modifiers::empty(), Scroll::LineDown),
            (Named::ArrowUp, Modifiers::empty(), Scroll::LineUp),
            (Named::Home, Modifiers::empty(), Scroll::Top),
            (Named::End, Modifiers::empty(), Scroll::Bottom),
        ];
        for (key, modifiers, expected) in keys {
            assert_eq!(scroll(named(key), modifiers, true), Some(expected), "{key:?}");
            assert_eq!(scroll(named(key), modifiers, false), None, "{key:?} types into a field");
            assert_eq!(command(named(key), modifiers), None, "{key:?} is not an app shortcut");
        }
    }

    #[test]
    fn scrolling_leaves_chords_with_a_modifier_alone() {
        assert_eq!(scroll(named(Named::PageDown), primary(), true), None);
        assert_eq!(scroll(named(Named::ArrowDown), Modifiers::ALT, true), None);
        assert_eq!(scroll(named(Named::Home), Modifiers::ALT, true), None, "Alt+Home is the home page");
        assert_eq!(command(named(Named::Home), Modifiers::ALT), Some(Command::Session(Event::Home)));
    }

    #[test]
    fn plain_keys_and_missing_modifiers_do_nothing() {
        assert_eq!(command(Key::Character("i"), primary()), None, "Shift is needed");
        assert_eq!(command(Key::Character("i"), Modifiers::empty()), None);
        assert_eq!(command(Key::Character("]"), Modifiers::empty()), None, "a bare bracket is typing");
        for key in ["=", "+", "-", "0"] {
            assert_eq!(command(Key::Character(key), Modifiers::empty()), None, "{key} types into the address bar");
        }
        assert_eq!(command(Key::Named(Named::ArrowLeft), Modifiers::empty()), None);
        assert_eq!(
            command(Key::Named(Named::Home), Modifiers::empty()),
            None,
            "Home scrolls the page; it needs Alt for the home page"
        );
    }
}
