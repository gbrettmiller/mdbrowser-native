//! Parsing what the reader types into a target. Nothing here touches the disk.

use std::path::{Path, PathBuf};

use url::Url;

use super::error::ErrorCode;
use super::location::{FILE_SCHEME, Location, Target, WEB_SCHEMES, is_markdown, without_credentials};

/// `scheme:` per RFC 3986, lowercased. Single letters are Windows drives, not schemes.
pub(super) fn scheme_of(reference: &str) -> Option<String> {
    let (scheme, _) = reference.split_once(':')?;
    let mut chars = scheme.chars();
    let valid = scheme.len() > 1
        && chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c));
    valid.then(|| scheme.to_ascii_lowercase())
}

/// Whether `input` starts with a URL scheme (`https:`, `file:`), not a drive letter.
pub fn has_scheme(input: &str) -> bool {
    scheme_of(input.trim()).is_some()
}

/// Percent-decodes; malformed or non-UTF-8 input is returned unchanged.
pub(crate) fn decode(text: &str) -> String {
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
    let prefix = url.get(.."file://".len())?;
    if !prefix.eq_ignore_ascii_case("file://") {
        return None; // `file:/x` or `file:x`: not the form we read
    }
    let rest = &url["file://".len()..];
    let slash = rest.find('/')?;
    let host = &rest[..slash];
    (host.is_empty() || host.eq_ignore_ascii_case("localhost")).then(|| {
        let (path, fragment) = split_reference(&rest[slash..]);
        (file_url_path(path, cfg!(windows)), fragment)
    })
}

