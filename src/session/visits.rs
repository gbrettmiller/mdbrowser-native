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
pub fn record(_visits: &[Visit], _address: &str, _title: &str, _now: u64) -> Vec<Visit> {
    todo!()
}

/// Reads stored history, dropping anything malformed rather than failing.
pub fn parse_stored(_raw: &str) -> Vec<Visit> {
    todo!()
}

pub fn to_stored(visits: &[Visit]) -> String {
    serde_json::to_string(visits).expect("visits serialise")
}
