//! YAML frontmatter. Only `title` matters; anything malformed is ignored.
//! Front matter is only ever the block at the very top of a file, as in Jekyll and Hugo.

use yaml_rust2::{Yaml, YamlLoader};

/// Splits a leading front matter block from the body. The block opens with `---` on the first
/// line, not followed by a blank line (so a leading thematic break stays a rule), and closes with
/// `---` or `...`. Anything else is all body.
pub fn split(source: &str) -> (Option<&str>, &str) {
    let text = source.strip_prefix('\u{feff}').unwrap_or(source);
    let Some(rest) = after_opening_fence(text) else { return (None, source) };
    if rest.lines().next().is_none_or(|line| line.trim().is_empty()) {
        return (None, source);
    }
    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if matches!(line.trim_end(), "---" | "...") {
            return (Some(&rest[..offset]), &rest[offset + line.len()..]);
        }
        offset += line.len();
    }
    (None, source)
}

fn after_opening_fence(text: &str) -> Option<&str> {
    let end = text.find('\n')?;
    (text[..end].trim_end() == "---").then(|| &text[end + 1..])
}

pub fn title(yaml: &str) -> Option<String> {
    let documents = YamlLoader::load_from_str(yaml).ok()?;
    let title = match &documents.first()?["title"] {
        Yaml::String(text) => text.trim().to_string(),
        Yaml::Integer(number) => number.to_string(),
        Yaml::Real(number) => number.clone(),
        _ => return None,
    };
    (!title.is_empty()).then_some(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_scalar_title() {
        assert_eq!(title("title: Home"), Some("Home".into()));
        assert_eq!(title("title: 42\nauthor: x"), Some("42".into()));
    }

    #[test]
    fn splits_a_leading_block_from_the_body() {
        assert_eq!(split("---\ntitle: A\n---\n# Body\n"), (Some("title: A\n"), "# Body\n"));
        assert_eq!(split("---\r\ntitle: A\r\n...\r\nBody"), (Some("title: A\r\n"), "Body"));
        assert_eq!(split("\u{feff}---\ntitle: A\n---\n"), (Some("title: A\n"), ""));
        assert_eq!(split("---\n---\nBody"), (Some(""), "Body"));
        assert_eq!(split("--- \nx\n---  \n"), (Some("x\n"), ""));
    }

    #[test]
    fn leaves_documents_without_a_leading_block_alone() {
        for source in ["# Title\n\n---\ntitle: x\n---\n", "---\n\nNot front matter\n---\n", "---\ntitle: unclosed\n"] {
            assert_eq!(split(source), (None, source), "{source:?}");
        }
    }

    #[test]
    fn ignores_missing_blank_or_malformed_titles() {
        assert_eq!(title("author: x"), None);
        assert_eq!(title("title: '   '"), None);
        assert_eq!(title("title: [bad"), None);
        assert_eq!(title("- just a list"), None);
        assert_eq!(title(""), None);
    }
}
