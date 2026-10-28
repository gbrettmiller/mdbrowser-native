//! YAML frontmatter. Only `title` matters; anything malformed is ignored.

use yaml_rust2::{Yaml, YamlLoader};

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
    fn ignores_missing_blank_or_malformed_titles() {
        assert_eq!(title("author: x"), None);
        assert_eq!(title("title: '   '"), None);
        assert_eq!(title("title: [bad"), None);
        assert_eq!(title("- just a list"), None);
        assert_eq!(title(""), None);
    }
}
