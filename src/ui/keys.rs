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
    /// Esc: close the panel that is open, and return to the page.
    ClosePanel,
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
        Key::Named(Named::Escape) if !primary && !alt && !shift => Command::ClosePanel,
        Key::Named(Named::F5) => Command::Session(Event::Reload),
        // About and help. F1 needs Fn on many Mac keyboards, so Cmd+? (Ctrl+? elsewhere) is the second route;
        // `?` is Shift+/, which some layouts report as `/`. Not bare keys: those type into the address bar.
        Key::Named(Named::F1) if !primary && !alt => Command::Session(Event::About),
        Key::Character("?") if primary && !alt => Command::Session(Event::About),
        Key::Character("/") if primary && shift && !alt => Command::Session(Event::About),
        Key::Character("r") if primary => Command::Session(Event::Reload),
        Key::Character("l") if primary => Command::FocusAddress,
        Key::Character("h") if primary => Command::ToggleHistory,
        Key::Character(",") if primary => Command::ToggleSettings,
        // `=` and `+` are one key; Shift makes the second on most layouts. Not bare keys: those
        // type into the address bar.
        Key::Character("=" | "+") if primary => Command::Session(Event::StepTextSize(TextStep::Larger)),
        Key::Character("-") if primary => Command::Session(Event::StepTextSize(TextStep::Smaller)),
        Key::Character("0") if primary => Command::Session(Event::StepTextSize(TextStep::Reset)),
        // The previous and next post of a blog.
        Key::Character("[") if primary => Command::Session(Event::Neighbour(Neighbour::Previous)),
        Key::Character("]") if primary => Command::Session(Event::Neighbour(Neighbour::Next)),
        Key::Named(Named::Home) if alt => Command::Session(Event::Home),
        // Opens the original of a page found by fallback, or an error's offer; the session ignores it otherwise.
        Key::Character("o" | "O") if primary && shift => Command::Session(Event::OpenInBrowser),
        // The keyboard route to every image note's click, and to images that are off.
        Key::Character("i" | "I") if primary && shift => Command::Session(Event::LoadPageImages),
        // Toggles the page and its Markdown source; the session ignores it with no page shown.
        Key::Character("u" | "U") if primary && !shift && !alt => Command::Session(Event::ToggleSource),
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
    fn ctrl_or_cmd_u_toggles_the_source_and_nothing_else_does() {
        let toggle = Some(Command::Session(Event::ToggleSource));
        assert_eq!(command(Key::Character("u"), primary()), toggle);
        assert_eq!(command(Key::Character("u"), Modifiers::empty()), None, "bare u types into a field");
        assert_eq!(command(Key::Character("U"), primary()), toggle, "Caps Lock");
        assert_eq!(command(Key::Character("u"), Modifiers::ALT), None);
        assert_eq!(command(Key::Character("U"), primary().union(Modifiers::SHIFT)), None);
    }

    #[test]
    fn f1_and_ctrl_or_cmd_question_mark_open_about_and_nothing_else_does() {
        let about = Some(Command::Session(Event::About));
        assert_eq!(command(Key::Named(Named::F1), Modifiers::empty()), about);
        assert_eq!(command(Key::Character("?"), primary()), about);
        assert_eq!(command(Key::Character("?"), primary().union(Modifiers::SHIFT)), about);
        assert_eq!(command(Key::Character("/"), primary().union(Modifiers::SHIFT)), about, "Shift+/ on a US layout");
        assert_eq!(command(Key::Character("?"), Modifiers::empty()), None, "a bare ? types into a field");
        assert_eq!(command(Key::Character("/"), primary()), None);
        assert_eq!(command(Key::Named(Named::F1), Modifiers::ALT), None);
        let with_alt = primary().union(Modifiers::ALT);
        assert_eq!(command(Key::Character("?"), with_alt), None, "Ctrl+Alt is how AltGr types characters");
        assert_eq!(command(Key::Character("/"), with_alt.union(Modifiers::SHIFT)), None);
    }

    /// Each shortcut the About page lists is in the page's text and does what the page says.
    #[test]
    fn every_shortcut_the_about_page_lists_works() {
        let page = include_str!("../../assets/about.md");
        let session = |event| Some(Command::Session(event));
        let table = [
            ("F1 or Ctrl+?", command(Key::Named(Named::F1), Modifiers::empty()), session(Event::About)),
            ("Ctrl+,", command(Key::Character(","), primary()), Some(Command::ToggleSettings)),
            ("Ctrl+H", command(Key::Character("h"), primary()), Some(Command::ToggleHistory)),
            ("Ctrl+L", command(Key::Character("l"), primary()), Some(Command::FocusAddress)),
            ("Ctrl+U", command(Key::Character("u"), primary()), session(Event::ToggleSource)),
            ("Alt+Home", command(Key::Named(Named::Home), Modifiers::ALT), session(Event::Home)),
            (
                "Ctrl+=, Ctrl+-, Ctrl+0",
                command(Key::Character("="), primary()),
                session(Event::StepTextSize(TextStep::Larger)),
            ),
            ("Tab or Shift+Tab", command(Key::Named(Named::Tab), Modifiers::empty()), Some(Command::FocusNext)),
            (
                "| Close a panel | Esc |",
                command(Key::Named(Named::Escape), Modifiers::empty()),
                Some(Command::ClosePanel),
            ),
        ];
        for (listed, actual, expected) in table {
            assert!(page.contains(listed), "the About page no longer lists {listed}");
            assert_eq!(actual, expected, "{listed}");
        }
    }

    #[test]
    fn every_earlier_chord_is_unchanged() {
        let session = |event| Some(Command::Session(event));
        let shifted = primary().union(Modifiers::SHIFT);
        let table = [
            (command(Key::Character("r"), primary()), session(Event::Reload)),
            (command(Key::Named(Named::F5), Modifiers::empty()), session(Event::Reload)),
            (command(Key::Character("l"), primary()), Some(Command::FocusAddress)),
            (command(Key::Character("h"), primary()), Some(Command::ToggleHistory)),
            (command(Key::Character(","), primary()), Some(Command::ToggleSettings)),
            (command(Key::Character("="), primary()), session(Event::StepTextSize(TextStep::Larger))),
            (command(Key::Character("-"), primary()), session(Event::StepTextSize(TextStep::Smaller))),
            (command(Key::Character("0"), primary()), session(Event::StepTextSize(TextStep::Reset))),
            (command(Key::Character("["), primary()), session(Event::Neighbour(Neighbour::Previous))),
            (command(Key::Character("]"), primary()), session(Event::Neighbour(Neighbour::Next))),
            (command(Key::Named(Named::ArrowLeft), Modifiers::ALT), session(Event::Back)),
            (command(Key::Named(Named::ArrowRight), Modifiers::ALT), session(Event::Forward)),
            (command(Key::Named(Named::Home), Modifiers::ALT), session(Event::Home)),
            (command(Key::Character("O"), shifted), session(Event::OpenInBrowser)),
            (command(Key::Character("I"), shifted), session(Event::LoadPageImages)),
            (command(Key::Named(Named::Escape), Modifiers::empty()), Some(Command::ClosePanel)),
            (command(Key::Named(Named::Tab), Modifiers::empty()), Some(Command::FocusNext)),
            (command(Key::Named(Named::Tab), Modifiers::SHIFT), Some(Command::FocusPrevious)),
        ];
        for (actual, expected) in table {
            assert_eq!(actual, expected);
        }
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
    fn shift_turns_none_of_the_other_scroll_keys_into_a_scroll() {
        for key in [Named::PageUp, Named::PageDown, Named::ArrowUp, Named::ArrowDown, Named::Home, Named::End] {
            assert_eq!(scroll(named(key), Modifiers::SHIFT, true), None, "{key:?}");
        }
    }

    #[test]
    fn every_scroll_key_ignores_ctrl_and_alt() {
        let keys =
            [Named::PageUp, Named::PageDown, Named::Space, Named::ArrowUp, Named::ArrowDown, Named::Home, Named::End];
        for key in keys {
            for modifiers in [primary(), Modifiers::ALT] {
                assert_eq!(scroll(named(key), modifiers, true), None, "{key:?} with {modifiers:?}");
            }
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
    fn escape_closes_a_panel_but_not_with_a_modifier() {
        assert_eq!(command(named(Named::Escape), Modifiers::empty()), Some(Command::ClosePanel));
        assert_eq!(command(named(Named::Escape), Modifiers::SHIFT), None);
        assert_eq!(command(named(Named::Escape), primary()), None);
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
