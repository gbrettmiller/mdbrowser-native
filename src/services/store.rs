//! Persists History and settings as small JSON files in the user's data directory.

use std::path::{Path, PathBuf};

use crate::session::Settings;
use crate::session::visits::{self, Visit};
use crate::theme::ReadingFont;

#[derive(Debug, Clone)]
pub struct Store {
    dir: PathBuf,
}

impl Store {
    pub fn at(dir: impl Into<PathBuf>) -> Self {
        Store { dir: dir.into() }
    }

    /// `$XDG_DATA_HOME/mdbrowser` (or the platform equivalent).
    pub fn default_location() -> Option<Self> {
        dirs::data_dir().map(|dir| Store::at(dir.join("mdbrowser")))
    }

    fn file(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    pub fn load_visits(&self) -> Vec<Visit> {
        std::fs::read_to_string(self.file(HISTORY)).map(|raw| visits::parse_stored(&raw)).unwrap_or_default()
    }

    pub fn save_visits(&self, visits: &[Visit]) -> std::io::Result<()> {
        write_atomically(&self.file(HISTORY), &visits::to_stored(visits))
    }

    pub fn load_settings(&self) -> Settings {
        let stored = std::fs::read_to_string(self.file(SETTINGS)).ok();
        let font = stored
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .and_then(|json| json.get("font").and_then(|font| font.as_str()).and_then(ReadingFont::from_key));
        Settings { font: font.unwrap_or_default() }
    }

    pub fn save_settings(&self, settings: Settings) -> std::io::Result<()> {
        let json = serde_json::json!({ "font": settings.font.key() });
        write_atomically(&self.file(SETTINGS), &json.to_string())
    }
}

const HISTORY: &str = "history.json";
const SETTINGS: &str = "settings.json";

/// Write to a sibling temp file, then rename, so a crash never leaves half a file.
fn write_atomically(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, contents)?;
    std::fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn visit(address: &str) -> Visit {
        Visit { address: address.into(), title: address.into(), visited_at: 1 }
    }

    #[test]
    fn a_fresh_store_is_empty_with_default_settings() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path().join("not-yet-created"));
        assert_eq!(store.load_visits(), vec![]);
        assert_eq!(store.load_settings(), Settings::default());
    }

    #[test]
    fn visits_and_settings_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path().join("mdbrowser"));
        store.save_visits(&[visit("/a.md"), visit("/b.md")]).unwrap();
        store.save_settings(Settings { font: ReadingFont::Quattro }).unwrap();
        assert_eq!(store.load_visits(), vec![visit("/a.md"), visit("/b.md")]);
        assert_eq!(store.load_settings().font, ReadingFont::Quattro);
    }

    #[test]
    fn damaged_files_fall_back_instead_of_failing() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path());
        std::fs::write(dir.path().join("history.json"), "{{{").unwrap();
        std::fs::write(dir.path().join("settings.json"), r#"{"font": "comic-sans"}"#).unwrap();
        assert_eq!(store.load_visits(), vec![]);
        assert_eq!(store.load_settings(), Settings::default());
    }

    #[test]
    fn saving_leaves_no_temporary_files_behind() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path());
        store.save_visits(&[visit("/a.md")]).unwrap();
        let names: Vec<_> = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names, ["history.json"]);
    }
}
