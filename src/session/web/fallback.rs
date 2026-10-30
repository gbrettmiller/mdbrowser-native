//! Where to look for Markdown when a page answers with something else (ADR-005).
//! Pure: the order of candidates is decided here, `services::web` tries them.

use url::Url;

/// Every address worth trying for `page`, best first: the `Link` header's Markdown
/// alternate (same host only), then the `.md` sibling.
pub fn candidates(page: &Url, link_header: Option<&str>) -> Vec<Url> {
    link_header.and_then(|header| alternate(page, header)).into_iter().chain(siblings(page)).collect()
}

/// The `rel="alternate"; type="text/markdown"` target in a `Link` header, if it
/// stays on the page's own host.
pub fn alternate(page: &Url, link_header: &str) -> Option<Url> {
    link_header.split(',').filter_map(link).find_map(|(target, params)| {
        let wants = |name: &str, value: &str| {
            params
                .iter()
                .any(|(key, found)| key == name && found.split_whitespace().any(|v| v.eq_ignore_ascii_case(value)))
        };
        let to = page.join(target).ok()?;
        (wants("rel", "alternate") && wants("type", "text/markdown") && same_host(page, &to)).then_some(to)
    })
}

/// One link-value: `<target>; key="value"; ...`.
fn link(value: &str) -> Option<(&str, Vec<(String, String)>)> {
    let value = value.trim();
    let (target, rest) = value.strip_prefix('<')?.split_once('>')?;
    let params = rest
        .split(';')
        .filter_map(|param| param.split_once('='))
        .map(|(key, value)| (key.trim().to_ascii_lowercase(), value.trim().trim_matches('"').to_string()))
        .collect();
    Some((target.trim(), params))
}

fn same_host(a: &Url, b: &Url) -> bool {
    a.scheme() == b.scheme() && a.host_str() == b.host_str() && a.port_or_known_default() == b.port_or_known_default()
}

/// `/post` → `/post.md`; `/post/` → `/post.md`, then `/post/index.md`. A last
/// segment with an extension has no sibling.
fn siblings(page: &Url) -> Vec<Url> {
    let path = page.path();
    let with_path = |path: String| {
        let mut to = page.clone();
        to.set_path(&path);
        to.set_query(None);
        to.set_fragment(None);
        to
    };
    let directory = path.ends_with('/');
    let trimmed = path.trim_end_matches('/');
    let last = trimmed.rsplit('/').next().unwrap_or_default();
    match (trimmed.is_empty(), directory, last.contains('.')) {
        (true, _, _) => vec![with_path("/index.md".into())],
        (_, _, true) => vec![],
        (_, true, false) => vec![with_path(format!("{trimmed}.md")), with_path(format!("{trimmed}/index.md"))],
        (_, false, false) => vec![with_path(format!("{trimmed}.md"))],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    fn texts(urls: Vec<Url>) -> Vec<String> {
        urls.into_iter().map(String::from).collect()
    }

    #[test]
    fn a_page_without_a_slash_tries_its_md_sibling() {
        assert_eq!(
            texts(candidates(&url("https://every.to/context-window/post"), None)),
            ["https://every.to/context-window/post.md"]
        );
    }

    #[test]
    fn a_trailing_slash_tries_the_sibling_then_index() {
        assert_eq!(
            texts(candidates(&url("https://a.example/post/"), None)),
            ["https://a.example/post.md", "https://a.example/post/index.md"]
        );
    }

    #[test]
    fn the_site_root_tries_index() {
        assert_eq!(texts(candidates(&url("https://a.example/"), None)), ["https://a.example/index.md"]);
    }

    #[test]
    fn query_and_fragment_are_not_carried_to_the_sibling() {
        assert_eq!(texts(candidates(&url("https://a.example/post?utm=1#top"), None)), ["https://a.example/post.md"]);
    }

    #[test]
    fn a_page_with_an_extension_has_no_sibling() {
        assert!(candidates(&url("https://a.example/post.html"), None).is_empty());
    }

    #[test]
    fn the_link_alternate_comes_before_the_sibling() {
        let header = r#"</raw/post.md>; rel="alternate"; type="text/markdown""#;
        assert_eq!(
            texts(candidates(&url("https://a.example/post"), Some(header))),
            ["https://a.example/raw/post.md", "https://a.example/post.md"]
        );
    }

    #[test]
    fn link_values_are_found_among_several_and_resolved_against_the_page() {
        let header = r#"<https://a.example/feed>; rel="alternate"; type="application/rss+xml", <p.md>; rel=alternate; type=text/markdown"#;
        assert_eq!(alternate(&url("https://a.example/dir/post"), header), Some(url("https://a.example/dir/p.md")));
    }

    #[test]
    fn only_markdown_alternates_count() {
        let rss = r#"</feed>; rel="alternate"; type="application/rss+xml""#;
        let canonical = r#"</x.md>; rel="canonical"; type="text/markdown""#;
        let untyped = r#"</x.md>; rel="alternate""#;
        for header in [rss, canonical, untyped, "garbage", ""] {
            assert_eq!(alternate(&url("https://a.example/post"), header), None, "{header}");
        }
    }

    #[test]
    fn an_alternate_on_another_host_scheme_or_port_is_ignored() {
        for target in ["https://evil.example/p.md", "http://a.example/p.md", "https://a.example:8443/p.md"] {
            let header = format!(r#"<{target}>; rel="alternate"; type="text/markdown""#);
            assert_eq!(alternate(&url("https://a.example/post"), &header), None, "{target}");
        }
    }

    #[test]
    fn rel_may_list_several_values() {
        let header = r#"</p.md>; rel="nofollow alternate"; type="text/markdown""#;
        assert_eq!(alternate(&url("https://a.example/post"), header), Some(url("https://a.example/p.md")));
    }
}
