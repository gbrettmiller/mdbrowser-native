//! Which of a page's images load, when, and what stays in memory. Pure: one
//! app-wide table of states keyed by location, the current page's sources, a
//! queue, and a least-recently-used cache of decoded pictures (ADR-004).

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use super::cache::Cache;
use super::limits::{CACHE_BYTES, MAX_CONCURRENT_LOADS};
use super::{ImageLoading, Note, Picture, Problem, Refusal, resolve};
use crate::session::location::Location;

/// One location's state, shared by every page that uses it.
#[derive(Debug, Clone, PartialEq)]
enum Entry {
    /// On click, until clicked.
    AwaitingClick,
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
    Shown { width: u32, height: u32 },
    /// The description, then this note.
    Note { note: Note, clickable: bool },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Images {
    table: HashMap<Location, Entry>,
    /// The current page's sources, as resolved when it became ready.
    page_sources: HashMap<String, Result<Location, Refusal>>,
    /// The current page's locations, once each, in order of first use.
    page_locations: Vec<Location>,
    /// The same locations as a set: the pictures eviction never drops.
    pinned: HashSet<Location>,
    queue: VecDeque<Location>,
    cache: Cache,
    /// Loads in flight, kept as entries change.
    loading: usize,
    /// Pixel bytes of every shown picture, kept as entries change.
    shown_bytes: u64,
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
        Images {
            table: HashMap::new(),
            page_sources: HashMap::new(),
            page_locations: Vec::new(),
            pinned: HashSet::new(),
            queue: VecDeque::new(),
            cache: Cache::new(budget),
            loading: 0,
            shown_bytes: 0,
            generation: 0,
        }
    }

    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Loads running now.
    pub const fn loading_count(&self) -> usize {
        self.loading
    }

    /// Bytes of shown pictures that the current page doesn't use.
    pub fn unpinned_bytes(&self) -> u64 {
        self.shown()
            .filter(|(location, _)| !self.pinned.contains(location))
            .map(|(_, picture)| picture.pixel_bytes())
            .sum()
    }

    /// The page is being left: its queued loads are dropped (running ones finish
    /// into the cache), and only shown and loading entries are kept.
    pub fn leave_page(mut self) -> Self {
        self.queue.clear();
        self.page_sources.clear();
        self.page_locations.clear();
        self.pinned.clear();
        self.table.retain(|_, entry| matches!(entry, Entry::Shown(_) | Entry::Loading));
        let table = &self.table;
        self.cache.retain(|location| table.contains_key(location));
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
        let resolved: Vec<(String, Result<Location, Refusal>)> =
            srcs.iter().map(|src| (src.clone(), resolve(src, current, home))).collect();
        let mut pinned = HashSet::new();
        self.page_locations = resolved
            .iter()
            .filter_map(|(_, resolved)| resolved.as_ref().ok())
            .filter(|location| pinned.insert((*location).clone()))
            .cloned()
            .collect();
        self.pinned = pinned;
        self.page_sources = resolved.into_iter().collect();
        for location in self.page_locations.clone() {
            self.cache.touch(&location);
            match self.table.get(&location) {
                Some(Entry::Shown(_) | Entry::Loading) => {}
                _ => self.set_initial(location, mode),
            }
        }
        self.generation += 1;
        self.evict();
        self.start_queued()
    }

    /// A load finished. Returns the loads to start next.
    pub fn loaded(mut self, location: Location, result: Result<Picture, Problem>) -> (Self, Vec<Location>) {
        if !matches!(self.table.get(&location), Some(Entry::Loading)) {
            return (self, Vec::new()); // a result for a load that wasn't started
        }
        self.loading -= 1;
        let on_page = self.pinned.contains(&location);
        match result {
            Ok(picture) => {
                self.shown_bytes += picture.pixel_bytes();
                self.table.insert(location.clone(), Entry::Shown(picture));
                self.cache.touch(&location);
            }
            Err(problem) if on_page => {
                self.table.insert(location, Entry::Failed(problem));
            }
            Err(_) => {
                self.table.remove(&location); // not cached: a revisit tries again
                self.cache.forget(&location);
            }
        }
        if on_page {
            self.generation += 1;
        }
        self.evict();
        self.start_queued()
    }

    /// A click on a waiting image of the current page, or one that failed to load.
    pub fn click(mut self, location: &Location) -> (Self, Vec<Location>) {
        if self.pinned.contains(location)
            && matches!(self.table.get(location), Some(Entry::AwaitingClick | Entry::Failed(Problem::Unavailable)))
        {
            self.queue_load(location.clone());
            self.generation += 1;
        }
        self.start_queued()
    }

    /// Ctrl/Cmd+Shift+I: every web image on this page that is waiting, off, or failed
    /// to load, whatever the setting.
    pub fn load_page(mut self) -> (Self, Vec<Location>) {
        for location in self.page_locations.clone().into_iter().filter(Location::is_web) {
            let entry = self.table.get(&location);
            if matches!(entry, Some(Entry::AwaitingClick | Entry::Off | Entry::Failed(Problem::Unavailable))) {
                self.queue_load(location);
            }
        }
        self.generation += 1;
        self.start_queued()
    }

