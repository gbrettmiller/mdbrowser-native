//! Reading Markdown from disk.

use std::io::{ErrorKind, Read};
use std::path::Path;

use crate::session::address::has_scheme;
use crate::session::error::ErrorCode;

/// Larger than any plausible document; guards against runaway reads.
pub const MAX_BYTES: u64 = 16 * 1024 * 1024;

/// Why a capped read stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadError {
    NotFound,
    /// Over `MAX_BYTES`.
    TooLarge,
    /// Permissions, a directory, a device or a pipe.
    Unreadable,
}

/// A regular file's bytes, up to `MAX_BYTES`. Pipes and devices are refused
/// without blocking. Symlinks to regular files are followed.
pub fn read_capped(path: &Path) -> Result<Vec<u8>, ReadError> {
    let failed = |error: std::io::Error| match error.kind() {
        ErrorKind::NotFound => ReadError::NotFound,
        _ => ReadError::Unreadable,
    };
    let file = open_without_blocking(path).map_err(failed)?;
    let metadata = file.metadata().map_err(failed)?;
    if !metadata.is_file() {
        return Err(ReadError::Unreadable);
    }
    if metadata.len() > MAX_BYTES {
        return Err(ReadError::TooLarge);
    }
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1).read_to_end(&mut bytes).map_err(failed)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(ReadError::TooLarge); // grew after the size check
    }
    Ok(bytes)
}

/// Strict UTF-8: a missing file is `NotFound`; anything else that stops us
/// reading it as text is `Unreadable`. That includes permissions, bad encoding,
/// directories, devices and pipes, and anything over `MAX_BYTES`.
pub fn read_markdown(path: &Path) -> Result<String, ErrorCode> {
    let bytes = read_capped(path).map_err(|error| match error {
        ReadError::NotFound => ErrorCode::NotFound,
        ReadError::TooLarge | ReadError::Unreadable => ErrorCode::Unreadable,
    })?;
    String::from_utf8(bytes).map_err(|_| ErrorCode::Unreadable)
}

/// Opening a pipe for reading blocks until a writer appears, so on Unix open
/// non-blocking and check the type on the handle. That leaves no window to
/// swap a pipe in after the check. Regular-file reads are unaffected.
#[cfg(unix)]
fn open_without_blocking(path: &Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    // O_NOCTTY: a symlink to a terminal must never become our controlling terminal.
    std::fs::OpenOptions::new().read(true).custom_flags(libc::O_NONBLOCK | libc::O_NOCTTY).open(path)
}

#[cfg(not(unix))]
fn open_without_blocking(path: &Path) -> std::io::Result<std::fs::File> {
    std::fs::File::open(path)
}

/// An address given on the command line: a URL as is, a path made absolute against the working directory.
pub fn from_command_line(argument: &str) -> String {
    if has_scheme(argument) {
        return argument.to_string(); // a URL, not a path
    }
    std::path::absolute(argument).map_or_else(|_| argument.to_string(), |path| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capped_reads_say_why_they_stop() {
        let dir = tempfile::tempdir().unwrap();
        let path = |name: &str| dir.path().join(name);
        std::fs::write(path("a.png"), [1, 2, 3]).unwrap();
        assert_eq!(read_capped(&path("a.png")), Ok(vec![1, 2, 3]));
        assert_eq!(read_capped(&path("missing.png")), Err(ReadError::NotFound));
        std::fs::create_dir(path("dir.png")).unwrap();
        assert_eq!(read_capped(&path("dir.png")), Err(ReadError::Unreadable));
        let big = std::fs::File::create(path("big.png")).unwrap();
        big.set_len(MAX_BYTES + 1).unwrap();
        assert_eq!(read_capped(&path("big.png")), Err(ReadError::TooLarge));
    }

    #[test]
    #[cfg(unix)]
    fn a_pipe_is_refused_without_blocking() {
        let dir = tempfile::tempdir().unwrap();
        let fifo = dir.path().join("pipe.png");
        let made = std::process::Command::new("mkfifo").arg(&fifo).status().unwrap();
        assert!(made.success());
        assert_eq!(read_capped(&fifo), Err(ReadError::Unreadable));
    }

    #[test]
    fn addresses_with_a_scheme_pass_through_the_command_line() {
        for argument in ["https://x.org/a.md", "HTTP://x.org/", "file:///x/a.md", "ftp://x.org/a.md"] {
            assert_eq!(from_command_line(argument), argument);
        }
    }

    #[test]
    fn paths_on_the_command_line_become_absolute() {
        assert!(std::path::Path::new(&from_command_line("notes/a.md")).is_absolute());
    }

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
