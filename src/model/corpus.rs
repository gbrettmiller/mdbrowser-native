//! Front matter over the corpus: real posts from the Web of Markdown archive and edge cases.

use super::parse;

/// (path under corpus/markdownweb, source, front matter title)
const POSTS: [(&str, &str, Option<&str>); 12] = [
    ("2024/03/31/a164241", include_str!("../../corpus/markdownweb/2024/03/31/a164241.md"), None),
    ("2024/03/05/200550", include_str!("../../corpus/markdownweb/2024/03/05/200550.md"), Some("How we learn to art")),
    ("2024/03/31/a161730", include_str!("../../corpus/markdownweb/2024/03/31/a161730.md"), None),
    (
        "2024/03/22/025201",
        include_str!("../../corpus/markdownweb/2024/03/22/025201.md"),
        Some("Everyone's driving to Vermont"),
    ),
    (
        "2024/03/15/015719",
        include_str!("../../corpus/markdownweb/2024/03/15/015719.md"),
        Some("Four things about threads.net"),
    ),
    ("2024/01/21/a194147", include_str!("../../corpus/markdownweb/2024/01/21/a194147.md"), None),
    (
        "2024/03/01/203935",
        include_str!("../../corpus/markdownweb/2024/03/01/203935.md"),
        Some("I see the bright lights of Memphis"),
    ),
    ("2024/03/12/120111", include_str!("../../corpus/markdownweb/2024/03/12/120111.md"), Some("AI and the NYT")),
    ("2026/10/22/a160031", include_str!("../../corpus/markdownweb/2026/10/22/a160031.md"), None),
    ("2022/05/21/a020940", include_str!("../../corpus/markdownweb/2022/05/21/a020940.md"), None),
    ("2022/03/05/a163449", include_str!("../../corpus/markdownweb/2022/03/05/a163449.md"), None),
    ("2024/03/17/a194939", include_str!("../../corpus/markdownweb/2024/03/17/a194939.md"), None),
];

#[test]
fn archive_posts_take_their_title_from_front_matter_or_the_file_name() {
    for (path, source, title) in POSTS {
        let name = path.rsplit('/').next().unwrap();
        assert_eq!(parse(source).title(name), title.unwrap_or(name), "{path}");
    }
}

#[test]
fn archive_front_matter_never_shows_as_text() {
    for (path, source, _) in POSTS {
        let text = parse(source).plain_text();
        for key in ["author:", "pubDate:", "generator:", "urlPrev:", "Scripting News"] {
            assert!(!text.contains(key), "{path} shows {key}");
        }
    }
}

#[test]
fn edge_cases_resolve_titles_without_failing() {
    let cases = [
        (include_str!("../../corpus/front-matter/title-over-heading.md"), "The front matter title"),
        (include_str!("../../corpus/front-matter/malformed.md"), "Heading used as the title"),
        (include_str!("../../corpus/front-matter/unknown-keys.md"), "Unknown keys are ignored"),
        (include_str!("../../corpus/front-matter/empty.md"), "An empty front matter block"),
        (include_str!("../../corpus/front-matter/not-at-top.md"), "Front matter only counts at the top"),
    ];
    for (source, title) in cases {
        assert_eq!(parse(source).title("fallback"), title);
    }
}

#[test]
fn unknown_keys_are_not_shown() {
    let text = parse(include_str!("../../corpus/front-matter/unknown-keys.md")).plain_text();
    assert!(!text.contains("readingTime") && !text.contains("deeply"));
}

#[test]
fn front_matter_fences_never_render_as_rules() {
    for source in [include_str!("../../corpus/front-matter/empty.md"), POSTS[0].1] {
        assert!(!parse(source).blocks.contains(&super::Block::Rule));
    }
}
