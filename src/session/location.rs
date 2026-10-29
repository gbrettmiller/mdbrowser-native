//! Where a document lives.

use std::path::{Path, PathBuf};

pub(crate) use super::address::decode;
pub use super::address::parse_address;
pub(super) use super::address::{is_network_path, normalise, parse_file_url, scheme_of, split_reference};
#[cfg(test)]
pub(crate) use super::address::{native, native_url};
pub use super::error::ErrorCode;

const MARKDOWN_EXTENSIONS: [&str; 2] = ["md", "markdown"];

/// A Markdown document's home. Phase 1 knows only local files.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    path: PathBuf,
}

/// A location plus the in-page anchor to scroll to, if any.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub location: Location,
    pub fragment: Option<String>,
}

impl Location {
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Location { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// What the address bar shows.
    pub fn address(&self) -> String {
        self.path.display().to_string()
    }

    /// Title of last resort: the file name.
    pub fn label(&self) -> String {
        self.path.file_name().map_or_else(|| self.address(), |name| name.to_string_lossy().into_owned())
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
}
