//! Addresses: what the reader types, and where a document lives.

use std::path::{Path, PathBuf};

const MARKDOWN_EXTENSIONS: [&str; 2] = ["md", "markdown"];

/// Why an address or load failed. `key()` names the user-facing string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Empty,
    NotAbsolute,
    UnsupportedScheme,
    NotMarkdown,
    NotFound,
    Unreadable,
}

impl ErrorCode {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::NotAbsolute => "notAbsolute",
            Self::UnsupportedScheme => "unsupportedScheme",
            Self::NotMarkdown => "notMarkdown",
            Self::NotFound => "notFound",
            Self::Unreadable => "unreadable",
        }
    }
}

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

pub fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| MARKDOWN_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
}

/// `scheme:` per RFC 3986, lowercased. Single letters are Windows drives, not schemes.
pub(super) fn scheme_of(reference: &str) -> Option<String> {
    let (scheme, _) = reference.split_once(':')?;
    let mut chars = scheme.chars();
    let valid = scheme.len() > 1
        && chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c));
    valid.then(|| scheme.to_ascii_lowercase())
}

/// Percent-decodes; malformed or non-UTF-8 input is returned unchanged.
pub(super) fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = (bytes[i] == b'%')
            .then(|| text.get(i + 1..i + 3))
            .flatten()
            .and_then(|pair| u8::from_str_radix(pair, 16).ok());
        match hex {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| text.to_string())
}

/// `path#fragment` → (decoded path, decoded non-empty fragment).
pub(super) fn split_reference(reference: &str) -> (String, Option<String>) {
    let (path, fragment) = reference.split_once('#').unwrap_or((reference, ""));
    let fragment = decode(fragment);
    (decode(path), (!fragment.is_empty()).then_some(fragment))
}

/// `file://[localhost]/abs/path[#fragment]` → (path, fragment). Other hosts aren't local.
pub(super) fn parse_file_url(url: &str) -> Option<(String, Option<String>)> {
    let rest = url.get("file://".len()..)?;
    let slash = rest.find('/')?;
    let host = &rest[..slash];
    (host.is_empty() || host.eq_ignore_ascii_case("localhost")).then(|| split_reference(&rest[slash..]))
}

/// `//host/…` or `\\\\host\\…` in any mix of slashes: a network share, not a local file.
/// Opening one can leak credentials to the host (SMB on Windows), so documents never may.
pub(super) fn is_network_path(path: &str) -> bool {
    let mut chars = path.chars();
    matches!((chars.next(), chars.next()), (Some('/' | '\\'), Some('/' | '\\')))
}

/// Lexically resolves `.` and `..` (the target may not exist yet).
pub(super) fn normalise(path: &Path) -> PathBuf {
    path.components().fold(PathBuf::new(), |mut acc, part| {
        match part {
            std::path::Component::ParentDir => {
                acc.pop();
            }
            std::path::Component::CurDir => {}
            other => acc.push(other),
        }
        acc
    })
}

fn expand_home(path: &str, home: &Path) -> PathBuf {
    match path.strip_prefix('~') {
        Some("") => home.to_path_buf(),
        Some(rest) if rest.starts_with('/') => home.join(&rest[1..]),
        _ => PathBuf::from(path),
    }
}

/// Validation order: empty → scheme → absolute → extension. Nothing touches the disk.
pub fn parse_address(input: &str, home: &Path) -> Result<Target, ErrorCode> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ErrorCode::Empty);
    }
    let (path, fragment) = match scheme_of(input).as_deref() {
        Some("file") => parse_file_url(input).ok_or(ErrorCode::NotAbsolute)?,
        Some(_) => return Err(ErrorCode::UnsupportedScheme),
        None => {
            let (path, fragment) = split_reference(input);
            (expand_home(&path, home).display().to_string(), fragment)
        }
    };
    if is_network_path(&path) {
        return Err(ErrorCode::NotAbsolute);
    }
    let path = Path::new(&path);
    if !path.is_absolute() {
        return Err(ErrorCode::NotAbsolute);
    }
    let path = normalise(path);
    if !is_markdown(&path) {
        return Err(ErrorCode::NotMarkdown);
    }
    Ok(Target { location: Location::file(path), fragment })
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = "/home/reader";

    fn parse(input: &str) -> Result<(String, Option<String>), ErrorCode> {
        parse_address(input, Path::new(HOME)).map(|t| (t.location.address(), t.fragment))
    }

    fn ok(path: &str, fragment: Option<&str>) -> Result<(String, Option<String>), ErrorCode> {
        Ok((path.to_string(), fragment.map(str::to_string)))
    }

    #[test]
    fn rejects_in_validation_order_without_touching_disk() {
        assert_eq!(parse(""), Err(ErrorCode::Empty));
        assert_eq!(parse("   "), Err(ErrorCode::Empty));
        assert_eq!(parse("https://example.com/readme.md"), Err(ErrorCode::UnsupportedScheme));
        assert_eq!(parse("ftp://x/a.md"), Err(ErrorCode::UnsupportedScheme));
        assert_eq!(parse("notes/a.md"), Err(ErrorCode::NotAbsolute));
        assert_eq!(parse("C:\\notes\\a.md"), Err(ErrorCode::NotAbsolute));
        assert_eq!(parse("/x/data.txt"), Err(ErrorCode::NotMarkdown));
        assert_eq!(parse("~"), Err(ErrorCode::NotMarkdown));
    }

    #[test]
    fn accepts_absolute_home_relative_and_file_url_forms() {
        assert_eq!(parse("/x/a.md"), ok("/x/a.md", None));
        assert_eq!(parse("  /x/a.markdown "), ok("/x/a.markdown", None));
        assert_eq!(parse("/x/A.MD"), ok("/x/A.MD", None));
        assert_eq!(parse("~/notes/a.md"), ok("/home/reader/notes/a.md", None));
        assert_eq!(parse("file:///x/a.md"), ok("/x/a.md", None));
        assert_eq!(parse("file://localhost/x/a.md"), ok("/x/a.md", None));
    }

    #[test]
    fn file_urls_need_a_local_host_and_are_percent_decoded() {
        assert_eq!(parse("file://server/x/a.md"), Err(ErrorCode::NotAbsolute));
        assert_eq!(parse("file:///x/my%20notes.md"), ok("/x/my notes.md", None));
    }

    #[test]
    fn fragments_split_off_and_paths_normalise() {
        assert_eq!(parse("file:///x/a.md#intro"), ok("/x/a.md", Some("intro")));
        assert_eq!(parse("/x/a.md#caf%C3%A9"), ok("/x/a.md", Some("café")));
        assert_eq!(parse("/x/../y/./a.md"), ok("/y/a.md", None));
    }

    #[test]
    fn network_paths_are_not_local_files() {
        for input in ["//server/share/a.md", "\\\\server\\share\\a.md", "file:////server/share/a.md", "/\\server/a.md"]
        {
            assert_eq!(parse(input), Err(ErrorCode::NotAbsolute), "{input}");
        }
    }

    #[test]
    fn label_is_the_file_name() {
        assert_eq!(Location::file("/x/notes/a.md").label(), "a.md");
    }
}
