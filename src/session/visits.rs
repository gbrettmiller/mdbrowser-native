//! History: one entry per address, newest first, capped.

use serde::{Deserialize, Serialize};

pub const MAX_VISITS: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Visit {
    pub address: String,
    pub title: String,
    pub visited_at: u64,
}

/// Puts `address` on top (moving it if present) and trims to the cap.
pub fn record(visits: &[Visit], address: &str, title: &str, now: u64) -> Vec<Visit> {
    let newest = Visit { address: address.into(), title: title.into(), visited_at: now };
    [newest].into_iter().chain(visits.iter().filter(|v| v.address != address).cloned()).take(MAX_VISITS).collect()
}

/// Reads stored history, dropping anything malformed rather than failing.
pub fn parse_stored(raw: &str) -> Vec<Visit> {
    match serde_json::from_str::<serde_json::Value>(raw) {
        Ok(serde_json::Value::Array(items)) => {
            items.into_iter().filter_map(|item| serde_json::from_value(item).ok()).take(MAX_VISITS).collect()
        }
        _ => Vec::new(),
    }
}

pub fn to_stored(visits: &[Visit]) -> String {
    serde_json::to_string(visits).expect("visits serialise")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn visit(address: &str, at: u64) -> Visit {
        Visit { address: address.into(), title: address.to_uppercase(), visited_at: at }
    }

    fn addresses(visits: &[Visit]) -> Vec<&str> {
        visits.iter().map(|v| v.address.as_str()).collect()
    }

    #[test]
    fn newest_first_without_duplicates() {
        let visits = record(&[], "a", "A", 1);
        let visits = record(&visits, "b", "B", 2);
        let visits = record(&visits, "a", "A again", 3);
        assert_eq!(addresses(&visits), ["a", "b"]);
        assert_eq!(visits[0], Visit { address: "a".into(), title: "A again".into(), visited_at: 3 });
    }

    #[test]
    fn capped_at_the_maximum() {
        let full: Vec<Visit> = (0..MAX_VISITS as u64).map(|i| visit(&format!("d{i}"), MAX_VISITS as u64 - i)).collect();
        let added = record(&full, "new", "New", 1000);
        assert_eq!(added.len(), MAX_VISITS);
        assert_eq!(added[0].address, "new");
        assert!(!addresses(&added).contains(&format!("d{}", MAX_VISITS - 1).as_str()));
        let revisited = record(&full, "d250", "D250", 1000);
        assert_eq!(revisited.len(), MAX_VISITS);
        assert_eq!(revisited[0].address, "d250");
    }

    #[test]
    fn stored_history_round_trips_and_tolerates_damage() {
        let visits = vec![visit("a", 2), visit("b", 1)];
        assert_eq!(parse_stored(&to_stored(&visits)), visits);
        assert_eq!(parse_stored("not json"), vec![]);
        assert_eq!(parse_stored(r#"{"a": 1}"#), vec![]);
        let mixed = r#"[{"address":"a","title":"A","visited_at":1},{"address":3},"junk"]"#;
        assert_eq!(addresses(&parse_stored(mixed)), ["a"]);
    }
}