    /// The setting changed. Nothing shown is hidden and nothing running is cancelled.
    pub fn mode_changed(mut self, mode: ImageLoading) -> (Self, Vec<Location>) {
        for location in self.page_locations.clone().into_iter().filter(Location::is_web) {
            let entry = self.table.get(&location).cloned();
            match (mode, entry) {
                (ImageLoading::Automatic, Some(Entry::AwaitingClick | Entry::Off)) => self.queue_load(location),
                (ImageLoading::OnClick, Some(Entry::Queued | Entry::Off)) => self.set(location, Entry::AwaitingClick),
                (ImageLoading::Off, Some(Entry::Queued | Entry::AwaitingClick)) => self.set(location, Entry::Off),
                _ => {}
            }
        }
        self.generation += 1;
        self.start_queued()
    }

    /// What `src`, on the current page, shows.
    pub fn view(&self, src: &str) -> View {
        let note = |note, clickable| View::Note { note, clickable };
        let Some(resolved) = self.page_sources.get(src) else { return note(Note::Loading, false) };
        let Ok(location) = resolved else { return note(Note::Refused, false) };
        match self.table.get(location) {
            Some(Entry::Shown(picture)) => View::Shown { width: picture.width, height: picture.height },
            Some(Entry::AwaitingClick) => note(Note::ClickToLoad, true),
            Some(Entry::Off) => note(Note::Off, false),
            Some(Entry::Failed(problem)) => note(problem.note(), *problem == Problem::Unavailable),
            Some(Entry::Queued | Entry::Loading) | None => note(Note::Loading, false),
        }
    }

    /// The picture to draw for `src`, if it is shown.
    pub fn picture(&self, src: &str) -> Option<&Picture> {
        match self.table.get(self.location(src)?) {
            Some(Entry::Shown(picture)) => Some(picture),
            _ => None,
        }
    }

    /// Where `src`, on the current page, comes from: what a click on it loads.
    pub fn location(&self, src: &str) -> Option<&Location> {
        self.page_sources.get(src)?.as_ref().ok()
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
            (true, ImageLoading::OnClick) => self.set(location, Entry::AwaitingClick),
            (true, ImageLoading::Off) => self.set(location, Entry::Off),
        }
    }

    /// Queues a load once: a location already queued keeps its place.
    fn queue_load(&mut self, location: Location) {
        if !matches!(self.table.insert(location.clone(), Entry::Queued), Some(Entry::Queued)) {
            self.queue.push_back(location);
        }
    }

    /// Sets a state other than queued, taking the location out of the queue.
    fn set(&mut self, location: Location, entry: Entry) {
        if matches!(self.table.get(&location), Some(Entry::Queued)) {
            self.queue.retain(|queued| *queued != location);
        }
        self.table.insert(location, entry);
    }

    /// Starts queued loads while fewer than `MAX_CONCURRENT_LOADS` are running.
    fn start_queued(mut self) -> (Self, Vec<Location>) {
        let mut started = Vec::new();
        while self.loading < MAX_CONCURRENT_LOADS {
            let Some(location) = self.queue.pop_front() else { break };
            if matches!(self.table.get(&location), Some(Entry::Queued)) {
                self.table.insert(location.clone(), Entry::Loading);
                self.loading += 1;
                started.push(location);
            }
        }
        (self, started)
    }

    /// Over budget: drop the least recently used pictures the current page doesn't use.
    fn evict(&mut self) {
        if self.cache.within_budget(self.shown_bytes) {
            return;
        }
        let shown: Vec<(Location, u64)> =
            self.shown().map(|(location, picture)| (location.clone(), picture.pixel_bytes())).collect();
        for location in self.cache.victims(&shown, &self.pinned) {
            if let Some(Entry::Shown(picture)) = self.table.remove(&location) {
                self.shown_bytes -= picture.pixel_bytes();
            }
            self.cache.forget(&location);
        }
    }

    /// Bytes of every shown picture; only eviction makes it fall.
    #[cfg(test)]
    pub(crate) const fn shown_bytes(&self) -> u64 {
        self.shown_bytes
    }

    /// Checks the planner's own consistency, for tests: the queue holds exactly the
    /// queued entries, waiting/off/queued entries belong to the current page, the
    /// running totals match the table, the cache tracks only entries, and a
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
        assert_eq!(queued.len(), self.queue.len(), "nothing is queued twice");
        for (location, entry) in &self.table {
            if matches!(entry, Entry::Queued | Entry::AwaitingClick | Entry::Off) {
                assert!(self.pinned.contains(location), "{location:?} is {entry:?} but not on the page");
            }
        }
        let loading = self.table.values().filter(|entry| matches!(entry, Entry::Loading)).count();
        assert_eq!(self.loading, loading, "the running count matches the table");
        let shown: u64 = self.shown().map(|(_, picture)| picture.pixel_bytes()).sum();
        assert_eq!(self.shown_bytes, shown, "the shown total matches the table");
        assert!(
            self.cache.locations().all(|location| self.table.contains_key(location)),
            "the cache tracks only entries"
        );
        assert!(self.queue.is_empty() || self.loading == MAX_CONCURRENT_LOADS, "a queue never waits for a free slot");
    }
}

#[cfg(test)]
#[path = "planner_tests.rs"]
mod tests;
