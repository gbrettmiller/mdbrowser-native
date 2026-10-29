//! Images in a document: where each comes from, which load and when, and the
//! limits that bound them (ADR-004).

pub mod limits;
mod resolve;

pub use resolve::{Refusal, resolve};

/// How web images load. Local images always load.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ImageLoading {
    #[default]
    Automatic,
    OnClick,
    Off,
}

impl ImageLoading {
    pub const ALL: [Self; 3] = [Self::Automatic, Self::OnClick, Self::Off];

    /// The name in the settings file. Renaming a key forgets readers' choices.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Automatic => "automatic",
            Self::OnClick => "on-click",
            Self::Off => "off",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.key() == key)
    }
}

/// Why an image isn't shown: the second line under its description.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Note {
    Loading,
    ClickToLoad,
    Off,
    Failed,
    Unsupported,
    TooLarge,
    Refused,
}

impl Note {
    pub const ALL: [Self; 7] =
        [Self::Loading, Self::ClickToLoad, Self::Off, Self::Failed, Self::Unsupported, Self::TooLarge, Self::Refused];
}
