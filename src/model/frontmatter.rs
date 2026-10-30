//! YAML frontmatter: the shared Web of Markdown vocabulary. Unknown keys and anything malformed are ignored.
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

/// The shared Web of Markdown vocabulary (ADR-006). Every field is optional, and a value of the
/// wrong kind is dropped on its own. The image fields (`image`, `inlineImage`, `metaImage`) are
/// deliberately not read: the body's own images are the only ones drawn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub author: Option<String>,
    /// As written; turning an RFC 822 date into words is the reader's job, not the parser's.
    pub pub_date: Option<String>,
    pub description: Option<String>,
    pub source: Option<Source>,
    pub url_prev: Option<String>,
    pub url_next: Option<String>,
    pub enclosure: Option<Enclosure>,
}

/// The feed a post came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub url: Option<String>,
    pub title: Option<String>,
}

/// A media file attached to a post. `url` is always an `http` or `https` address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Enclosure {
    pub url: String,
    pub mime: Option<String>,
    pub length: Option<u64>,
}

/// Reads a front matter block. Anything malformed gives the fields it can't read as `None`.
pub fn parse(yaml: &str) -> FrontMatter {
    let Some(document) = YamlLoader::load_from_str(yaml).ok().and_then(|documents| documents.into_iter().next()) else {
        return FrontMatter::default();
    };
    FrontMatter {
        title: text(&document["title"]),
        author: text(&document["author"]),
        pub_date: text(&document["pubDate"]),
        description: text(&document["description"]),
        source: source(&document["source"]),
        url_prev: text(&document["urlPrev"]),
        url_next: text(&document["urlNext"]),
        enclosure: enclosure(&document),
    }
}

/// A non-blank scalar, trimmed. Lists, maps and nulls are not text.
fn text(value: &Yaml) -> Option<String> {
    let text = match value {
        Yaml::String(text) => text.trim().to_string(),
        Yaml::Integer(number) => number.to_string(),
        Yaml::Real(number) => number.clone(),
        _ => return None,
    };
    (!text.is_empty()).then_some(text)
}

fn source(value: &Yaml) -> Option<Source> {
    let (url, title) = (text(&value["url"]), text(&value["title"]));
    (url.is_some() || title.is_some()).then_some(Source { url, title })
}

fn enclosure(document: &Yaml) -> Option<Enclosure> {
    let url = text(&document["enclosure"]).filter(|url| is_web_address(url))?;
    let length = text(&document["enclosureLength"]).and_then(|length| length.parse().ok());
    Some(Enclosure { url, mime: text(&document["enclosureType"]), length })
}

/// `http://` or `https://` followed by something, with no spaces. The opener checks again.
fn is_web_address(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    let rest = lower.strip_prefix("http://").or_else(|| lower.strip_prefix("https://"));
    rest.is_some_and(|rest| !rest.is_empty() && !url.contains(char::is_whitespace))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_scalar_title() {
        assert_eq!(parse("title: Home").title, Some("Home".into()));
        assert_eq!(parse("title: 42\nauthor: x").title, Some("42".into()));
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

    const POST: &str = "title: T\nauthor: Dave Winer\npubDate: Sun, 31 Mar 2024 16:17:30 GMT\n\
        description: A summary\nsource:\n  url: http://scripting.com/rss.xml\n  title: Scripting News\n\
        urlPrev: a.md\nurlNext: ../b.md\nenclosure: http://x.com/s.m4a\nenclosureType: audio/mpeg\n\
        enclosureLength: '6374173'\n";

    #[test]
    fn reads_the_shared_vocabulary() {
        let front = parse(POST);
        assert_eq!(front.title.as_deref(), Some("T"));
        assert_eq!(front.author.as_deref(), Some("Dave Winer"));
        assert_eq!(front.pub_date.as_deref(), Some("Sun, 31 Mar 2024 16:17:30 GMT"));
        assert_eq!(front.description.as_deref(), Some("A summary"));
        assert_eq!(
            front.source,
            Some(Source { url: Some("http://scripting.com/rss.xml".into()), title: Some("Scripting News".into()) })
        );
        assert_eq!(front.url_prev.as_deref(), Some("a.md"));
        assert_eq!(front.url_next.as_deref(), Some("../b.md"));
        assert_eq!(
            front.enclosure,
            Some(Enclosure {
                url: "http://x.com/s.m4a".into(),
                mime: Some("audio/mpeg".into()),
                length: Some(6374173)
            })
        );
    }

    #[test]
    fn unknown_keys_and_image_fields_are_ignored() {
        let front = parse("generator: g\nurlvideo: v\nimage: i\ninlineImage: i\nmetaImage: m\n");
        assert_eq!(front, FrontMatter::default());
    }

    #[test]
    fn drops_values_of_the_wrong_kind_and_keeps_the_rest() {
        let front = parse("author: [a, b]\nurlNext: {x: 1}\ndescription: ''\npubDate: 7\nurlPrev: p.md");
        assert_eq!((front.author, front.url_next, front.description), (None, None, None));
        assert_eq!(front.pub_date.as_deref(), Some("7"));
        assert_eq!(front.url_prev.as_deref(), Some("p.md"));
    }

    #[test]
    fn source_keeps_whichever_half_it_has() {
        assert_eq!(parse("source: {title: S}").source, Some(Source { url: None, title: Some("S".into()) }));
        assert_eq!(parse("source: plain").source, None);
        assert_eq!(parse("source: {}").source, None);
    }

    #[test]
    fn enclosures_are_web_addresses_only() {
        for bad in ["file:///etc/passwd", "javascript:alert(1)", "not a url", "ftp://x/y.mp3", "http://", ""] {
            assert_eq!(parse(&format!("enclosure: '{bad}'")).enclosure, None, "{bad:?}");
        }
        assert!(parse("enclosure: HTTPS://x.com/a.mp3").enclosure.is_some());
    }

    #[test]
    fn enclosure_type_and_length_are_optional_and_length_must_be_a_number() {
        let front = parse("enclosure: http://x/a.mp3\nenclosureLength: lots\nenclosureType: ''");
        assert_eq!(front.enclosure, Some(Enclosure { url: "http://x/a.mp3".into(), mime: None, length: None }));
        assert_eq!(parse("enclosure: http://x/a\nenclosureLength: 12").enclosure.unwrap().length, Some(12));
    }

    #[test]
    fn broken_yaml_is_an_empty_front_matter() {
        for yaml in ["author: [unclosed", "- just a list", "", "plain"] {
            assert_eq!(parse(yaml), FrontMatter::default(), "{yaml:?}");
        }
    }

    #[test]
    fn ignores_missing_blank_or_malformed_titles() {
        assert_eq!(parse("author: x").title, None);
        assert_eq!(parse("title: '   '").title, None);
        assert_eq!(parse("title: [bad").title, None);
        assert_eq!(parse("- just a list").title, None);
        assert_eq!(parse("").title, None);
    }
}
