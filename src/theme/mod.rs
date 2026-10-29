//! The reader's stylesheet. Documents carry no presentation; all of it lives here.

pub mod fonts;

use iced::font::{Style, Weight};
use iced::{Color, Font};

/// The body typeface the reader has chosen. Code is always JetBrains Mono.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReadingFont {
    #[default]
    Literata,
    SourceSerif,
    Quattro,
    Inter,
}

impl ReadingFont {
    pub const ALL: [ReadingFont; 4] = [Self::Literata, Self::SourceSerif, Self::Quattro, Self::Inter];

    /// The family name inside the bundled font files.
    pub const fn family(self) -> &'static str {
        match self {
            Self::Literata => "Literata",
            Self::SourceSerif => "Source Serif 4",
            Self::Quattro => "iA Writer Quattro S",
            Self::Inter => "Inter",
        }
    }

    /// Stable identifier for settings files.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Literata => "literata",
            Self::SourceSerif => "source-serif-4",
            Self::Quattro => "ia-writer-quattro",
            Self::Inter => "inter",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|font| font.key() == key)
    }
}

pub const MONO_FAMILY: &str = "JetBrains Mono";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Theme {
    pub body_font: Font,
    pub mono_font: Font,
    pub body_size: f32,
    pub line_ratio: f32,
    pub mono_scale: f32,
    /// The note under an image's description is smaller than the text.
    pub note_scale: f32,
    pub heading_scales: [f32; 6],
    pub measure: f32,
    pub gutter: f32,
    pub block_gap: f32,
    pub item_gap: f32,
    pub indent: f32,
    pub code_padding: f32,
    pub cell_padding: f32,
    pub background: Color,
    pub ink: Color,
    pub muted: Color,
    pub link: Color,
    pub code_ink: Color,
    pub code_background: Color,
    pub rule: Color,
    pub selection: Color,
}

const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

const LIGHT: Theme = Theme {
    body_font: Font::with_name("Literata"),
    mono_font: Font::with_name(MONO_FAMILY),
    body_size: 18.0,
    line_ratio: 1.6,
    mono_scale: 0.88,
    note_scale: 0.85,
    heading_scales: [1.9, 1.5, 1.25, 1.1, 1.0, 0.9],
    measure: 720.0,
    gutter: 32.0,
    block_gap: 16.0,
    item_gap: 4.0,
    indent: 28.0,
    code_padding: 14.0,
    cell_padding: 10.0,
    background: rgb(0xfb, 0xfa, 0xf7),
    ink: rgb(0x22, 0x22, 0x22),
    muted: rgb(0x6b, 0x6b, 0x6b),
    link: rgb(0x1f, 0x5f, 0xbf),
    code_ink: rgb(0x33, 0x33, 0x33),
    code_background: rgb(0xef, 0xed, 0xe8),
    rule: rgb(0xd8, 0xd5, 0xce),
    selection: Color::from_rgba8(0x1f, 0x5f, 0xbf, 0.22),
};

/// The narrowest text column, however narrow the window.
const MIN_COLUMN: f32 = 80.0;

impl Theme {
    /// The text column's width in a window `window` wide: the measure, less the
    /// gutters when the window is narrow.
    pub fn column_width(&self, window: f32) -> f32 {
        (window - 2.0 * self.gutter).min(self.measure).max(MIN_COLUMN)
    }
}

/// The reader theme with the chosen body typeface.
pub const fn reading(font: ReadingFont) -> Theme {
    Theme { body_font: Font::with_name(font.family()), ..LIGHT }
}

pub fn bold(font: Font) -> Font {
    Font { weight: Weight::Bold, ..font }
}

pub fn italic(font: Font) -> Font {
    Font { style: Style::Italic, ..font }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_round_trip_and_unknown_keys_are_rejected() {
        for font in ReadingFont::ALL {
            assert_eq!(ReadingFont::from_key(font.key()), Some(font));
        }
        assert_eq!(ReadingFont::from_key("comic-sans"), None);
    }

    #[test]
    fn reading_theme_swaps_only_the_body_face() {
        let theme = reading(ReadingFont::Inter);
        assert_eq!(theme.body_font, Font::with_name("Inter"));
        assert_eq!(theme.mono_font, Font::with_name(MONO_FAMILY));
        assert_eq!(reading(ReadingFont::default()).body_font, Font::with_name("Literata"));
    }
}
