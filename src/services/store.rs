//! Persists History and settings as small JSON files in the user's data directory.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::session::Settings;
use crate::session::visits::{self, Visit};
use crate::theme::ReadingFont;

/// Orders saves: issued when an effect is created (in dispatch order), so a
/// save that finishes late can't overwrite a newer one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticket(u64);

/// Saves run one at a time; each file remembers the newest ticket written.
#[derive(Debug)]
pub struct Store {
    dir: PathBuf,
    next_ticket: AtomicU64,
    written: Mutex<HashMap<&'static str, Ticket>>,
}

impl Store {
    pub fn at(dir: impl Into<PathBuf>) -> Self {
        Store { dir: dir.into(), next_ticket: AtomicU64::new(0), written: Mutex::new(HashMap::new()) }
    }

    /// `$MDBROWSER_DATA_DIR` if set (tests, screenshots), else
    /// `$XDG_DATA_HOME/mdbrowser` or the platform equivalent.
    pub fn default_location() -> Option<Self> {
        match std::env::var_os("MDBROWSER_DATA_DIR") {
            Some(dir) => Some(Store::at(dir)),
            None => dirs::data_dir().map(|dir| Store::at(dir.join("mdbrowser"))),
        }
    }

    fn file(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }

    pub fn load_visits(&self) -> Vec<Visit> {
        std::fs::read_to_string(self.file(HISTORY)).map(|raw| visits::parse_stored(&raw)).unwrap_or_default()
    }

    pub fn ticket(&self) -> Ticket {
        Ticket(self.next_ticket.fetch_add(1, Ordering::SeqCst))
    }

    /// Writes `contents` unless a newer ticket has already been written to `name`.
    fn save(&self, name: &'static str, ticket: Ticket, contents: &str) -> std::io::Result<()> {
        let mut written = self.written.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        if written.get(name).is_some_and(|newest| *newest >= ticket) {
            return Ok(());
        }
        write_atomically(&self.file(name), ticket, contents)?;
        written.insert(name, ticket);
        Ok(())
    }

    pub fn save_visits(&self, ticket: Ticket, visits: &[Visit]) -> std::io::Result<()> {
        self.save(HISTORY, ticket, &visits::to_stored(visits))
    }

    pub fn load_settings(&self) -> Settings {
        let stored = std::fs::read_to_string(self.file(SETTINGS)).ok();
        let font = stored
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .and_then(|json| json.get("font").and_then(|font| font.as_str()).and_then(ReadingFont::from_key));
        Settings { font: font.unwrap_or_default() }
    }

    pub fn save_settings(&self, ticket: Ticket, settings: Settings) -> std::io::Result<()> {
        let json = serde_json::json!({ "font": settings.font.key() });
        self.save(SETTINGS, ticket, &json.to_string())
    }
}

const HISTORY: &str = "history.json";
const SETTINGS: &str = "settings.json";

/// Write to a sibling temp file, then rename, so a crash never leaves half a file.
/// The temp name is unique per process and save, so two app instances can't collide.
fn write_atomically(path: &Path, ticket: Ticket, contents: &str) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let temporary = path.with_extension(format!("json.{}.{}.tmp", std::process::id(), ticket.0));
    let written = std::fs::write(&temporary, contents).and_then(|()| std::fs::rename(&temporary, path));
    if written.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    written
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
        store.save_visits(store.ticket(), &[visit("/a.md"), visit("/b.md")]).unwrap();
        store.save_settings(store.ticket(), Settings { font: ReadingFont::Quattro }).unwrap();
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
        store.save_visits(store.ticket(), &[visit("/a.md")]).unwrap();
        let names: Vec<_> = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names, ["history.json"]);
    }

    #[test]
    fn a_late_older_save_never_overwrites_a_newer_one() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path());
        let (older, newer) = (store.ticket(), store.ticket());
        store.save_visits(newer, &[]).unwrap();
        store.save_visits(older, &[visit("/stale.md")]).unwrap();
        assert_eq!(store.load_visits(), vec![]);
        let (older, newer) = (store.ticket(), store.ticket());
        store.save_settings(newer, Settings { font: ReadingFont::Inter }).unwrap();
        store.save_settings(older, Settings::default()).unwrap();
        assert_eq!(store.load_settings().font, ReadingFont::Inter);
    }

    #[test]
    fn concurrent_saves_keep_the_newest_snapshot() {
        let dir = tempfile::tempdir().unwrap();
        let store = std::sync::Arc::new(Store::at(dir.path()));
        let tickets: Vec<_> = (0..16).map(|i| (store.ticket(), i)).collect();
        let threads: Vec<_> = tickets
            .into_iter()
            .rev()
            .map(|(ticket, i)| {
                let store = store.clone();
                std::thread::spawn(move || store.save_visits(ticket, &[visit(&format!("/{i}.md"))]))
            })
            .collect();
        threads.into_iter().for_each(|thread| thread.join().unwrap().unwrap());
        assert_eq!(store.load_visits(), vec![visit("/15.md")]);
        let names: Vec<_> = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names, ["history.json"]);
    }

    #[test]
    fn a_failed_save_leaves_no_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::at(dir.path());
        std::fs::create_dir(dir.path().join("history.json")).unwrap(); // rename onto a directory fails
        assert!(store.save_visits(store.ticket(), &[visit("/a.md")]).is_err());
        let names: Vec<_> = std::fs::read_dir(dir.path()).unwrap().map(|e| e.unwrap().file_name()).collect();
        assert_eq!(names, ["history.json"]);
    }
}
