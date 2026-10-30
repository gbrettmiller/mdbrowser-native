//! The text sizes of the window around the page: labels, panels, notes. They do not follow
//! the reader's text size (ADR-008): the panels and toolbar are a fixed width, and the
//! reader's size is for the document. None is smaller than `NOTE`.

/// The smallest chrome text: the document title, the status line, a visit's address.
pub const NOTE: f32 = 13.0;
/// Settings labels, buttons and field text.
pub const LABEL: f32 = 14.0;
/// Messages, hints, and a visit's title.
pub const BODY: f32 = 16.0;
/// A panel's heading.
pub const HEADING: f32 = 18.0;
/// The headline of an error.
pub const HEADLINE: f32 = 20.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_chrome_text_is_smaller_than_13_and_each_step_is_larger_than_the_last() {
        let scale = [NOTE, LABEL, BODY, HEADING, HEADLINE];
        assert!(scale[0] >= 13.0);
        assert!(scale.windows(2).all(|pair| pair[0] < pair[1]), "{scale:?}");
    }

    /// Every `.size(...)` in `ui/` names a token, so a small literal can't creep back in.
    #[test]
    fn the_ui_sets_no_literal_text_size() {
        fn sources(dir: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).expect("read ui dir").flatten() {
                let path = entry.path();
                if path.is_dir() {
                    sources(&path, found);
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    found.push(path);
                }
            }
        }
        let mut files = Vec::new();
        sources(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ui"), &mut files);
        assert!(files.len() > 10, "found the ui sources");
        let literal = |line: &&str| {
            line.match_indices(".size(").any(|(at, _)| line[at + 6..].starts_with(|c: char| c.is_ascii_digit()))
        };
        let offenders: Vec<_> = files
            .iter()
            .flat_map(|file| {
                let source = std::fs::read_to_string(file).expect("read source");
                source
                    .lines()
                    .filter(literal)
                    .map(|line| format!("{}: {}", file.display(), line.trim()))
                    .collect::<Vec<_>>()
            })
            .collect();
        assert!(offenders.is_empty(), "{offenders:#?}");
    }
}
