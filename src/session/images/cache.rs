//! Which decoded pictures to keep: least recently used first out, never the
//! current page's, within a budget of pixel bytes (ADR-004).

use std::collections::{HashMap, HashSet};

use crate::session::location::Location;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Cache {
    budget: u64,
    clock: u64,
    last_used: HashMap<Location, u64>,
}

impl Cache {
    pub(super) fn new(budget: u64) -> Self {
        Cache { budget, clock: 0, last_used: HashMap::new() }
    }

    /// A page that uses `location` became ready, or its picture arrived.
    pub(super) fn touch(&mut self, location: &Location) {
        self.clock += 1;
        self.last_used.insert(location.clone(), self.clock);
    }

    pub(super) fn forget(&mut self, location: &Location) {
        self.last_used.remove(location);
    }

    /// Forgets every location `keep` rejects.
    pub(super) fn retain(&mut self, keep: impl Fn(&Location) -> bool) {
        self.last_used.retain(|location, _| keep(location));
    }

    pub(super) const fn within_budget(&self, bytes: u64) -> bool {
        bytes <= self.budget
    }

    /// Locations with a last use recorded.
    #[cfg(test)]
    pub(super) fn tracked(&self) -> usize {
        self.last_used.len()
    }

    #[cfg(test)]
    pub(super) fn locations(&self) -> impl Iterator<Item = &Location> {
        self.last_used.keys()
    }

    /// The shown pictures to drop, least recently used first, until the total is
    /// within budget. Pinned ones are never chosen, so they can hold it over.
    pub(super) fn victims(&self, shown: &[(Location, u64)], pinned: &HashSet<Location>) -> Vec<Location> {
        let mut total: u64 = shown.iter().map(|(_, bytes)| bytes).sum();
        let mut candidates: Vec<&(Location, u64)> =
            shown.iter().filter(|(location, _)| !pinned.contains(location)).collect();
        candidates.sort_by_key(|(location, _)| self.last_used.get(location).copied().unwrap_or(0));
        let mut victims = Vec::new();
        for (location, bytes) in candidates {
            if total <= self.budget {
                break;
            }
            total -= bytes;
            victims.push(location.clone());
        }
        victims
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(name: &str) -> Location {
        Location::file(format!("/x/{name}"))
    }

    #[test]
    fn the_least_recently_used_go_first_until_within_budget() {
        let mut cache = Cache::new(20);
        ["a", "b", "c"].iter().for_each(|name| cache.touch(&at(name)));
        cache.touch(&at("a"));
        let shown = [(at("a"), 10), (at("b"), 10), (at("c"), 10)];
        assert_eq!(cache.victims(&shown, &HashSet::new()), [at("b")]);
    }

    #[test]
    fn nothing_goes_within_budget_and_pinned_pictures_never_go() {
        let mut cache = Cache::new(30);
        ["a", "b", "c"].iter().for_each(|name| cache.touch(&at(name)));
        let shown = [(at("a"), 10), (at("b"), 10), (at("c"), 10)];
        assert!(cache.victims(&shown, &HashSet::new()).is_empty());
        let tight = Cache { budget: 5, ..cache };
        let pinned: HashSet<Location> = [at("a")].into();
        assert_eq!(tight.victims(&shown, &pinned), [at("b"), at("c")]);
    }
}
