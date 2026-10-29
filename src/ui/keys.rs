//! Keyboard shortcuts: which key chord does what. Pure, so each chord is tested.

use iced::keyboard::{Key, Modifiers, key::Named};

use crate::session::Event;

/// What a shortcut asks for.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Session(Event),
    FocusAddress,
    ToggleHistory,
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
        assert_eq!(command(Key::Named(Named::F5), Modifiers::empty()), Some(Command::Session(Event::Reload)));
        assert_eq!(command(Key::Named(Named::ArrowLeft), Modifiers::ALT), Some(Command::Session(Event::Back)));
    }

    #[test]
    fn plain_keys_and_missing_modifiers_do_nothing() {
        assert_eq!(command(Key::Character("i"), primary()), None, "Shift is needed");
        assert_eq!(command(Key::Character("i"), Modifiers::empty()), None);
        assert_eq!(command(Key::Named(Named::ArrowLeft), Modifiers::empty()), None);
    }
}
