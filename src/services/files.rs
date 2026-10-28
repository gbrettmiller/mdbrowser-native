//! Reading Markdown from disk.

use std::path::Path;

pub fn read(path: &Path) -> std::io::Result<String> {
    std::fs::read_to_string(path)
}
