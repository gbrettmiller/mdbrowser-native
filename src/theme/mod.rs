//! The reader's stylesheet. Documents carry no presentation; all of it lives here.

pub mod fonts;
pub mod scale;

use iced::font::{Style, Weight};
use iced::{Color, Font};

pub use scale::{Spacing, TextSize};

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

/// Which palette the reader wants. System follows the operating system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

impl Appearance {
    pub const ALL: [Appearance; 3] = [Self::System, Self::Light, Self::Dark];

    /// Stable identifier for settings files.
    pub const fn key(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|appearance| appearance.key() == key)
    }

    /// The palette to draw with, given what the operating system reports (if anything).
    pub const fn palette(self, system: Option<Palette>) -> Palette {
        match (self, system) {
            (Self::Light, _) => Palette::Light,
            (Self::Dark, _) => Palette::Dark,
            (Self::System, Some(palette)) => palette,
            (Self::System, None) => Palette::Light,
        }
    }
}

/// The colours in use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Palette {
    #[default]
    Light,
    Dark,
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

const DARK: Theme = Theme {
    background: rgb(0x1b, 0x1b, 0x1a),
    ink: rgb(0xe4, 0xe2, 0xdc),
    muted: rgb(0xa1, 0x9f, 0x98),
    link: rgb(0x7f, 0xaf, 0xf0),
    code_ink: rgb(0xd6, 0xd4, 0xce),
    code_background: rgb(0x26, 0x26, 0x24),
    rule: rgb(0x3a, 0x39, 0x36),
    selection: Color::from_rgba8(0x7f, 0xaf, 0xf0, 0.30),
    ..LIGHT
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

/// The palette's tokens with the default body typeface: where only colours matter.
pub const fn tokens(palette: Palette) -> Theme {
    match palette {
        Palette::Light => LIGHT,
        Palette::Dark => DARK,
    }
}

/// Everything the reader chooses about how a page looks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Look {
    pub font: ReadingFont,
    pub palette: Palette,
    pub size: TextSize,
    pub spacing: Spacing,
}

/// A length set for the default body size, scaled to `size`. Multiplying first keeps
/// whole numbers exact, so the default size gives back the base value unchanged.
fn per_size(at_default: f32, size: TextSize) -> f32 {
    at_default * f32::from(size.px()) / f32::from(TextSize::default().px())
}

