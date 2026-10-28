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
        todo!()
    }

    pub fn can_go_back(&self) -> bool {
        todo!()
    }

    pub fn can_go_forward(&self) -> bool {
        todo!()
    }

    /// Adds an entry after the current one, dropping anything forward of it.
    /// Pushing the current entry again is a no-op.
    pub fn push(self, _entry: Entry) -> Self {
        todo!()
    }

    pub fn back(self) -> Self {
        todo!()
    }

    pub fn forward(self) -> Self {
        todo!()
    }
}
