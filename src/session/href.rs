//! Reading a link or image reference as a browser would, and the rules for
//! resolving it on a web page. Shared by `links` and `images`, so both obey one
//! boundary (ADR-003).

use std::path::Path;

use url::Url;

use super::address::{is_network_path, parse_file_url, scheme_of, split_reference};
use super::location::{FILE_SCHEME, Location, WEB_SCHEMES};

/// The reference as a browser's URL parser reads it: whitespace and control
/// characters trimmed from the ends, tabs and newlines removed throughout. Checking
/// this form keeps the checks below from seeing a different string than `Url::join`.
pub(super) fn as_browsers_read(reference: &str) -> String {
    reference.trim().trim_matches(|c: char| c <= ' ').chars().filter(|c| !matches!(c, '\t' | '\n' | '\r')).collect()
}

/// A backslash start (`\\host\x`, `\x`), a slash-backslash share (`/\host`), or a
/// drive letter (`C:\x`, `c:/x`, `c:x`). `//host` is a web address, not a share.
pub(super) fn is_disk_shaped(reference: &str) -> bool {
    let bytes = reference.as_bytes();
    let drive = bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':';
    let share = is_network_path(reference) && !reference.starts_with("//");
    reference.starts_with('\\') || share || drive
}

/// A reference on a web page, resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum OnPage {
    /// An http(s) URL. The caller still decides about the local network.
    Web(Url),
    /// Shaped like a disk path or a `file:` URL, before or after resolving.
    Disk,
    /// Another scheme, empty, or unparseable.
    Unsupported,
}

/// Resolves `reference` (already `as_browsers_read`) against `page`. Disk shapes
/// are refused before resolving, since the URL parser would quietly read `\\host\x`
/// or `C:\x` as a path on the same site.
pub(super) fn join_on_page(reference: &str, page: &Url) -> OnPage {
    if is_disk_shaped(reference) {
        return OnPage::Disk;
    }
    match scheme_of(reference).as_deref() {
        Some(FILE_SCHEME) => return OnPage::Disk,
        Some(scheme) if !WEB_SCHEMES.contains(&scheme) => return OnPage::Unsupported,
        _ if reference.is_empty() => return OnPage::Unsupported,
        _ => {}
    }
    // The boundary: whatever the checks above missed, only http(s) leaves here.
    match page.join(reference) {
        Ok(url) if url.scheme() == FILE_SCHEME => OnPage::Disk,
        Ok(url) if WEB_SCHEMES.contains(&url.scheme()) => OnPage::Web(url),
        _ => OnPage::Unsupported,
    }
}

/// A local reference (relative, absolute or `file://`) resolved against the current
/// file: the path and any fragment. `None` for network shares and other schemes.
pub(super) fn local_target(reference: &str, current: &Location) -> Option<(String, Option<String>)> {
    if is_network_path(reference) {
        return None; // protocol-relative or a network share: not a local file
    }
    let (path, fragment) = match scheme_of(reference).as_deref() {
        Some(FILE_SCHEME) => parse_file_url(reference)?,
        Some(_) => return None,
        None => split_reference(reference),
    };
    if path.is_empty() || is_network_path(&path) {
        return None;
    }
    let directory = current.as_file()?.parent().unwrap_or(Path::new("/"));
    Some((directory.join(path).display().to_string(), fragment))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> Url {
        Url::parse("https://x.org/a/post.md").unwrap()
    }

    /// The last line of defence: input that skipped `as_browsers_read` can hide
    /// `file:` from the checks before resolving, but not from the boundary after it.
    #[test]
    fn a_file_url_found_only_after_resolving_is_still_disk() {
        for reference in ["fi\tle:///x.png", "file\n:///x.png", "\u{0}file:///x.png"] {
            assert_eq!(join_on_page(reference, &page()), OnPage::Disk, "{reference:?}");
        }
    }

    #[test]
    fn web_references_resolve_and_others_are_unsupported() {
        assert_eq!(join_on_page("b.md", &page()), OnPage::Web(Url::parse("https://x.org/a/b.md").unwrap()));
        assert_eq!(join_on_page("data:,x", &page()), OnPage::Unsupported);
        assert_eq!(join_on_page("", &page()), OnPage::Unsupported);
    }
}
