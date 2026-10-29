//! Images in a document: where each comes from, which load and when, and the
//! limits that bound them (ADR-004).

pub mod limits;
mod resolve;

pub use resolve::{Refusal, resolve};
