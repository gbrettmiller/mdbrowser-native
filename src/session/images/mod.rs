//! Images in a document: where each comes from, which load and when, and the
//! limits that bound them (ADR-004).

mod cache;
pub mod limits;
mod planner;
mod resolve;

pub use planner::{Images, View};
pub use resolve::{Refusal, resolve};

use iced::advanced::image::Handle;

use super::web::FetchError;

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

/// Why an image couldn't be shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Problem {
    /// Network, server, timeout or a broken body: worth trying again.
    Unavailable,
    /// A type or format we don't show, or bytes that don't decode.
    Unsupported,
    /// Over ADR-004's limits.
    TooLarge,
    /// A source or redirect the rules refuse (ADR-003, ADR-004).
    Refused,
}

impl Problem {
    pub const fn from_fetch(error: FetchError) -> Self {
        match error {
            FetchError::NotFound | FetchError::Server | FetchError::Unreachable | FetchError::Undecodable => {
                Self::Unavailable
            }
            FetchError::BadRedirect | FetchError::LocalAddress => Self::Refused,
            FetchError::TooLarge => Self::TooLarge,
            FetchError::WrongType => Self::Unsupported,
        }
    }

    pub const fn note(self) -> Note {
        match self {
            Self::Unavailable => Note::Failed,
            Self::Unsupported => Note::Unsupported,
            Self::TooLarge => Note::TooLarge,
            Self::Refused => Note::Refused,
        }
    }
}

/// A decoded image, ready to draw. The handle shares its pixels when cloned.
#[derive(Debug, Clone)]
pub struct Picture {
    pub width: u32,
    pub height: u32,
    pub handle: Handle,
}

impl Picture {
    /// Bytes of RGBA held in memory, for the cache budget.
    pub const fn pixel_bytes(&self) -> u64 {
        self.width as u64 * self.height as u64 * limits::RGBA_BYTES_PER_PIXEL
    }
}

/// Pictures are equal when they share pixels; comparing the pixels would be slow.
impl PartialEq for Picture {
    fn eq(&self, other: &Self) -> bool {
        self.handle.id() == other.handle.id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::web::FetchError;

    #[test]
    fn fetch_failures_become_image_problems() {
        for (error, problem) in [
            (FetchError::NotFound, Problem::Unavailable),
            (FetchError::Server, Problem::Unavailable),
            (FetchError::Unreachable, Problem::Unavailable),
            (FetchError::BadRedirect, Problem::Refused),
            (FetchError::LocalAddress, Problem::Refused),
            (FetchError::TooLarge, Problem::TooLarge),
            (FetchError::WrongType, Problem::Unsupported),
            (FetchError::Undecodable, Problem::Unavailable),
        ] {
            assert_eq!(Problem::from_fetch(error), problem, "{error:?}");
        }
    }

    #[test]
    fn each_problem_has_its_note() {
        assert_eq!(Problem::Unavailable.note(), Note::Failed);
        assert_eq!(Problem::Unsupported.note(), Note::Unsupported);
        assert_eq!(Problem::TooLarge.note(), Note::TooLarge);
        assert_eq!(Problem::Refused.note(), Note::Refused);
    }
}
