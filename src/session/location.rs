//! Where a document lives.

use std::path::{Path, PathBuf};

use url::Url;

use super::address::decode;

const MARKDOWN_EXTENSIONS: [&str; 2] = ["md", "markdown"];

/// A Markdown document's home: a local file or a web page. Build web locations
/// with `Location::web`, which keeps the fragment out (it lives on `Target`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Location {
    File(PathBuf),
    Web(Url),
}

/// A location plus the in-page anchor to scroll to, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub location: Location,
    pub fragment: Option<String>,
}

impl Location {
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Location::File(path.into())
    }

    pub fn web(mut url: Url) -> Self {
        url.set_fragment(None);
        Location::Web(url)
    }

    pub fn as_file(&self) -> Option<&Path> {
        match self {
            Location::File(path) => Some(path),
            Location::Web(_) => None,
        }
    }

    /// What the address bar shows: the path, or the full URL.
    pub fn address(&self) -> String {
        match self {
            Location::File(path) => path.display().to_string(),
            Location::Web(url) => url.to_string(),
        }
    }

    /// Title of last resort: the file name, the URL's last path segment, or its host.
    pub fn label(&self) -> String {
        let name = match self {
            Location::File(path) => path.file_name().map(|name| name.to_string_lossy().into_owned()),
            Location::Web(url) => url
                .path_segments()
                .and_then(|mut segments| segments.rfind(|segment| !segment.is_empty()))
                .map(decode)
                .or_else(|| url.host_str().map(str::to_string)),
        };
        name.unwrap_or_else(|| self.address())
    }
}

/// A path, or a URL's path with any query removed, that names a Markdown file.
/// A trailing slash names a directory, though `Path` would ignore it.
pub fn is_markdown_name(name: &str) -> bool {
    !name.ends_with(['/', '\\']) && is_markdown(Path::new(name))
}

pub fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| MARKDOWN_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_names_are_recognised_case_insensitively() {
        for name in ["a.md", "A.MARKDOWN", "/x/notes.Md", "dir.d/a.markdown"] {
            assert!(is_markdown_name(name), "{name}");
        }
        for name in ["a.md?x", "a.txt", "md", ".md", "a.md/", "a.mdx"] {
            assert!(!is_markdown_name(name), "{name}");
        }
    }

    #[test]
    fn label_is_the_file_name() {
        assert_eq!(Location::file("/x/notes/a.md").label(), "a.md");
    }

    fn web(url: &str) -> Location {
        Location::web(Url::parse(url).unwrap())
    }

    #[test]
    fn web_locations_have_no_fragment() {
        assert_eq!(web("https://example.com/a.md#intro").address(), "https://example.com/a.md");
        assert_eq!(web("https://example.com/a.md#intro"), web("https://example.com/a.md"));
        assert_ne!(web("https://example.com/a.md"), web("https://example.com/b.md"));
    }

    #[test]
    fn web_addresses_are_the_full_url() {
        assert_eq!(web("HTTP://Example.COM/a.md?v=1").address(), "http://example.com/a.md?v=1");
        assert_eq!(web("https://example.com").address(), "https://example.com/");
    }

    #[test]
    fn web_labels_are_the_last_segment_or_the_host() {
        assert_eq!(web("https://example.com/docs/guide.md").label(), "guide.md");
        assert_eq!(web("https://example.com/docs/my%20notes.md").label(), "my notes.md");
        assert_eq!(web("https://example.com/docs/").label(), "docs");
        assert_eq!(web("https://example.com/").label(), "example.com");
    }

    #[test]
    fn only_file_locations_have_a_path() {
        assert_eq!(Location::file("/x/a.md").as_file(), Some(Path::new("/x/a.md")));
        assert_eq!(web("https://example.com/a.md").as_file(), None);
    }
}
