//! Whether a successful response is Markdown we can read, and its text.

use url::Url;

use crate::session::error::ErrorCode;
use crate::session::location::is_markdown_name;

const MARKDOWN_TYPES: [&str; 2] = ["text/markdown", "text/x-markdown"];
/// No type (`""`) or plain text: they say nothing about the format, so the path decides.
const PLAIN_TYPES: [&str; 2] = ["", "text/plain"];
/// Charsets that decode as UTF-8 (`utf8` is a common misspelling).
const READABLE_CHARSETS: [&str; 3] = ["utf-8", "utf8", "us-ascii"];

/// Whether a successful response at `final_url` is a Markdown document we can read,
/// judged by its headers alone. The caller must still decode the bytes strictly.
pub fn body_policy(content_type: Option<&str>, final_url: &Url) -> Result<(), ErrorCode> {
    let mut parts = content_type.unwrap_or_default().split(';');
    let media = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
    let markdown = MARKDOWN_TYPES.contains(&media.as_str())
        || (PLAIN_TYPES.contains(&media.as_str()) && is_markdown_name(final_url.path()));
    if !markdown {
        return Err(ErrorCode::NotMarkdown);
    }
    // Every charset parameter must be readable, so a decoy can't vouch for the real one.
    let readable = parts
        .filter_map(|param| param.split_once('='))
        .filter(|(name, _)| name.trim().eq_ignore_ascii_case("charset"))
        .all(|(_, value)| READABLE_CHARSETS.contains(&value.trim().trim_matches('"').to_ascii_lowercase().as_str()));
    if readable { Ok(()) } else { Err(ErrorCode::Unreadable) }
}

/// Whether a body arrived as plain bytes. The client decodes gzip itself and then
/// drops the header, so any encoding still declared was not decoded.
pub fn encoding_policy(content_encoding: Option<&str>) -> Result<(), ErrorCode> {
    match content_encoding.map(str::trim) {
        None | Some("") => Ok(()),
        Some(encoding) if encoding.eq_ignore_ascii_case("identity") => Ok(()),
        Some(_) => Err(ErrorCode::Unreadable),
    }
}

/// A body as text: strict UTF-8, without a leading byte-order mark.
pub fn to_text(bytes: Vec<u8>) -> Result<String, ErrorCode> {
    let text = String::from_utf8(bytes).map_err(|_| ErrorCode::Unreadable)?;
    Ok(match text.strip_prefix('\u{feff}') {
        Some(rest) => rest.to_string(),
        None => text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(text: &str) -> Url {
        Url::parse(text).unwrap()
    }

    fn body(content_type: Option<&str>, at: &str) -> Result<(), ErrorCode> {
        body_policy(content_type, &url(at))
    }

    const PAGE: &str = "https://example.com/page";

    #[test]
    fn markdown_types_are_markdown_at_any_path() {
        for content_type in ["text/markdown; charset=utf-8", "Text/Markdown", "text/x-markdown"] {
            assert_eq!(body(Some(content_type), PAGE), Ok(()), "{content_type}");
        }
    }

    #[test]
    fn plain_or_missing_types_are_markdown_only_at_markdown_paths() {
        assert_eq!(body(Some("text/plain ; charset=UTF-8"), "https://example.com/a.md"), Ok(()));
        assert_eq!(body(Some("text/plain"), "https://example.com/a.md?v=1"), Ok(()));
        assert_eq!(body(Some("text/plain"), "https://example.com/notes.markdown"), Ok(()));
        assert_eq!(body(None, "https://example.com/A.MD"), Ok(()));
        assert_eq!(body(Some(""), "https://example.com/a.md"), Ok(()));
        for at in ["https://example.com/a.txt", "https://example.com/a.md/", "https://example.com/readme"] {
            assert_eq!(body(Some("text/plain; charset=utf-8"), at), Err(ErrorCode::NotMarkdown), "{at}");
        }
        assert_eq!(body(None, "https://example.com/"), Err(ErrorCode::NotMarkdown));
    }

    #[test]
    fn other_types_are_not_markdown_even_at_markdown_paths() {
        for content_type in ["text/html", "application/json", "application/octet-stream", "text/markdownish"] {
            assert_eq!(
                body(Some(content_type), "https://example.com/a.md"),
                Err(ErrorCode::NotMarkdown),
                "{content_type}"
            );
        }
        assert_eq!(body(Some("text/html; charset=latin1"), PAGE), Err(ErrorCode::NotMarkdown));
    }

    #[test]
    fn only_utf8_and_ascii_charsets_are_readable() {
        for content_type in [
            "text/markdown; charset=utf-8",
            "text/markdown; charset=\"UTF-8\"",
            "text/markdown;CHARSET=us-ascii",
            "text/markdown; charset=utf8",
            "text/markdown; format=fixed; charset=utf-8",
            "text/markdown; charset",
        ] {
            assert_eq!(body(Some(content_type), PAGE), Ok(()), "{content_type}");
        }
        for content_type in [
            "text/markdown; charset=iso-8859-1",
            "text/markdown; format=fixed; charset=latin1",
            "text/markdown; charset=",
            "text/markdown; charset='utf-8'",
            "text/markdown; x=\"; charset=utf-8\"; charset=iso-8859-1",
        ] {
            assert_eq!(body(Some(content_type), PAGE), Err(ErrorCode::Unreadable), "{content_type}");
        }
        assert_eq!(
            body(Some("text/plain; charset=iso-8859-1"), "https://example.com/a.md"),
            Err(ErrorCode::Unreadable)
        );
    }

    #[test]
    fn only_unencoded_bodies_are_readable() {
        for encoding in [None, Some(""), Some("identity"), Some(" Identity ")] {
            assert_eq!(encoding_policy(encoding), Ok(()), "{encoding:?}");
        }
        for encoding in ["gzip", "br", "deflate", "gzip, gzip", "identity, br"] {
            assert_eq!(encoding_policy(Some(encoding)), Err(ErrorCode::Unreadable), "{encoding}");
        }
    }

    #[test]
    fn bodies_decode_as_strict_utf8_without_a_bom() {
        assert_eq!(to_text(b"# Hi".to_vec()), Ok("# Hi".into()));
        assert_eq!(to_text("\u{feff}# Hi".into()), Ok("# Hi".into()));
        assert_eq!(to_text("a\u{feff}b".into()), Ok("a\u{feff}b".into()));
        assert_eq!(to_text(Vec::new()), Ok(String::new()));
        assert_eq!(to_text(b"\xef\xbb\xbf".to_vec()), Ok(String::new()));
        assert_eq!(to_text("\u{feff}\u{feff}x".into()), Ok("\u{feff}x".into()));
        for bytes in [&b"caf\xe9"[..], b"\xff\xfe#\x00", b"\xe2\x82", b"\xc0\xaf", b"\xed\xa0\x80"] {
            assert_eq!(to_text(bytes.to_vec()), Err(ErrorCode::Unreadable), "{bytes:?}");
        }
    }
}
