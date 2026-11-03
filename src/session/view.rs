//! Which view of the page the reader has: the page drawn, or the Markdown it was made from.

use std::sync::Arc;

use crate::model::{Document, source_document};

use super::Shown;

/// The reader's view of the shown page. A momentary lens, so it is never saved, and never a
/// History entry. The source document is the view's own, so it cannot outlive the view.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum View {
    #[default]
    Page,
    /// The page's Markdown, as one code block.
    Source(Arc<Document>),
}

impl View {
    /// The other view of `shown`.
    pub fn toggled(&self, shown: &Shown) -> View {
        match self {
            View::Page => View::Source(Arc::new(source_document(&shown.source))),
            View::Source(_) => View::Page,
        }
    }
}
