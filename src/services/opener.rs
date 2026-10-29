//! Hands http(s) and mailto links to the system, never through a shell.

/// Defence in depth on top of `session::links`: only web and mail URLs, with no
/// characters a shell or launcher could reinterpret, and no mail attachments.
pub fn is_safe_to_open(url: &str) -> bool {
    let scheme = url.split_once(':').map(|(scheme, _)| scheme.to_ascii_lowercase());
    let allowed_scheme = matches!(scheme.as_deref(), Some("http" | "https" | "mailto"));
    let clean = !url.chars().any(|c| c.is_whitespace() || c.is_control() || "\"\\<>`^|".contains(c));
    allowed_scheme && clean && !(scheme.as_deref() == Some("mailto") && has_attachment(url))
}

/// Some mail clients attach local files named in `attach`/`attachment` parameters.
fn has_attachment(url: &str) -> bool {
    url.split_once('?').is_some_and(|(_, query)| {
        query
            .split('&')
            .any(|pair| pair.split('=').next().unwrap_or_default().to_ascii_lowercase().starts_with("attach"))
    })
}

/// Opens the URL with the system's default handler (ShellExecuteW on Windows,
/// `open` on macOS, xdg-open and friends elsewhere).
pub fn open(url: &str) -> Result<(), String> {
    if !is_safe_to_open(url) {
        return Err(format!("refused to open {url:?}"));
    }
    open::that(url).map_err(|error| format!("couldn't open {url}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordinary_web_and_mail_links_are_allowed() {
        for url in ["https://example.com/a?x=1&y=2#top", "http://x.org/%20a", "mailto:a@b.co?subject=Hi&body=x"] {
            assert!(is_safe_to_open(url), "{url}");
        }
    }

    #[test]
    fn shell_metacharacters_and_whitespace_are_refused() {
        for url in [
            "https://x.org/?a&calc.exe\"",
            "https://x.org/a b",
            "https://x.org/a\tb",
            "https://x.org/\u{0}",
            "https://x.org/a\\b",
            "https://x.org/a\nb",
        ] {
            assert!(!is_safe_to_open(url), "{url:?}");
        }
    }

    #[test]
    fn other_schemes_and_mail_attachments_are_refused() {
        for url in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "mailto:a@b.co?attach=/etc/passwd",
            "mailto:a@b.co?Attachment=x",
        ] {
            assert!(!is_safe_to_open(url), "{url}");
        }
    }
}
