//! Hands http(s) and mailto links to the system, never through a shell.

use crate::session::location::decode;

/// Defence in depth on top of `session::links`: only web and mail URLs, with no
/// characters a shell or launcher could reinterpret, and no mail attachments.
pub fn is_safe_to_open(url: &str) -> bool {
    let scheme = url.split_once(':').map(|(scheme, _)| scheme.to_ascii_lowercase());
    let allowed_scheme = matches!(scheme.as_deref(), Some("http" | "https" | "mailto"));
    let clean = !url.chars().any(|c| c.is_whitespace() || c.is_control() || "\"\\<>`^|".contains(c));
    allowed_scheme && clean && !(scheme.as_deref() == Some("mailto") && has_unsafe_mail_fields(url))
}

/// Mail headers a link may pre-fill. Anything else (`attach`, client-specific
/// extras), after percent-decoding the name, makes the link unsafe: some clients
/// attach local files named in such parameters.
const MAIL_FIELDS: [&str; 6] = ["to", "cc", "bcc", "subject", "body", "in-reply-to"];

fn has_unsafe_mail_fields(url: &str) -> bool {
    url.split_once('?').is_some_and(|(_, query)| {
        query.split('&').filter(|pair| !pair.is_empty()).any(|pair| {
            let name = decode(pair.split('=').next().unwrap_or_default()).trim().to_ascii_lowercase();
            !MAIL_FIELDS.contains(&name.as_str())
        })
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
        for url in [
            "https://example.com/a?x=1&y=2#top",
            "http://x.org/%20a",
            "mailto:a@b.co?subject=Hi&body=x&cc=c@d.co&%62ody=y",
        ] {
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
            "mailto:a@b.co?%61ttach=/home/u/.ssh/id_rsa",
            "mailto:a@b.co?subject=x&X-Unknown=y",
        ] {
            assert!(!is_safe_to_open(url), "{url}");
        }
    }
}
