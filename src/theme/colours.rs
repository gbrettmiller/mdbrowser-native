//! The reader's own colours: three typed values, the rest derived from them.

use iced::Color;

use super::{Palette, Theme, tokens};

/// WCAG 2.x AA contrast for body text.
pub const AA: f32 = 4.5;

/// What the reader types: the page, the text and the links. Whole bytes, so a set can be
/// compared, hashed and stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Colours {
    pub page: [u8; 3],
    pub text: [u8; 3],
    pub link: [u8; 3],
}

/// Why a set of colours is refused: the text, or the links, can't be read on the page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unreadable {
    Text,
    Link,
}

/// `#rgb` or `#rrggbb`, in either case, and nothing around it.
pub fn parse_hex(typed: &str) -> Option<[u8; 3]> {
    let digits = typed.strip_prefix('#')?;
    if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let value = |digits: &str| u8::from_str_radix(digits, 16).ok();
    match digits.len() {
        3 => {
            let each = |at: usize| value(&digits[at..=at]).map(|nibble| nibble * 17);
            Some([each(0)?, each(1)?, each(2)?])
        }
        6 => Some([value(&digits[0..2])?, value(&digits[2..4])?, value(&digits[4..6])?]),
        _ => None,
    }
}

/// The `#rrggbb` form, as the panel shows a colour.
pub fn hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

pub fn rgb8(colour: Color) -> [u8; 3] {
    let byte = |channel: f32| (channel * 255.0).round() as u8;
    [byte(colour.r), byte(colour.g), byte(colour.b)]
}

fn colour([r, g, b]: [u8; 3]) -> Color {
    Color::from_rgb8(r, g, b)
}

fn luminance(colour: Color) -> f32 {
    let channel = |c: f32| if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
    0.2126 * channel(colour.r) + 0.7152 * channel(colour.g) + 0.0722 * channel(colour.b)
}

/// WCAG 2.x contrast ratio, 1 to 21.
pub fn contrast(a: Color, b: Color) -> f32 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

/// `from` moved `amount` of the way to `to`, channel by channel.
fn mix(from: [u8; 3], to: [u8; 3], amount: f32) -> [u8; 3] {
    let channel = |from: u8, to: u8| (f32::from(from) + (f32::from(to) - f32::from(from)) * amount).round() as u8;
    [channel(from[0], to[0]), channel(from[1], to[1]), channel(from[2], to[2])]
}

/// How far the code background is nudged from the page toward the text, softest last.
/// The first that keeps text and links readable on it wins; none is always readable.
const CODE_NUDGES: [f32; 5] = [0.06, 0.045, 0.03, 0.015, 0.0];
/// How far the muted text is moved from the text toward the page, strongest first.
const MUTED_FADES: [f32; 8] = [0.35, 0.3, 0.25, 0.2, 0.15, 0.1, 0.05, 0.0];
/// The rule is this far from the page toward the text.
const RULE_MIX: f32 = 0.15;

/// The theme colours for a set the reader typed. Everything but the three typed colours
/// is derived, and every pair drawn together meets AA whenever the typed pairs do.
pub fn derive(typed: Colours) -> Theme {
    let Colours { page, text, link } = typed;
    let readable_on =
        |ground: [u8; 3], inks: &[[u8; 3]]| inks.iter().all(|&ink| contrast(colour(ink), colour(ground)) >= AA);
    let code_background = CODE_NUDGES
        .iter()
        .map(|&nudge| mix(page, text, nudge))
        .find(|&ground| readable_on(ground, &[text, link]))
        .unwrap_or(page);
    let muted = MUTED_FADES
        .iter()
        .map(|&fade| mix(text, page, fade))
        .find(|&ink| readable_on(page, &[ink]) && readable_on(code_background, &[ink]))
        .unwrap_or(text);
    let dark = luminance(colour(page)) < luminance(colour(text));
    let [r, g, b] = link;
    Theme {
        background: colour(page),
        ink: colour(text),
        muted: colour(muted),
        link: colour(link),
        code_ink: colour(text),
        code_background: colour(code_background),
        rule: colour(mix(page, text, RULE_MIX)),
        selection: Color::from_rgba8(r, g, b, if dark { 0.30 } else { 0.22 }),
        ..tokens(Palette::Light)
    }
}

