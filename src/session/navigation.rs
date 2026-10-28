//! The back/forward stack. Entries keep what was typed, so an address that
//! failed to parse can still be revisited (and fail again) like any other.

use super::location::Target;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub input: String,
    pub target: Option<Target>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Navigation {
    entries: Vec<Entry>,
    index: usize,
}

impl Navigation {
    pub fn current(&self) -> Option<&Entry> {
        self.entries.get(self.index)
    }

    pub fn can_go_back(&self) -> bool {
        self.index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.index + 1 < self.entries.len()
    }

    /// Adds an entry after the current one, dropping anything forward of it.
    /// Pushing the current entry again is a no-op.
    pub fn push(self, entry: Entry) -> Self {
        if self.current() == Some(&entry) {
            return self;
        }
        let kept = if self.entries.is_empty() { 0 } else { self.index + 1 };
        let entries: Vec<Entry> = self.entries.into_iter().take(kept).chain([entry]).collect();
        Navigation { index: entries.len() - 1, entries }
    }

    pub fn back(self) -> Self {
        Navigation { index: self.index.saturating_sub(1), ..self }
    }

    pub fn forward(self) -> Self {
        let index = if self.can_go_forward() { self.index + 1 } else { self.index };
        Navigation { index, ..self }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(input: &str) -> Entry {
        Entry { input: input.into(), target: None }
    }

    fn inputs(nav: &Navigation) -> Vec<&str> {
        nav.entries.iter().map(|e| e.input.as_str()).collect()
    }

    #[test]
    fn a_fresh_stack_has_nowhere_to_go() {
        let nav = Navigation::default();
        assert_eq!(nav.current(), None);
        assert!(!nav.can_go_back() && !nav.can_go_forward());
        assert_eq!(nav.clone().back(), nav);
    }

    #[test]
    fn back_and_forward_move_within_bounds() {
        let nav = Navigation::default().push(entry("a")).push(entry("b"));
        assert_eq!(nav.current(), Some(&entry("b")));
        let nav = nav.back();
        assert_eq!(nav.current(), Some(&entry("a")));
        assert!(!nav.can_go_back() && nav.can_go_forward());
        assert_eq!(nav.clone().back().current(), Some(&entry("a")));
        assert_eq!(nav.forward().current(), Some(&entry("b")));
    }

    #[test]
    fn pushing_after_going_back_drops_forward_entries() {
        let nav = Navigation::default().push(entry("a")).push(entry("b")).back().push(entry("c"));
        assert_eq!(inputs(&nav), ["a", "c"]);
        assert!(!nav.can_go_forward());
    }

    #[test]
    fn pushing_the_current_entry_is_a_no_op() {
        let nav = Navigation::default().push(entry("a")).push(entry("a"));
        assert_eq!(inputs(&nav), ["a"]);
    }
}
