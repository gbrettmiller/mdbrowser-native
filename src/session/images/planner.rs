//! Which of a page's images load, when, and what stays in memory. Pure: one
//! app-wide table of states keyed by location, the current page's sources, a
//! queue, and a least-recently-used cache of decoded pictures (ADR-004).

use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::cache::Cache;
use super::limits::{CACHE_BYTES, MAX_LOADING};
use super::{ImageLoading, Note, Picture, Problem, Refusal, resolve};
use crate::session::location::Location;

/// One location's state, shared by every page that uses it.
#[derive(Debug, Clone, PartialEq)]
enum Entry {
    /// On click, until clicked.
    Waiting,
    /// Web images are off.
    Off,
    Queued,
    Loading,
    Shown(Picture),
    Failed(Problem),
}

/// What an image source on the current page shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Drawn, at this size in pixels.
    Shown((u32, u32)),
    /// The description, then this note; `true` if clicking the note does something.
    Note(Note, bool),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Images {
    table: HashMap<Location, Entry>,
    /// The current page's sources, in order, as resolved when it became ready.
    page: Vec<(String, Result<Location, Refusal>)>,
    queue: Vec<Location>,
    cache: Cache,
    /// Bumped whenever what the current page shows changes, so it is laid out again.
    generation: u64,
}

impl Default for Images {
    fn default() -> Self {
        Self::new()
    }
}

impl Images {
    pub fn new() -> Self {
        Self::with_budget(CACHE_BYTES)
    }

