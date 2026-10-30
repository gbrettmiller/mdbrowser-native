//! Keyboard shortcuts: which key chord does what. Pure, so each chord is tested.

use iced::keyboard::{Key, Modifiers, key::Named};

use crate::session::{Event, Neighbour, TextStep};

/// What a shortcut asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Session(Event),
    FocusAddress,
    ToggleHistory,
    ToggleSettings,
}

/// The command for `key` pressed with `modifiers`, if it is a shortcut.
pub fn command(key: Key<&str>, modifiers: Modifiers) -> Option<Command> {
    let (primary, shift, alt) = (modifiers.command(), modifiers.shift(), modifiers.alt());
    let command = match key {
        Key::Named(Named::ArrowLeft) if alt => Command::Session(Event::Back),
        Key::Named(Named::ArrowRight) if alt => Command::Session(Event::Forward),
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
    fn plain_keys_and_missing_modifiers_do_nothing() {
        assert_eq!(command(Key::Character("i"), primary()), None, "Shift is needed");
        assert_eq!(command(Key::Character("i"), Modifiers::empty()), None);
        assert_eq!(command(Key::Character("]"), Modifiers::empty()), None, "a bare bracket is typing");
        for key in ["=", "+", "-", "0"] {
            assert_eq!(command(Key::Character(key), Modifiers::empty()), None, "{key} types into the address bar");
        }
        assert_eq!(command(Key::Named(Named::ArrowLeft), Modifiers::empty()), None);
        assert_eq!(command(Key::Named(Named::Home), Modifiers::empty()), None, "Home scrolls; it needs Alt");
    }
}
