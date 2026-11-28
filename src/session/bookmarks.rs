//! Bookmarks: one per address, in the reader's own order. New ones go at the end,
//! and each can move up or down.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bookmark {
    pub address: String,
    /// The page's title when it was bookmarked.
    pub title: String,
}

/// Which way a bookmark moves in the list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Up,
    Down,
}

pub fn contains(bookmarks: &[Bookmark], address: &str) -> bool {
    bookmarks.iter().any(|b| b.address == address)
}

/// Removes `address` if it is bookmarked, otherwise adds it at the end.
pub fn toggle(bookmarks: &[Bookmark], address: &str, title: &str) -> Vec<Bookmark> {
    if contains(bookmarks, address) {
        return remove(bookmarks, address);
    }
    let added = Bookmark { address: address.into(), title: title.into() };
    bookmarks.iter().cloned().chain([added]).collect()
}

pub fn remove(bookmarks: &[Bookmark], address: &str) -> Vec<Bookmark> {
    bookmarks.iter().filter(|b| b.address != address).cloned().collect()
}

/// Swaps `address` with its neighbour that way; at either end, or if absent, nothing changes.
pub fn shift(bookmarks: &[Bookmark], address: &str, by: Move) -> Vec<Bookmark> {
    let mut moved = bookmarks.to_vec();
    let Some(at) = bookmarks.iter().position(|b| b.address == address) else { return moved };
    let other = match by {
        Move::Up => at.checked_sub(1),
        Move::Down => Some(at + 1).filter(|&next| next < bookmarks.len()),
    };
    if let Some(other) = other {
        moved.swap(at, other);
    }
    moved
}

/// Reads stored bookmarks in their order, dropping anything malformed or repeated rather than failing.
pub fn parse_stored(raw: &str) -> Vec<Bookmark> {
    let Ok(serde_json::Value::Array(items)) = serde_json::from_str::<serde_json::Value>(raw) else {
        return Vec::new();
    };
    let mut seen = std::collections::HashSet::new();
    items
        .into_iter()
        .filter_map(|item| serde_json::from_value::<Bookmark>(item).ok())
        .filter(|b| seen.insert(b.address.clone()))
        .collect()
}

pub fn to_stored(bookmarks: &[Bookmark]) -> String {
    serde_json::to_string(bookmarks).expect("bookmarks serialise")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(addresses: &[&str]) -> Vec<Bookmark> {
        addresses.iter().fold(Vec::new(), |list, a| toggle(&list, a, &a.to_uppercase()))
    }

    fn addresses(bookmarks: &[Bookmark]) -> Vec<&str> {
        bookmarks.iter().map(|b| b.address.as_str()).collect()
    }

    #[test]
    fn new_bookmarks_go_at_the_end() {
        let added = toggle(&list(&["a", "b"]), "c", "C");
        assert_eq!(addresses(&added), ["a", "b", "c"]);
        assert_eq!(added[2], Bookmark { address: "c".into(), title: "C".into() });
    }

    #[test]
    fn toggling_a_bookmarked_address_removes_it() {
        assert_eq!(addresses(&toggle(&list(&["a", "b", "c"]), "b", "B")), ["a", "c"]);
    }

    #[test]
    fn remove_takes_out_only_that_address() {
        assert_eq!(addresses(&remove(&list(&["a", "b"]), "a")), ["b"]);
        assert_eq!(addresses(&remove(&list(&["a"]), "z")), ["a"]);
    }

    #[test]
    fn moving_swaps_with_the_neighbour() {
        let three = list(&["a", "b", "c"]);
        assert_eq!(addresses(&shift(&three, "b", Move::Up)), ["b", "a", "c"]);
        assert_eq!(addresses(&shift(&three, "b", Move::Down)), ["a", "c", "b"]);
    }

    #[test]
    fn moving_past_an_end_or_an_unknown_address_changes_nothing() {
        let three = list(&["a", "b", "c"]);
        assert_eq!(shift(&three, "a", Move::Up), three);
        assert_eq!(shift(&three, "c", Move::Down), three);
        assert_eq!(shift(&three, "z", Move::Up), three);
        assert_eq!(shift(&[], "a", Move::Down), []);
    }

    #[test]
    fn stored_bookmarks_round_trip_in_order() {
        let three = shift(&list(&["a", "b", "c"]), "c", Move::Up);
        assert_eq!(parse_stored(&to_stored(&three)), three);
    }

    #[test]
    fn malformed_or_repeated_stored_entries_are_dropped() {
        let raw = r#"[{"address":"a","title":"A"},{"address":"b"},7,
                      {"address":"a","title":"A again"},{"address":"c","title":"C"}]"#;
        let parsed = parse_stored(raw);
        assert_eq!(addresses(&parsed), ["a", "c"]);
        assert_eq!(parsed[0].title, "A");
        assert!(parse_stored("not json").is_empty());
        assert!(parse_stored(r#"{"address":"a"}"#).is_empty());
    }
}
