//! The window around the page (toolbar, panels, pick lists, scrollbar) wears the
//! reader's palette, so dark mode has no stock-theme patches.

use std::sync::{Mutex, OnceLock};

use iced::theme::Palette as ChromePalette;

use crate::theme::{self, Colours, Palette};

/// An iced theme built from the palette's tokens: the page's background and ink,
/// with the link colour as the accent. Built once per palette, since `view` and
/// `theme` ask for it on every frame (for a custom palette, once per change of
/// colours). Success, danger and warning stay iced's own.
pub fn theme(palette: Palette) -> iced::Theme {
    static LIGHT: OnceLock<iced::Theme> = OnceLock::new();
    static DARK: OnceLock<iced::Theme> = OnceLock::new();
    match palette {
        Palette::Light => LIGHT.get_or_init(|| build("mdbrowser light", palette, ChromePalette::LIGHT)).clone(),
        Palette::Dark => DARK.get_or_init(|| build("mdbrowser dark", palette, ChromePalette::DARK)).clone(),
        Palette::Custom(colours) => custom(colours, palette),
    }
}

/// The latest custom theme, rebuilt only when the reader's colours change. Built on the
/// stock palette that matches the page, so the colours we don't set (success, danger)
/// read on it.
fn custom(colours: Colours, palette: Palette) -> iced::Theme {
    static LATEST: Mutex<Option<(Colours, iced::Theme)>> = Mutex::new(None);
    let mut latest = LATEST.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    match latest.as_ref() {
        Some((built, theme)) if *built == colours => theme.clone(),
        _ => {
            let tokens = theme::tokens(palette);
            let dark = tokens.background.r + tokens.background.g + tokens.background.b < 1.5;
            let stock = if dark { ChromePalette::DARK } else { ChromePalette::LIGHT };
            let theme = build("mdbrowser custom", palette, stock);
            *latest = Some((colours, theme.clone()));
            theme
        }
    }
}

fn build(name: &'static str, palette: Palette, stock: ChromePalette) -> iced::Theme {
    let tokens = theme::tokens(palette);
    iced::Theme::custom(
        name,
        ChromePalette { background: tokens.background, text: tokens.ink, primary: tokens.link, ..stock },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chrome_wears_the_palettes_background_ink_and_link() {
        let sepia = Colours { page: [0xfd, 0xf6, 0xe3], text: [0x33, 0x33, 0x33], link: [0x0b, 0x5f, 0xa5] };
        let night = Colours { page: [0x10, 0x14, 0x20], text: [0xe0, 0xe0, 0xe8], link: [0x80, 0xb0, 0xff] };
        for palette in [Palette::Light, Palette::Dark, Palette::Custom(sepia), Palette::Custom(night)] {
            let (tokens, chrome) = (theme::tokens(palette), theme(palette).palette());
            assert_eq!((chrome.background, chrome.text, chrome.primary), (tokens.background, tokens.ink, tokens.link));
        }
    }

    #[test]
    fn a_change_of_custom_colours_changes_the_chrome_and_the_same_colours_reuse_it() {
        let one = Colours { page: [0xfd, 0xf6, 0xe3], text: [0x33, 0x33, 0x33], link: [0x0b, 0x5f, 0xa5] };
        let two = Colours { page: [0xee, 0xee, 0xff], ..one };
        let (first, again, other) =
            (theme(Palette::Custom(one)), theme(Palette::Custom(one)), theme(Palette::Custom(two)));
        assert_eq!(first.palette().background, again.palette().background);
        assert_ne!(first.palette().background, other.palette().background);
        assert_eq!(theme(Palette::Custom(one)).palette().background, first.palette().background);
    }
}
