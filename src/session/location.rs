//! Where a document lives.

use std::path::{Path, PathBuf};

use url::Url;

use super::address::decode;

const MARKDOWN_EXTENSIONS: [&str; 2] = ["md", "markdown"];

/// The only schemes fetched or shown in the app.
pub const WEB_SCHEMES: [&str; 2] = ["http", "https"];
pub const FILE_SCHEME: &str = "file";
/// The built-in About page: its address, the scheme of that address, and its title of last resort.
pub const ABOUT_ADDRESS: &str = "about:mdbrowser";
pub const ABOUT_SCHEME: &str = "about";
const ABOUT_LABEL: &str = "About mdBrowser";

/// The URL without a user name or password, which are never sent anywhere.
pub fn without_credentials(mut url: Url) -> Url {
    // Infallible for http(s) URLs, which always have a host.
    let _ = url.set_username("");
    let _ = url.set_password(None);
    url
}

/// A Markdown document's home: a local file or a web page. Build web locations
/// with `Location::web`, which keeps the fragment out (it lives on `Target`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Location {
    File(PathBuf),
    Web(Url),
    /// The built-in welcome page. It has a blank address and isn't a file or a URL.
    Welcome,
    /// The built-in About page (ADR-013), at `about:mdbrowser`. Not a file or a URL either.
    About,
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
            Location::Web(_) | Location::Welcome | Location::About => None,
        }
    }

    /// A page bundled in the app: no file, no URL, and no network.
    pub fn is_builtin(&self) -> bool {
        matches!(self, Location::Welcome | Location::About)
    }

    /// Whether visiting it adds a History entry: a built-in page is not a place to revisit.
    pub fn records_history(&self) -> bool {
        !self.is_builtin()
    }

    pub fn is_web(&self) -> bool {
        matches!(self, Location::Web(_))
    }

    /// A web page's host name; files have none.
    pub fn host(&self) -> Option<&str> {
        match self {
            Location::File(_) | Location::Welcome | Location::About => None,
            Location::Web(url) => url.host_str(),
        }
    }

    /// What the address bar shows: the path, or the full URL.
    pub fn address(&self) -> String {
        match self {
            Location::File(path) => path.display().to_string(),
            Location::Web(url) => url.to_string(),
            Location::Welcome => String::new(),
            Location::About => ABOUT_ADDRESS.to_string(),
        }
    }

    /// Title of last resort: the file name, the URL's last path segment, or its host.
    pub fn label(&self) -> String {
        let name = match self {
            Location::Welcome => None,
            Location::About => Some(ABOUT_LABEL.to_string()),
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
    fn the_welcome_page_has_no_label_of_its_own() {
        assert_eq!(Location::Welcome.label(), "");
    }

    #[test]
    fn the_about_page_has_its_own_address_and_label_and_is_neither_file_nor_web() {
        assert_eq!(Location::About.address(), "about:mdbrowser");
        assert_eq!(Location::About.label(), "About mdBrowser");
        assert_eq!((Location::About.host(), Location::About.as_file(), Location::About.is_web()), (None, None, false));
    }

    #[test]
    fn built_in_pages_make_no_history_entry_and_show_no_local_images() {
        for page in [Location::Welcome, Location::About] {
            assert!(page.is_builtin() && !page.records_history(), "{page:?}");
        }
        assert!(Location::file("/x/a.md").records_history() && !Location::file("/x/a.md").is_builtin());
        assert!(web("https://example.com/a.md").records_history());
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
    fn only_web_locations_have_a_host() {
        assert_eq!(web("https://example.com/a.md").host(), Some("example.com"));
        assert_eq!(web("http://127.0.0.1:8080/a.md").host(), Some("127.0.0.1"));
        assert_eq!(Location::file("/x/a.md").host(), None);
        assert!(web("https://example.com/a.md").is_web() && !Location::file("/x/a.md").is_web());
    }

    #[test]
    fn only_file_locations_have_a_path() {
        assert_eq!(Location::file("/x/a.md").as_file(), Some(Path::new("/x/a.md")));
        assert_eq!(web("https://example.com/a.md").as_file(), None);
    }
}
