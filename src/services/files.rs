//! Reading Markdown from disk.

use std::io::{ErrorKind, Read};
use std::path::Path;

use crate::session::location::ErrorCode;

/// Larger than any plausible document; guards against runaway reads.
pub const MAX_BYTES: u64 = 16 * 1024 * 1024;

fn code(error: std::io::Error) -> ErrorCode {
    match error.kind() {
        ErrorKind::NotFound => ErrorCode::NotFound,
        _ => ErrorCode::Unreadable,
    }
}

/// Strict UTF-8: a missing file is `NotFound`; anything else that stops us
/// reading it as text is `Unreadable`. That includes permissions, bad encoding,
/// directories, devices and pipes (checked before opening, since opening a pipe
/// blocks), and anything over `MAX_BYTES`. Symlinks to regular files are followed.
pub fn read_markdown(path: &Path) -> Result<String, ErrorCode> {
    let metadata = std::fs::metadata(path).map_err(code)?;
    if !metadata.is_file() || metadata.len() > MAX_BYTES {
        return Err(ErrorCode::Unreadable);
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path).map_err(code)?.take(MAX_BYTES + 1).read_to_end(&mut bytes).map_err(code)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(ErrorCode::Unreadable); // grew after the size check
    }
    String::from_utf8(bytes).map_err(|_| ErrorCode::Unreadable)
}

/// A path given on the command line, made absolute against the working directory.
pub fn from_command_line(argument: &str) -> String {
    std::path::absolute(argument).map_or_else(|_| argument.to_string(), |path| path.display().to_string())
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

    #[test]
    fn oversized_files_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("huge.md");
        std::fs::File::create(&path).unwrap().set_len(MAX_BYTES + 1).unwrap();
        assert_eq!(read_markdown(&path), Err(ErrorCode::Unreadable));
    }

    #[cfg(unix)]
    #[test]
    fn devices_and_pipes_are_refused_but_links_to_files_are_followed() {
        let dir = tempfile::tempdir().unwrap();
        let path = |name: &str| dir.path().join(name);
        std::fs::write(path("real.md"), "# Real").unwrap();
        std::os::unix::fs::symlink(path("real.md"), path("alias.md")).unwrap();
        std::os::unix::fs::symlink("/dev/zero", path("zero.md")).unwrap();
        let made = std::process::Command::new("mkfifo").arg(path("pipe.md")).status().unwrap();
        assert!(made.success());
        assert_eq!(read_markdown(&path("alias.md")), Ok("# Real".into()));
        assert_eq!(read_markdown(&path("zero.md")), Err(ErrorCode::Unreadable));
        assert_eq!(read_markdown(&path("pipe.md")), Err(ErrorCode::Unreadable));
    }
}