/// On Windows a file URL's path carries its drive after a slash (`/C:/x`); the path is `C:/x`.
fn file_url_path(path: String, windows: bool) -> String {
    let bytes = path.as_bytes();
    let has_drive = bytes.len() >= 3 && bytes[0] == b'/' && bytes[1].is_ascii_alphabetic() && bytes[2] == b':';
    if windows && has_drive { path[1..].to_string() } else { path }
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

pub(super) fn expand_home(path: &str, home: &Path) -> PathBuf {
    match path.strip_prefix('~') {
        Some("") => home.to_path_buf(),
        Some(rest) if rest.starts_with('/') => home.join(&rest[1..]),
        _ => PathBuf::from(path),
    }
}

/// An http(s) URL as a target: credentials dropped, the fragment decoded onto the target.
pub(super) fn web_target(url: Url) -> Target {
    let fragment = url.fragment().map(decode).filter(|fragment| !fragment.is_empty());
    Target { location: Location::web(without_credentials(url)), fragment }
}

/// An http(s) address. Any extension may be Markdown (the server says), so only
/// a host is required.
fn parse_web_url(input: &str) -> Result<Target, ErrorCode> {
    Url::parse(input).map(web_target).map_err(|_| ErrorCode::NotAbsolute)
}

/// Validation order: empty → scheme → absolute → extension. Nothing touches the disk.
pub fn parse_address(input: &str, home: &Path) -> Result<Target, ErrorCode> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ErrorCode::Empty);
    }
    let (path, fragment) = match scheme_of(input).as_deref() {
        Some(scheme) if WEB_SCHEMES.contains(&scheme) => return parse_web_url(input),
        Some(FILE_SCHEME) => parse_file_url(input).ok_or(ErrorCode::NotAbsolute)?,
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

/// Test paths are written Unix-style; on Windows they live on drive C:.
#[cfg(test)]
pub(crate) fn native(unix: &str) -> String {
    if cfg!(windows) { format!("C:{}", unix.replace('/', "\\")) } else { unix.to_string() }
}

/// A `file://` URL for a Unix-style test path, on drive C: under Windows.
#[cfg(test)]
pub(crate) fn native_url(unix: &str) -> String {
    if cfg!(windows) { format!("file:///C:{unix}") } else { format!("file://{unix}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOME: &str = "/home/reader";

    fn parse(input: &str) -> Result<(String, Option<String>), ErrorCode> {
        parse_address(input, Path::new(&native(HOME))).map(|t| (t.location.address(), t.fragment))
    }

    fn ok(path: &str, fragment: Option<&str>) -> Result<(String, Option<String>), ErrorCode> {
        Ok((native(path), fragment.map(str::to_string)))
    }

    #[test]
    fn rejects_in_validation_order_without_touching_disk() {
        assert_eq!(parse(""), Err(ErrorCode::Empty));
        assert_eq!(parse("   "), Err(ErrorCode::Empty));
        assert_eq!(parse("ftp://x/a.md"), Err(ErrorCode::UnsupportedScheme));
        assert_eq!(parse("notes/a.md"), Err(ErrorCode::NotAbsolute));
        assert_eq!(parse(&native("/x/data.txt")), Err(ErrorCode::NotMarkdown));
        assert_eq!(parse("~"), Err(ErrorCode::NotMarkdown));
    }

    #[test]
    fn web_addresses_are_accepted_whatever_their_extension() {
        assert_eq!(parse("https://example.com/readme.md"), Ok(("https://example.com/readme.md".into(), None)));
        assert_eq!(parse("HTTP://X.ORG/a"), Ok(("http://x.org/a".into(), None)));
        assert_eq!(parse(" https://x.org/post "), Ok(("https://x.org/post".into(), None)));
        assert_eq!(parse("https://x.org"), Ok(("https://x.org/".into(), None)));
        assert_eq!(parse("http://127.0.0.1:8080/a.md?v=1"), Ok(("http://127.0.0.1:8080/a.md?v=1".into(), None)));
        assert_eq!(parse("http://[::1]:80/a.md"), Ok(("http://[::1]/a.md".into(), None)));
    }

    #[test]
    fn web_fragments_split_off_decoded() {
        assert_eq!(parse("https://x.org/a.md#caf%C3%A9"), Ok(("https://x.org/a.md".into(), Some("café".into()))));
        assert_eq!(parse("https://x.org/a.md#"), Ok(("https://x.org/a.md".into(), None)));
    }

    #[test]
    fn web_addresses_need_a_host_and_lose_their_credentials() {
        assert_eq!(parse("http://"), Err(ErrorCode::NotAbsolute));
        assert_eq!(parse("https://user:pw@x.org/a.md"), Ok(("https://x.org/a.md".into(), None)));
        assert_eq!(parse("https://user@x.org/a.md"), Ok(("https://x.org/a.md".into(), None)));
        assert_eq!(parse("ftp://x.org/a.md"), Err(ErrorCode::UnsupportedScheme));
    }

    #[test]
    #[cfg(not(windows))]
    fn windows_drive_paths_are_relative_elsewhere() {
        assert_eq!(parse("C:\\notes\\a.md"), Err(ErrorCode::NotAbsolute));
    }

    #[test]
    #[cfg(windows)]
    fn windows_paths_need_a_drive() {
        assert_eq!(parse("C:\\notes\\a.md"), Ok(("C:\\notes\\a.md".into(), None)));
        assert_eq!(parse("c:/notes/a.md"), Ok(("c:\\notes\\a.md".into(), None)));
        assert_eq!(parse("\\notes\\a.md"), Err(ErrorCode::NotAbsolute));
    }

    #[test]
    fn accepts_absolute_home_relative_and_file_url_forms() {
        assert_eq!(parse(&native("/x/a.md")), ok("/x/a.md", None));
        assert_eq!(parse(&format!("  {} ", native("/x/a.markdown"))), ok("/x/a.markdown", None));
        assert_eq!(parse(&native("/x/A.MD")), ok("/x/A.MD", None));
        assert_eq!(parse("~/notes/a.md"), ok("/home/reader/notes/a.md", None));
        assert_eq!(parse(&native_url("/x/a.md")), ok("/x/a.md", None));
        assert_eq!(parse(&native_url("/x/a.md").replace("file://", "file://localhost")), ok("/x/a.md", None));
    }

    #[test]
    fn file_urls_need_a_local_host_and_are_percent_decoded() {
        assert_eq!(parse("file://server/x/a.md"), Err(ErrorCode::NotAbsolute));
        assert_eq!(parse(&native_url("/x/my%20notes.md")), ok("/x/my notes.md", None));
    }

    #[test]
    fn drive_letters_in_file_urls_lose_their_leading_slash_only_on_windows() {
        assert_eq!(file_url_path("/C:/x/a.md".into(), true), "C:/x/a.md");
        assert_eq!(file_url_path("/c:/x/a.md".into(), true), "c:/x/a.md");
        assert_eq!(file_url_path("/x/a.md".into(), true), "/x/a.md");
        assert_eq!(file_url_path("/C:/x/a.md".into(), false), "/C:/x/a.md");
        assert_eq!(file_url_path("/x/a.md".into(), false), "/x/a.md");
    }

    #[test]
    fn fragments_split_off_and_paths_normalise() {
        assert_eq!(parse(&native_url("/x/a.md#intro")), ok("/x/a.md", Some("intro")));
        assert_eq!(parse(&format!("{}#caf%C3%A9", native("/x/a.md"))), ok("/x/a.md", Some("café")));
        assert_eq!(parse(&native("/x/../y/./a.md")), ok("/y/a.md", None));
    }

    #[test]
    fn network_paths_are_not_local_files() {
        for input in ["//server/share/a.md", "\\\\server\\share\\a.md", "file:////server/share/a.md", "/\\server/a.md"]
        {
            assert_eq!(parse(input), Err(ErrorCode::NotAbsolute), "{input}");
        }
    }
}