/// The set itself if the text and the links can be read on the page.
pub fn check(typed: Colours) -> Result<Colours, Unreadable> {
    let on_page = |ink| contrast(colour(ink), colour(typed.page));
    if on_page(typed.text) < AA {
        Err(Unreadable::Text)
    } else if on_page(typed.link) < AA {
        Err(Unreadable::Link)
    } else {
        Ok(typed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPREAD: [[u8; 3]; 10] = [
        [0xff, 0xff, 0xff],
        [0xfb, 0xfa, 0xf7],
        [0xfd, 0xf6, 0xe3],
        [0x00, 0x00, 0x00],
        [0x1b, 0x1b, 0x1a],
        [0x22, 0x22, 0x22],
        [0x80, 0x80, 0x80],
        [0x33, 0x66, 0xcc],
        [0xff, 0x00, 0x00],
        [0xff, 0xff, 0x00],
    ];

    fn colour([r, g, b]: [u8; 3]) -> Color {
        Color::from_rgb8(r, g, b)
    }

    #[test]
    fn hex_is_accepted_in_both_lengths_and_either_case() {
        assert_eq!(parse_hex("#fdf6e3"), Some([0xfd, 0xf6, 0xe3]));
        assert_eq!(parse_hex("#FDF6E3"), Some([0xfd, 0xf6, 0xe3]));
        assert_eq!(parse_hex("#333"), Some([0x33, 0x33, 0x33]));
        assert_eq!(parse_hex("#0Bf"), Some([0x00, 0xbb, 0xff]));
    }

    #[test]
    fn anything_else_is_not_a_colour() {
        for typed in
            ["red", "#12", "#ggg", "", "#", "fdf6e3", "#fdf6e", "#fdf6e3a", " #333", "#333 ", "#33 3", "#+12", "#é12"]
        {
            assert_eq!(parse_hex(typed), None, "{typed:?}");
        }
    }

    #[test]
    fn colours_print_back_as_the_hex_that_parses_to_them() {
        for rgb in SPREAD {
            assert_eq!(parse_hex(&hex(rgb)), Some(rgb));
        }
        assert_eq!(hex([0x1f, 0x5f, 0xbf]), "#1f5fbf");
    }

    #[test]
    fn contrast_is_the_wcag_ratio() {
        assert!((contrast(colour([0, 0, 0]), colour([255, 255, 255])) - 21.0).abs() < 0.01);
        assert!((contrast(colour([255, 255, 255]), colour([255, 255, 255])) - 1.0).abs() < 0.001);
    }

    #[test]
    fn the_built_in_palettes_are_readable_colours_of_their_own() {
        for palette in [super::super::Palette::Light, super::super::Palette::Dark] {
            let theme = super::super::tokens(palette);
            let typed = Colours { page: rgb8(theme.background), text: rgb8(theme.ink), link: rgb8(theme.link) };
            assert_eq!(check(typed), Ok(typed), "{palette:?}");
        }
    }

    #[test]
    fn a_set_is_readable_when_text_and_link_meet_aa_on_the_page_and_refused_otherwise() {
        for page in SPREAD {
            for text in SPREAD {
                for link in SPREAD {
                    let typed = Colours { page, text, link };
                    let on_page = |ink| contrast(colour(ink), colour(page));
                    let expected = if on_page(text) < AA {
                        Err(Unreadable::Text)
                    } else if on_page(link) < AA {
                        Err(Unreadable::Link)
                    } else {
                        Ok(typed)
                    };
                    assert_eq!(check(typed), expected, "{typed:?}");
                }
            }
        }
    }

    #[test]
    fn every_derived_pair_meets_aa_whenever_the_typed_ones_do() {
        let mut readable = 0;
        for page in SPREAD {
            for text in SPREAD {
                for link in SPREAD {
                    let typed = Colours { page, text, link };
                    if check(typed).is_err() {
                        continue;
                    }
                    readable += 1;
                    let t = derive(typed);
                    assert_eq!((t.background, t.ink, t.link), (colour(page), colour(text), colour(link)));
                    for (name, ink, ground) in [
                        ("ink", t.ink, t.background),
                        ("muted", t.muted, t.background),
                        ("link", t.link, t.background),
                        ("code", t.code_ink, t.code_background),
                        ("link on code", t.link, t.code_background),
                        ("muted on code", t.muted, t.code_background),
                    ] {
                        assert!(contrast(ink, ground) >= AA, "{typed:?} {name}: {}", contrast(ink, ground));
                    }
                }
            }
        }
        assert!(readable > 50, "the spread should include plenty of readable sets, got {readable}");
    }

    #[test]
    fn the_derivation_has_the_shape_of_a_palette() {
        let theme = derive(Colours { page: [0xfd, 0xf6, 0xe3], text: [0x33, 0x33, 0x33], link: [0x0b, 0x5f, 0xa5] });
        assert_eq!(theme.code_ink, theme.ink);
        assert_ne!(theme.muted, theme.ink, "muted is softer than the text where there is room");
        assert_ne!(theme.code_background, theme.background);
        assert_ne!(theme.rule, theme.background);
        assert_eq!(
            (theme.selection.r, theme.selection.g, theme.selection.b),
            (theme.link.r, theme.link.g, theme.link.b)
        );
        assert_eq!(theme.selection.a, 0.22, "light pages select at 22%");
        let dark = derive(Colours { page: [0x1b, 0x1b, 0x1a], text: [0xe4, 0xe2, 0xdc], link: [0x7f, 0xaf, 0xf0] });
        assert_eq!(dark.selection.a, 0.30, "dark pages select at 30%");
    }

    #[test]
    fn the_derivation_changes_only_colours() {
        let typed = Colours { page: [0xfd, 0xf6, 0xe3], text: [0x33, 0x33, 0x33], link: [0x0b, 0x5f, 0xa5] };
        let base = super::super::tokens(super::super::Palette::Light);
        let derived = derive(typed);
        assert_eq!(
            Theme {
                background: base.background,
                ink: base.ink,
                muted: base.muted,
                link: base.link,
                code_ink: base.code_ink,
                code_background: base.code_background,
                rule: base.rule,
                selection: base.selection,
                ..derived
            },
            base
        );
    }
}
