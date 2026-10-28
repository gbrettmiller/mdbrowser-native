//! GitHub-style heading anchors, so `#fragment` links written for GitHub work here.

use std::collections::HashMap;

/// Lowercase, drop punctuation and symbols, spaces become hyphens.
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

/// Hands out unique slugs in document order: `intro`, `intro-1`, `intro-2`.
#[derive(Debug, Default)]
pub struct Slugger {
    seen: HashMap<String, usize>,
}

impl Slugger {
    pub fn slug(&mut self, text: &str) -> String {
        let base = slugify(text);
        let count = self.seen.entry(base.clone()).or_insert(0);
        let slug = if *count == 0 { base } else { format!("{base}-{count}") };
        *count += 1;
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_github_rules() {
        assert_eq!(slugify("Hello, World!"), "hello-world");
        assert_eq!(slugify("API v2.0 — notes"), "api-v20--notes");
        assert_eq!(slugify("snake_case-name"), "snake_case-name");
        assert_eq!(slugify("Café Über"), "café-über");
    }

    #[test]
    fn duplicates_get_numbered_suffixes() {
        let mut slugger = Slugger::default();
        let slugs: Vec<_> = ["Intro", "Intro", "Other", "Intro"].iter().map(|t| slugger.slug(t)).collect();
        assert_eq!(slugs, ["intro", "intro-1", "other", "intro-2"]);
    }
}
