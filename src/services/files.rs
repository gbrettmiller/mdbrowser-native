//! Reading Markdown from disk.

use std::io::ErrorKind;
use std::path::Path;

use crate::session::location::ErrorCode;

/// Strict UTF-8: a missing file is `NotFound`; anything else that stops us
/// reading it as text (permissions, a directory, bad encoding) is `Unreadable`.
pub fn read_markdown(path: &Path) -> Result<String, ErrorCode> {
    std::fs::read_to_string(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound => ErrorCode::NotFound,
        _ => ErrorCode::Unreadable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_read_failures_to_error_codes() {
        let dir = tempfile::tempdir().unwrap();
        let path = |name: &str| dir.path().join(name);
        std::fs::write(path("ok.md"), "# Hi").unwrap();
        std::fs::write(path("bad.md"), [0xff, 0xfe]).unwrap();
        std::fs::create_dir(path("folder.md")).unwrap();
        assert_eq!(read_markdown(&path("ok.md")), Ok("# Hi".into()));
        assert_eq!(read_markdown(&path("missing.md")), Err(ErrorCode::NotFound));
        assert_eq!(read_markdown(&path("bad.md")), Err(ErrorCode::Unreadable));
        assert_eq!(read_markdown(&path("folder.md")), Err(ErrorCode::Unreadable));
    }
}