/// The reader theme for a look: its typeface and palette, and its size and spacing.
pub fn reading(look: Look) -> Theme {
    let base = tokens(look.palette);
    Theme {
        body_font: Font::with_name(look.font.family()),
        body_size: f32::from(look.size.px()),
        line_ratio: look.spacing.line_ratio(),
        measure: per_size(base.measure, look.size),
        gutter: per_size(base.gutter, look.size),
        indent: per_size(base.indent, look.size),
        block_gap: look.spacing.block_gap(),
        item_gap: look.spacing.item_gap(),
        ..base
    }
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

    fn look(font: ReadingFont, palette: Palette) -> Look {
        Look { font, palette, ..Look::default() }
    }

    #[test]
    fn keys_round_trip_and_unknown_keys_are_rejected() {
        for font in ReadingFont::ALL {
            assert_eq!(ReadingFont::from_key(font.key()), Some(font));
        }
        assert_eq!(ReadingFont::from_key("comic-sans"), None);
    }

    #[test]
    fn appearance_keys_round_trip_and_default_to_system() {
        for appearance in Appearance::ALL {
            assert_eq!(Appearance::from_key(appearance.key()), Some(appearance));
        }
        assert_eq!(Appearance::from_key("sepia"), None);
        assert_eq!(Appearance::default(), Appearance::System);
    }

    #[test]
    fn system_follows_the_os_and_is_light_when_it_says_nothing() {
        assert_eq!(Appearance::System.palette(Some(Palette::Dark)), Palette::Dark);
        assert_eq!(Appearance::System.palette(Some(Palette::Light)), Palette::Light);
        assert_eq!(Appearance::System.palette(None), Palette::Light);
        assert_eq!(Appearance::Dark.palette(Some(Palette::Light)), Palette::Dark);
        assert_eq!(Appearance::Light.palette(Some(Palette::Dark)), Palette::Light);
    }

    #[test]
    fn dark_changes_every_colour_and_nothing_else() {
        let (light, dark) =
            (reading(look(ReadingFont::Inter, Palette::Light)), reading(look(ReadingFont::Inter, Palette::Dark)));
        let colours =
            |t: &Theme| [t.background, t.ink, t.muted, t.link, t.code_ink, t.code_background, t.rule, t.selection];
        for (l, d) in colours(&light).into_iter().zip(colours(&dark)) {
            assert_ne!(l, d);
        }
        assert_eq!(
            Theme {
                background: light.background,
                ink: light.ink,
                muted: light.muted,
                link: light.link,
                code_ink: light.code_ink,
                code_background: light.code_background,
                rule: light.rule,
                selection: light.selection,
                ..dark
            },
            light
        );
    }

    /// WCAG 2.x relative luminance contrast.
    fn contrast(a: Color, b: Color) -> f32 {
        let channel = |c: f32| if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
        let luminance = |c: Color| 0.2126 * channel(c.r) + 0.7152 * channel(c.g) + 0.0722 * channel(c.b);
        let (hi, lo) = (luminance(a).max(luminance(b)), luminance(a).min(luminance(b)));
        (hi + 0.05) / (lo + 0.05)
    }

    #[test]
    fn text_meets_wcag_aa_in_both_palettes() {
        for palette in [Palette::Light, Palette::Dark] {
            let t = reading(look(ReadingFont::Literata, palette));
            for (name, ink, ground) in [
                ("ink", t.ink, t.background),
                ("muted", t.muted, t.background),
                ("link", t.link, t.background),
                ("code", t.code_ink, t.code_background),
                ("link on code", t.link, t.code_background),
                ("muted on code", t.muted, t.code_background),
            ] {
                assert!(contrast(ink, ground) >= 4.5, "{palette:?} {name}: {}", contrast(ink, ground));
            }
        }
    }

    #[test]
    fn reading_theme_swaps_only_the_body_face() {
        let theme = reading(look(ReadingFont::Inter, Palette::Light));
        assert_eq!(theme.body_font, Font::with_name("Inter"));
        assert_eq!(theme.mono_font, Font::with_name(MONO_FAMILY));
        assert_eq!(reading(Look::default()).body_font, Font::with_name("Literata"));
    }

    #[test]
    fn the_default_look_is_exactly_the_fixed_light_theme() {
        assert_eq!(reading(Look::default()), Theme { body_font: Font::with_name("Literata"), ..LIGHT });
        assert_eq!(reading(Look { palette: Palette::Dark, ..Look::default() }), DARK);
    }

    #[test]
    fn size_scales_the_measure_gutter_and_indent_in_proportion() {
        let size = |px| TextSize::from_px(px).unwrap();
        for px in (12..=32).step_by(2) {
            let (base, scaled) = (reading(Look::default()), reading(Look { size: size(px), ..Look::default() }));
            let ratio = px as f32 / 18.0;
            assert_eq!(scaled.body_size, px as f32);
            for (name, was, now) in [
                ("measure", base.measure, scaled.measure),
                ("gutter", base.gutter, scaled.gutter),
                ("indent", base.indent, scaled.indent),
            ] {
                assert!((now - was * ratio).abs() < 0.01, "{px} {name}: {now}");
            }
            assert_eq!(scaled.measure, 40.0 * px as f32, "the measure is 40 em");
            assert_eq!((scaled.heading_scales, scaled.mono_scale), (base.heading_scales, base.mono_scale));
        }
    }

    #[test]
    fn size_leaves_spacing_and_colours_alone() {
        let (base, big) =
            (reading(Look::default()), reading(Look { size: TextSize::default().larger(), ..Look::default() }));
        assert_eq!(
            Theme { body_size: base.body_size, measure: base.measure, gutter: base.gutter, indent: base.indent, ..big },
            base
        );
    }

    #[test]
    fn compact_is_tighter_and_spacious_looser_on_every_spacing_value() {
        let at = |spacing| reading(Look { spacing, ..Look::default() });
        let (compact, comfortable, spacious) = (at(Spacing::Compact), at(Spacing::Comfortable), at(Spacing::Spacious));
        assert_eq!((comfortable.line_ratio, comfortable.block_gap, comfortable.item_gap), (1.6, 16.0, 4.0));
        assert_eq!((compact.line_ratio, compact.block_gap, compact.item_gap), (1.45, 12.0, 2.0));
        assert_eq!((spacious.line_ratio, spacious.block_gap, spacious.item_gap), (1.8, 22.0, 8.0));
    }

    #[test]
    fn spacing_leaves_size_and_colours_alone() {
        let (base, loose) = (reading(Look::default()), reading(Look { spacing: Spacing::Spacious, ..Look::default() }));
        assert_eq!(
            Theme { line_ratio: base.line_ratio, block_gap: base.block_gap, item_gap: base.item_gap, ..loose },
            base
        );
    }
}