    /// A cache of `budget` bytes of pixels; tests use small ones.
    pub fn with_budget(budget: u64) -> Self {
        Images { table: HashMap::new(), page: Vec::new(), queue: Vec::new(), cache: Cache::new(budget), generation: 0 }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Loads running now.
    pub fn loading(&self) -> usize {
        self.table.values().filter(|entry| matches!(entry, Entry::Loading)).count()
    }

    /// Bytes of shown pictures that the current page doesn't use.
    pub fn unpinned_bytes(&self) -> u64 {
        let pinned = self.pinned();
        self.shown().filter(|(location, _)| !pinned.contains(location)).map(|(_, picture)| picture.bytes()).sum()
    }

    /// The page is being left: its queued loads are dropped (running ones finish
    /// into the cache), and only shown and loading entries are kept.
    pub fn leave_page(mut self) -> Self {
        self.queue.clear();
        self.page.clear();
        self.table.retain(|_, entry| matches!(entry, Entry::Shown(_) | Entry::Loading));
        self.generation += 1;
        self.evict(); // nothing is pinned now
        self
    }

    /// A page became ready with these image sources. Returns the loads to start.
    pub fn open_page(
        mut self,
        srcs: &[String],
        current: &Location,
        home: &Path,
        mode: ImageLoading,
    ) -> (Self, Vec<Location>) {
        self.page = srcs.iter().map(|src| (src.clone(), resolve(src, current, home))).collect();
        for location in self.page_locations() {
            self.touch(&location);
            match self.table.get(&location) {
                Some(Entry::Shown(_) | Entry::Loading) => {}
                _ => self.set_initial(location, mode),
            }
        }
        self.generation += 1;
        self.evict();
        self.pump()
    }

    /// A load finished. Returns the loads to start next.
    pub fn loaded(mut self, location: Location, result: Result<Picture, Problem>) -> (Self, Vec<Location>) {
        if !matches!(self.table.get(&location), Some(Entry::Loading)) {
            return (self, Vec::new()); // a result for a load that wasn't started
        }
        let on_page = self.page_locations().contains(&location);
        match result {
            Ok(picture) => {
                self.table.insert(location.clone(), Entry::Shown(picture));
                self.touch(&location);
            }
            Err(problem) if on_page => {
                self.table.insert(location.clone(), Entry::Failed(problem));
            }
            Err(_) => {
                self.table.remove(&location); // not cached: a revisit tries again
            }
        }
        if on_page {
            self.generation += 1;
        }
        self.evict();
        self.pump()
    }

    /// A click on a waiting image of the current page, or one that failed to load.
    pub fn click(mut self, location: &Location) -> (Self, Vec<Location>) {
        if self.page_locations().contains(location)
            && matches!(self.table.get(location), Some(Entry::Waiting | Entry::Failed(Problem::Unavailable)))
        {
            self.queue_load(location.clone());
            self.generation += 1;
        }
        self.pump()
    }

    /// Ctrl/Cmd+Shift+I: every web image on this page that is waiting, off, or failed
    /// to load, whatever the setting.
    pub fn load_page(mut self) -> (Self, Vec<Location>) {
        for location in self.page_locations().into_iter().filter(Location::is_web) {
            let entry = self.table.get(&location);
            if matches!(entry, Some(Entry::Waiting | Entry::Off | Entry::Failed(Problem::Unavailable))) {
                self.queue_load(location);
            }
        }
        self.generation += 1;
        self.pump()
    }

    /// The setting changed. Nothing shown is hidden and nothing running is cancelled.
    pub fn mode_changed(mut self, mode: ImageLoading) -> (Self, Vec<Location>) {
        for location in self.page_locations().into_iter().filter(Location::is_web) {
            let entry = self.table.get(&location).cloned();
            match (mode, entry) {
                (ImageLoading::Automatic, Some(Entry::Waiting | Entry::Off)) => self.queue_load(location),
                (ImageLoading::OnClick, Some(Entry::Queued | Entry::Off)) => self.set(location, Entry::Waiting),
                (ImageLoading::Off, Some(Entry::Queued | Entry::Waiting)) => self.set(location, Entry::Off),
                _ => {}
            }
        }
        self.generation += 1;
        self.pump()
    }

    /// What `src`, on the current page, shows.
    pub fn view(&self, src: &str) -> View {
        let Some((_, resolved)) = self.page.iter().find(|(page_src, _)| page_src == src) else {
            return View::Note(Note::Loading, false);
        };
        let Ok(location) = resolved else { return View::Note(Note::Refused, false) };
        match self.table.get(location) {
            Some(Entry::Shown(picture)) => View::Shown((picture.width, picture.height)),
            Some(Entry::Waiting) => View::Note(Note::ClickToLoad, true),
            Some(Entry::Off) => View::Note(Note::Off, false),
            Some(Entry::Failed(problem)) => View::Note(problem.note(), *problem == Problem::Unavailable),
            Some(Entry::Queued | Entry::Loading) | None => View::Note(Note::Loading, false),
        }
    }

    /// The picture to draw for `src`, if it is shown.
    pub fn picture(&self, src: &str) -> Option<&Picture> {
        let (_, Ok(location)) = self.page.iter().find(|(page_src, _)| page_src == src)? else { return None };
        match self.table.get(location) {
            Some(Entry::Shown(picture)) => Some(picture),
            _ => None,
        }
    }

    /// Where `src`, on the current page, comes from: what a click on it loads.
    pub fn location(&self, src: &str) -> Option<&Location> {
        self.page.iter().find(|(page_src, _)| page_src == src).and_then(|(_, resolved)| resolved.as_ref().ok())
    }

    /// The current page's locations, once each, in order of first use.
    fn page_locations(&self) -> Vec<Location> {
        let mut seen = HashSet::new();
        self.page
            .iter()
            .filter_map(|(_, resolved)| resolved.as_ref().ok())
            .filter(|location| seen.insert((*location).clone()))
            .cloned()
            .collect()
    }

    fn pinned(&self) -> HashSet<Location> {
        self.page_locations().into_iter().collect()
    }

    fn shown(&self) -> impl Iterator<Item = (&Location, &Picture)> {
        self.table.iter().filter_map(|(location, entry)| match entry {
            Entry::Shown(picture) => Some((location, picture)),
            _ => None,
        })
    }

    /// Local images always load; web images follow the setting.
    fn set_initial(&mut self, location: Location, mode: ImageLoading) {
        match (location.is_web(), mode) {
            (false, _) | (true, ImageLoading::Automatic) => self.queue_load(location),
            (true, ImageLoading::OnClick) => self.set(location, Entry::Waiting),
            (true, ImageLoading::Off) => self.set(location, Entry::Off),
        }
    }

    fn queue_load(&mut self, location: Location) {
        self.table.insert(location.clone(), Entry::Queued);
        if !self.queue.contains(&location) {
            self.queue.push(location);
        }
    }

    /// Sets a state other than queued, taking the location out of the queue.
    fn set(&mut self, location: Location, entry: Entry) {
        self.queue.retain(|queued| *queued != location);
        self.table.insert(location, entry);
    }

    /// Starts queued loads while fewer than `MAX_LOADING` are running.
    fn pump(mut self) -> (Self, Vec<Location>) {
        let mut started = Vec::new();
        while self.loading() < MAX_LOADING && !self.queue.is_empty() {
            let location = self.queue.remove(0);
            if matches!(self.table.get(&location), Some(Entry::Queued)) {
                self.table.insert(location.clone(), Entry::Loading);
                started.push(location);
            }
        }
        (self, started)
    }

    fn touch(&mut self, location: &Location) {
        self.cache.touch(location);
    }

    /// Over budget: drop the least recently used pictures the current page doesn't use.
    fn evict(&mut self) {
        let shown: Vec<(Location, u64)> =
            self.shown().map(|(location, picture)| (location.clone(), picture.bytes())).collect();
        for location in self.cache.victims(&shown, &self.pinned()) {
            self.table.remove(&location);
            self.cache.forget(&location);
        }
    }

    /// Bytes of every shown picture; only eviction makes it fall.
    #[cfg(test)]
    pub(crate) fn shown_bytes(&self) -> u64 {
        self.shown().map(|(_, picture)| picture.bytes()).sum()
    }

    /// Checks the planner's own consistency, for tests: the queue holds exactly the
    /// queued entries, waiting/off/queued entries belong to the current page, and a
    /// non-empty queue means every load slot is in use.
    #[cfg(test)]
    pub(crate) fn check(&self) {
        let queued: HashSet<&Location> = self
            .table
            .iter()
            .filter(|(_, entry)| matches!(entry, Entry::Queued))
            .map(|(location, _)| location)
            .collect();
        assert_eq!(queued, self.queue.iter().collect(), "queue and queued entries agree");
        let page: HashSet<Location> = self.page_locations().into_iter().collect();
        for (location, entry) in &self.table {
            if matches!(entry, Entry::Queued | Entry::Waiting | Entry::Off) {
                assert!(page.contains(location), "{location:?} is {entry:?} but not on the page");
            }
        }
        assert!(self.queue.is_empty() || self.loading() == MAX_LOADING, "a queue never waits for a free slot");
    }
}

#[cfg(test)]
#[path = "planner_tests.rs"]
mod tests;
