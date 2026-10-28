//! Bundled typefaces. Host font metadata is unreliable (spike finding 1), so the
//! reading faces ship inside the binary and win over any host copy.

use std::sync::{Arc, Once};

use iced::advanced::graphics::text::cosmic_text::fontdb::{FaceInfo, Source, Style, Weight};
use iced::advanced::graphics::text::font_system;

use super::{MONO_FAMILY, ReadingFont};

/// A bundled file and the variant it is *meant* to be. Some upstream files
/// misdeclare this (iA Writer Quattro S Bold says weight 400), so the
/// declared variant wins over the file's own metadata.
type Bundled = (&'static [u8], Weight, Style);

macro_rules! bundled {
    ($($path:literal => $weight:ident $style:ident),* $(,)?) => {
        [$((
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/", $path)).as_slice(),
            Weight::$weight,
            Style::$style,
        )),*]
    };
}

const FILES: [Bundled; 20] = bundled![
    "literata/Literata-Regular.ttf" => NORMAL Normal,
    "literata/Literata-Bold.ttf" => BOLD Normal,
    "literata/Literata-Italic.ttf" => NORMAL Italic,
    "literata/Literata-BoldItalic.ttf" => BOLD Italic,
    "source-serif-4/SourceSerif4-Regular.ttf" => NORMAL Normal,
    "source-serif-4/SourceSerif4-Bold.ttf" => BOLD Normal,
    "source-serif-4/SourceSerif4-It.ttf" => NORMAL Italic,
    "source-serif-4/SourceSerif4-BoldIt.ttf" => BOLD Italic,
    "ia-writer-quattro/iAWriterQuattroS-Regular.ttf" => NORMAL Normal,
    "ia-writer-quattro/iAWriterQuattroS-Bold.ttf" => BOLD Normal,
    "ia-writer-quattro/iAWriterQuattroS-Italic.ttf" => NORMAL Italic,
    "ia-writer-quattro/iAWriterQuattroS-BoldItalic.ttf" => BOLD Italic,
    "inter/Inter-Regular.ttf" => NORMAL Normal,
    "inter/Inter-Bold.ttf" => BOLD Normal,
    "inter/Inter-Italic.ttf" => NORMAL Italic,
    "inter/Inter-BoldItalic.ttf" => BOLD Italic,
    "jetbrains-mono/JetBrainsMono-Regular.ttf" => NORMAL Normal,
    "jetbrains-mono/JetBrainsMono-Bold.ttf" => BOLD Normal,
    "jetbrains-mono/JetBrainsMono-Italic.ttf" => NORMAL Italic,
    "jetbrains-mono/JetBrainsMono-BoldItalic.ttf" => BOLD Italic,
];

fn is_bundled_family(name: &str) -> bool {
    name == MONO_FAMILY || ReadingFont::ALL.iter().any(|font| font.family() == name)
}

/// Loads the bundled faces into iced's shared font system, first evicting any
/// host faces with the same family names. Safe to call repeatedly.
pub fn install() {
    static INSTALLED: Once = Once::new();
    INSTALLED.call_once(|| {
        let mut system = font_system().write().expect("font system lock");
        let db = system.raw().db_mut();
        let host_copies: Vec<_> = db
            .faces()
            .filter(|face| !matches!(face.source, Source::Binary(_)))
            .filter(|face| face.families.iter().any(|(name, _)| is_bundled_family(name)))
            .map(|face| face.id)
            .collect();
        host_copies.into_iter().for_each(|id| db.remove_face(id));
        for (bytes, weight, style) in FILES {
            for id in db.load_font_source(Source::Binary(Arc::new(bytes))) {
                let Some(face) = db.face(id).cloned() else { continue };
                if (face.weight, face.style) != (weight, style) {
                    db.remove_face(id);
                    db.push_face_info(FaceInfo { weight, style, ..face });
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{MONO_FAMILY, ReadingFont};
    use iced::advanced::graphics::text::cosmic_text::fontdb::{Family, Query, Source, Stretch, Style, Weight};
    use iced::advanced::graphics::text::font_system;

    const VARIANTS: [(Weight, Style); 4] = [
        (Weight::NORMAL, Style::Normal),
        (Weight::BOLD, Style::Normal),
        (Weight::NORMAL, Style::Italic),
        (Weight::BOLD, Style::Italic),
    ];

    #[test]
    fn every_family_resolves_to_its_own_bundled_face() {
        install();
        let mut system = font_system().write().expect("font system lock");
        let db = system.raw().db();
        let families = ReadingFont::ALL.map(ReadingFont::family).into_iter().chain([MONO_FAMILY]);
        for family in families {
            for (weight, style) in VARIANTS {
                let query = Query { families: &[Family::Name(family)], weight, stretch: Stretch::Normal, style };
                let face = db.query(&query).and_then(|id| db.face(id));
                let face = face.unwrap_or_else(|| panic!("{family} {weight:?} {style:?}: no face"));
                assert!(face.families.iter().any(|(name, _)| name == family), "{family}: got {:?}", face.families);
                assert_eq!((face.weight, face.style), (weight, style), "{family} {weight:?} {style:?}");
                assert!(matches!(face.source, Source::Binary(_)), "{family} {weight:?} {style:?} came from the host");
            }
        }
    }
}
