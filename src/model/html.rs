//! Raw HTML is stripped, never interpreted (ADR-002): tags, comments and
//! declarations go, and the text between them stays. Recognises CommonMark's
//! raw-HTML shapes; a `<` that starts none of them is ordinary text.
//!
//! The output is plain text for the native renderer. A single pass can leave
//! text that looks like markup (`<scr<b>ipt>` → `<script>`); that is harmless
//! because nothing downstream interprets HTML, and must stay that way.

/// Empty comments, which CommonMark closes early.
const EMPTY_COMMENTS: [&str; 2] = ["<!-->", "<!--->"];

/// CommonMark attribute names: an ASCII letter or one of these, then letters, digits or these.
const ATTRIBUTE_NAME_START: &str = "_:";
const ATTRIBUTE_NAME_REST: &str = "_.:-";

/// Characters that end an unquoted attribute value, besides whitespace.
const UNQUOTED_VALUE_STOP: &str = "\"'=<>`";

pub fn strip(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(at) = rest.find('<') {
        out.push_str(&rest[..at]);
        match markup_len(&rest[at..]) {
            Some(len) => rest = &rest[at + len..],
            None => {
                out.push('<');
                rest = &rest[at + 1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/// Length of the markup starting at `text` (which begins with `<`), if any.
fn markup_len(text: &str) -> Option<usize> {
    EMPTY_COMMENTS
        .iter()
        .find(|empty| text.starts_with(**empty))
        .map(|empty| empty.len())
        .or_else(|| runs_until(text, "<!--", "-->"))
        .or_else(|| runs_until(text, "<![CDATA[", "]]>"))
        .or_else(|| runs_until(text, "<?", "?>"))
        .or_else(|| declaration(text))
        .or_else(|| closing_tag(text))
        .or_else(|| open_tag(text))
}

/// `open … close`. Unclosed, it runs to the end, as its HTML block does; that
/// also keeps the scan linear, since a failed search is never repeated.
fn runs_until(text: &str, open: &str, close: &str) -> Option<usize> {
    let body = text.strip_prefix(open)?;
    Some(body.find(close).map_or(text.len(), |at| open.len() + at + close.len()))
}

/// `<!NAME …>`, running to the end when unclosed (see `runs_until`).
fn declaration(text: &str) -> Option<usize> {
    text.strip_prefix("<!")?.starts_with(|c: char| c.is_ascii_alphabetic()).then_some(())?;
    runs_until(text, "<!", ">")
}

/// `</name >`
fn closing_tag(text: &str) -> Option<usize> {
    let rest = tag_name(text.strip_prefix("</")?)?.trim_start();
    let after = rest.strip_prefix('>')?;
    Some(text.len() - after.len())
}

/// `<name attr="v" attr2=v attr3 />`
fn open_tag(text: &str) -> Option<usize> {
    let mut rest = tag_name(text.strip_prefix('<')?)?;
    loop {
        let trimmed = rest.trim_start();
        if let Some(after) = trimmed.strip_prefix("/>").or_else(|| trimmed.strip_prefix('>')) {
            return Some(text.len() - after.len());
        }
        if trimmed.len() == rest.len() {
            return None; // attributes must be separated by whitespace
        }
        rest = attribute(trimmed)?;
    }
}

/// A tag name; returns what follows it.
fn tag_name(text: &str) -> Option<&str> {
    text.starts_with(|c: char| c.is_ascii_alphabetic()).then_some(())?;
    let end = text.find(|c: char| !(c.is_ascii_alphanumeric() || c == '-')).unwrap_or(text.len());
    Some(&text[end..])
}

/// `name`, `name=value`, `name='value'` or `name="value"`; returns what follows it.
fn attribute(text: &str) -> Option<&str> {
    text.starts_with(|c: char| c.is_ascii_alphabetic() || ATTRIBUTE_NAME_START.contains(c)).then_some(())?;
    let end =
        text.find(|c: char| !(c.is_ascii_alphanumeric() || ATTRIBUTE_NAME_REST.contains(c))).unwrap_or(text.len());
    let after_name = &text[end..];
    let Some(value) = after_name.trim_start().strip_prefix('=') else { return Some(after_name) };
    let value = value.trim_start();
    if let Some(quote) = value.chars().next().filter(|c| matches!(c, '"' | '\'')) {
        let inside = &value[quote.len_utf8()..];
        return inside.find(quote).map(|at| &inside[at + quote.len_utf8()..]);
    }
    let end = value.find(|c: char| c.is_whitespace() || UNQUOTED_VALUE_STOP.contains(c)).unwrap_or(value.len());
    (end > 0).then(|| &value[end..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_go_and_text_stays() {
        assert_eq!(strip("before <b>bold</b> after"), "before bold after");
        assert_eq!(strip("<img src=x onerror=alert(1)>tail"), "tail");
        assert_eq!(strip("<script>alert(1)</script>"), "alert(1)");
    }

    #[test]
    fn tag_shapes() {
        assert_eq!(strip("<br/>a<hr />b"), "ab");
        assert_eq!(strip("<custom-tag data-x=\"1\">y</custom-tag >"), "y");
        assert_eq!(strip("<input disabled>x"), "x");
        assert_eq!(strip("<a href=x>t</a>"), "t");
        assert_eq!(strip("<a\nhref=x>t</a>"), "t");
        assert_eq!(strip("<B>x</B>"), "x");
    }

    #[test]
    fn quoted_attributes_may_contain_angle_brackets() {
        assert_eq!(strip("<a title=\"a>b\">x</a>"), "x");
        assert_eq!(strip("<a title='<c>'>x</a>"), "x");
    }

    #[test]
    fn comments_declarations_and_instructions_go() {
        assert_eq!(strip("<!-- a comment -->kept"), "kept");
        assert_eq!(strip("a<!---->b<!-->c"), "abc");
        assert_eq!(strip("<!DOCTYPE html>x"), "x");
        assert_eq!(strip("<?php echo 1; ?>x"), "x");
        assert_eq!(strip("<![CDATA[ raw <b> ]]>x"), "x");
    }

    #[test]
    fn unclosed_comments_cdata_instructions_and_declarations_run_to_the_end() {
        assert_eq!(strip("a<!-- never closed\nmore"), "a");
        assert_eq!(strip("a<![CDATA[ never closed"), "a");
        assert_eq!(strip("a<? never closed"), "a");
        assert_eq!(strip("a<!DOCTYPE never closed"), "a");
    }

    #[test]
    fn angle_brackets_that_are_not_tags_stay() {
        assert_eq!(strip("a < b and <3"), "a < b and <3");
        assert_eq!(strip("x <- y"), "x <- y");
        assert_eq!(strip("a <b"), "a <b");
        assert_eq!(strip("ends with <"), "ends with <");
        assert_eq!(strip("<1a>"), "<1a>");
        assert_eq!(strip("< b>"), "< b>");
        assert_eq!(strip("</ b>"), "</ b>");
        assert_eq!(strip("<!1>"), "<!1>");
        assert_eq!(strip("<a title=\"open>"), "<a title=\"open>");
    }

    #[test]
    fn the_output_is_plain_text_even_when_it_looks_like_markup() {
        assert_eq!(strip("<scr<b>ipt>"), "<script>");
    }

    /// Adversarial input that would be quadratic if a failed search were repeated
    /// from every `<`; at this size that would take seconds, not milliseconds.
    #[test]
    fn pathological_input_stays_linear() {
        let declarations = "<!a".repeat(300_000);
        assert_eq!(strip(&declarations), "");
        let quotes = format!("<a x=\"{}", "<a x='".repeat(150_000));
        assert!(strip(&quotes).starts_with("<a x=\""));
    }
}
