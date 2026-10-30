//! The reader's size and spacing choices: a stepped text size and three spacings.

/// The body size in whole pixels: 12 to 32 in steps of 2, 18 by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextSize(u8);

impl TextSize {
    const MIN: u8 = 12;
    const MAX: u8 = 32;
    const STEP: u8 = 2;

    pub const fn px(self) -> u8 {
        self.0
    }

    /// The size for a stored or typed pixel value, if it is a step on the scale.
    pub fn from_px(px: u64) -> Option<Self> {
        let px = u8::try_from(px).ok()?;
        (Self::MIN..=Self::MAX).contains(&px).then_some(px).filter(|px| (px - Self::MIN) % Self::STEP == 0).map(Self)
    }

    /// One step larger, stopping at the largest.
    pub const fn larger(self) -> Self {
        Self(if self.0 + Self::STEP > Self::MAX { Self::MAX } else { self.0 + Self::STEP })
    }

    /// One step smaller, stopping at the smallest.
    pub const fn smaller(self) -> Self {
        Self(if self.0 < Self::MIN + Self::STEP { Self::MIN } else { self.0 - Self::STEP })
    }
}

impl Default for TextSize {
    fn default() -> Self {
        Self(18)
    }
}

/// How airy the page is: line height and the gaps between blocks and list items together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Spacing {
    Compact,
    #[default]
    Comfortable,
    Spacious,
}

impl Spacing {
    pub const ALL: [Spacing; 3] = [Self::Compact, Self::Comfortable, Self::Spacious];

    /// Stable identifier for settings files.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Comfortable => "comfortable",
            Self::Spacious => "spacious",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|spacing| spacing.key() == key)
    }

    /// Line height as a multiple of the body size.
    pub const fn line_ratio(self) -> f32 {
        match self {
            Self::Compact => 1.45,
            Self::Comfortable => 1.6,
            Self::Spacious => 1.8,
        }
    }

    /// The gap between blocks, in pixels.
    pub const fn block_gap(self) -> f32 {
        match self {
            Self::Compact => 12.0,
            Self::Comfortable => 16.0,
            Self::Spacious => 22.0,
        }
    }

    /// The gap between list items, in pixels.
    pub const fn item_gap(self) -> f32 {
        match self {
            Self::Compact => 2.0,
            Self::Comfortable => 4.0,
            Self::Spacious => 8.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_size_is_eighteen_and_steps_stop_at_both_ends() {
        assert_eq!(TextSize::default().px(), 18);
        let largest = (0..40).fold(TextSize::default(), |size, _| size.larger());
        let smallest = (0..40).fold(TextSize::default(), |size, _| size.smaller());
        assert_eq!((smallest.px(), largest.px()), (12, 32));
        assert_eq!((largest.larger(), smallest.smaller()), (largest, smallest));
        assert_eq!((TextSize::default().larger().px(), TextSize::default().smaller().px()), (20, 16));
    }

    #[test]
    fn only_whole_steps_inside_the_scale_are_sizes() {
        for px in (12..=32).step_by(2) {
            assert_eq!(TextSize::from_px(px).map(|size| u64::from(size.px())), Some(px));
        }
        for px in [0, 10, 13, 17, 33, 34, 999, u64::MAX] {
            assert_eq!(TextSize::from_px(px), None, "{px}");
        }
    }

    #[test]
    fn spacing_keys_round_trip_and_unknown_keys_are_rejected() {
        for spacing in Spacing::ALL {
            assert_eq!(Spacing::from_key(spacing.key()), Some(spacing));
        }
        assert_eq!(Spacing::from_key("tight"), None);
        assert_eq!(Spacing::default(), Spacing::Comfortable);
    }
}
