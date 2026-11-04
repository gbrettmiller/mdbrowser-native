//! The app's own icon, drawn into the window's title bar and taskbar entry.

use iced::window;

/// The 256 px PNG made from `assets/icon/icon.svg` by `scripts/icons.sh`.
const BUNDLED: &[u8] = include_bytes!("../../assets/icon/icon-256.png");

/// The icon as RGBA pixels and its side in pixels (it is square), or `None` if it cannot be read.
fn decode(png: &[u8]) -> Option<(Vec<u8>, u32)> {
    let image = image::load_from_memory_with_format(png, image::ImageFormat::Png).ok()?.into_rgba8();
    let side = image.width();
    (side == image.height()).then(|| (image.into_raw(), side))
}

/// The window icon. Without it a window shows the system's generic one. `None` only if the bundled file is damaged.
pub fn window_icon() -> Option<window::Icon> {
    let (rgba, side) = decode(BUNDLED)?;
    window::icon::from_rgba(rgba, side, side).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bundled_icon_decodes_to_a_square_with_some_ink_and_some_transparent_corners() {
        let (rgba, side) = decode(BUNDLED).expect("the bundled icon decodes");
        assert_eq!(side, 256);
        assert_eq!(rgba.len(), (side * side * 4) as usize);
        assert_eq!(rgba[3], 0, "the top-left corner is transparent");
        let middle = ((side / 2 * side + side / 2) * 4 + 3) as usize;
        assert_eq!(rgba[middle], 255, "the centre is opaque");
        assert!(window_icon().is_some());
    }
}
