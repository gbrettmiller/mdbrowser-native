//! The window around the page (toolbar, panels, pick lists, scrollbar) wears the
//! reader's palette, so dark mode has no stock-theme patches.

use std::sync::OnceLock;

use iced::theme::Palette as ChromePalette;

use crate::theme::{self, Palette};

/// An iced theme built from the palette's tokens: the page's background and ink,
/// with the link colour as the accent. Built once per palette, since `view` and
/// `theme` ask for it on every frame. Success, danger and warning stay iced's own.
pub fn theme(palette: Palette) -> iced::Theme {
    static LIGHT: OnceLock<iced::Theme> = OnceLock::new();
    static DARK: OnceLock<iced::Theme> = OnceLock::new();
    match palette {
        Palette::Light => LIGHT.get_or_init(|| build("mdbrowser light", palette, ChromePalette::LIGHT)),
        Palette::Dark => DARK.get_or_init(|| build("mdbrowser dark", palette, ChromePalette::DARK)),
    }
    .clone()
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
        for palette in [Palette::Light, Palette::Dark] {
            let (tokens, chrome) = (theme::tokens(palette), theme(palette).palette());
            assert_eq!((chrome.background, chrome.text, chrome.primary), (tokens.background, tokens.ink, tokens.link));
        }
    }
}
