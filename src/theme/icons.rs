//! The chrome's icons (ADR-016): characters in Font Awesome Free's bundled Solid and Regular
//! fonts, which share one family and differ by weight.

use iced::Font;
use iced::font::{Family, Weight};

pub const ICON_FAMILY: &str = "Font Awesome 7 Free";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Back,
    Forward,
    Reload,
    /// The source toggle in the page view: shows the Markdown.
    Source,
    /// The source toggle in the source view: back to the page.
    Page,
    Home,
    /// The bookmark toggle when the page isn't kept: an outline star.
    Bookmark,
    /// The bookmark toggle when the page is kept: a filled star.
    Bookmarked,
    Bookmarks,
    History,
    Settings,
    MoveUp,
    MoveDown,
    Remove,
}

impl Icon {
    pub const ALL: [Icon; 14] = [
        Self::Back,
        Self::Forward,
        Self::Reload,
        Self::Source,
        Self::Page,
        Self::Home,
        Self::Bookmark,
        Self::Bookmarked,
        Self::Bookmarks,
        Self::History,
        Self::Settings,
        Self::MoveUp,
        Self::MoveDown,
        Self::Remove,
    ];

    /// The icon's character, by its Font Awesome name.
    pub const fn glyph(self) -> char {
        match self {
            Self::Back => '\u{f060}',                        // arrow-left
            Self::Forward => '\u{f061}',                     // arrow-right
            Self::Reload => '\u{f01e}',                      // arrow-rotate-right
            Self::Source => '\u{f121}',                      // code
            Self::Page => '\u{f15c}',                        // file-lines
            Self::Home => '\u{f015}',                        // house
            Self::Bookmark | Self::Bookmarked => '\u{f005}', // star
            Self::Bookmarks => '\u{f02e}',                   // bookmark
            Self::History => '\u{f1da}',                     // clock-rotate-left
            Self::Settings => '\u{f013}',                    // gear
            Self::MoveUp => '\u{f062}',                      // arrow-up
            Self::MoveDown => '\u{f063}',                    // arrow-down
            Self::Remove => '\u{f00d}',                      // xmark
        }
    }

    /// Solid (900) for every icon but the outline star, which is Regular (400).
    pub const fn weight(self) -> Weight {
        match self {
            Self::Bookmark => Weight::Normal,
            _ => Weight::Black,
        }
    }

    pub const fn font(self) -> Font {
        Font { family: Family::Name(ICON_FAMILY), weight: self.weight(), ..Font::DEFAULT }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::fonts::install;
    use iced::advanced::graphics::text::cosmic_text::fontdb::{self, Query, Source, Stretch, Style};
    use iced::advanced::graphics::text::font_system;

    const fn fontdb_weight(weight: Weight) -> fontdb::Weight {
        match weight {
            Weight::Normal => fontdb::Weight::NORMAL,
            _ => fontdb::Weight::BLACK,
        }
    }

    #[test]
    fn every_icon_is_drawn_by_its_bundled_font() {
        install();
        let mut guard = font_system().write().expect("font system lock");
        let system = guard.raw();
        for icon in Icon::ALL {
            let weight = fontdb_weight(icon.weight());
            let query = Query {
                families: &[fontdb::Family::Name(ICON_FAMILY)],
                weight,
                stretch: Stretch::Normal,
                style: Style::Normal,
            };
            let id = system.db().query(&query).unwrap_or_else(|| panic!("{icon:?}: no face"));
            let face = system.db().face(id).expect("face");
            assert_eq!(face.weight, weight, "{icon:?}");
            assert!(matches!(face.source, Source::Binary(_)), "{icon:?} came from the host");
            let font = system.get_font(id, weight).expect("font loads");
            assert_ne!(font.as_swash().charmap().map(icon.glyph()), 0, "{icon:?} has no glyph");
        }
    }

    #[test]
    fn the_star_is_an_outline_until_the_page_is_kept() {
        assert_eq!(Icon::Bookmark.glyph(), Icon::Bookmarked.glyph());
        assert_ne!(Icon::Bookmark.font(), Icon::Bookmarked.font());
    }
}
