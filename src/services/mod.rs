//! All I/O lives here.

pub mod background;
pub mod files;
pub mod images;
pub mod opener;
pub mod runner;
pub mod store;
pub mod web;

/// The std-only HTTP server shared with the acceptance suite.
#[cfg(test)]
#[path = "../../tests/support/http.rs"]
mod test_server;
