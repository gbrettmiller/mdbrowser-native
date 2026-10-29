//! Reading or fetching an image and decoding it within ADR-004's limits. I/O and
//! decoding only: `session::images` decides which images load and when.

use std::io::Cursor;
use std::path::Path;
use std::sync::Mutex;

use iced::advanced::image::Handle;
use image::{ImageFormat, ImageReader, RgbaImage, imageops};

use super::files::{self, ReadError};
use super::web::{self, Limits};
use crate::session::images::limits::{MAX_BYTES, MAX_DECODED_WIDTH, MAX_PIXELS, MAX_SIDE, RGBA_BYTES_PER_PIXEL};
use crate::session::images::{Picture, Problem};
use crate::session::location::Location;
use crate::session::web::{Origin, Want};

/// Extensions a local image may have; the bytes still decide the format.
const IMAGE_EXTENSIONS: [&str; 5] = ["png", "jpg", "jpeg", "gif", "webp"];

// One cap for local and web images.
const _: () = assert!(MAX_BYTES == files::MAX_BYTES);

/// The formats ADR-004 decodes.
const FORMATS: [ImageFormat; 4] = [ImageFormat::Png, ImageFormat::Jpeg, ImageFormat::Gif, ImageFormat::WebP];

/// One decode at a time, so peak memory is one image's worth (ADR-004).
static DECODING: Mutex<()> = Mutex::new(());

/// Reads a local image or fetches a web one, then decodes it. `origin` is who asked
/// for it, for the connect-time check (ADR-004).
pub fn load(location: Location, origin: Origin) -> Result<Picture, Problem> {
    let bytes = match location {
        Location::File(path) => read(&path)?,
        Location::Web(url) => {
            let limits = Limits { max_bytes: MAX_BYTES, ..Limits::default() };
            web::fetch_bytes(&url, Want::Image, &limits, origin).map(|(_, bytes)| bytes).map_err(Problem::from_fetch)?
        }
    };
    decode(&bytes)
}

/// A local image: refused by extension before it is opened.
fn read(path: &Path) -> Result<Vec<u8>, Problem> {
    let extension = path.extension().and_then(|ext| ext.to_str()).map(str::to_ascii_lowercase);
    if !extension.is_some_and(|ext| IMAGE_EXTENSIONS.contains(&ext.as_str())) {
        return Err(Problem::Unsupported);
    }
    files::read_capped(path).map_err(|error| match error {
        ReadError::TooLarge => Problem::TooLarge,
        ReadError::NotFound | ReadError::Unreadable => Problem::Unavailable,
    })
}

/// Decodes PNG, JPEG, GIF (first frame) or WebP to RGBA, downscaled to at most
/// `MAX_DECODED_WIDTH`. The size in the header is checked before any pixels exist.
pub fn decode(bytes: &[u8]) -> Result<Picture, Problem> {
    let reader = || ImageReader::new(Cursor::new(bytes)).with_guessed_format().map_err(|_| Problem::Unsupported);
    let format = reader()?.format().ok_or(Problem::Unsupported)?;
    if !FORMATS.contains(&format) {
        return Err(Problem::Unsupported);
    }
    let (width, height) = reader()?.into_dimensions().map_err(|_| Problem::Unsupported)?;
    if width == 0 || height == 0 {
        return Err(Problem::Unsupported);
    }
    if width > MAX_SIDE || height > MAX_SIDE || u64::from(width) * u64::from(height) > MAX_PIXELS {
        return Err(Problem::TooLarge);
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_SIDE);
    limits.max_image_height = Some(MAX_SIDE);
    limits.max_alloc = Some(MAX_PIXELS * RGBA_BYTES_PER_PIXEL + MAX_BYTES);
    // Held through decoding, downscaling and conversion, so only one full-size
    // image is ever in memory; just the finished picture leaves the lock.
    let _one_at_a_time = DECODING.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut reader = reader()?;
    reader.limits(limits);
    let decoded = reader.decode().map_err(|error| match error {
        image::ImageError::Limits(_) => Problem::TooLarge,
        _ => Problem::Unsupported,
    })?;
    Ok(picture(downscaled(decoded.into_rgba8())))
}

/// Box-sampled straight into the output, with no wide intermediate buffer (a
/// filtered resize would hold the whole height in 16-byte pixels).
fn downscaled(image: RgbaImage) -> RgbaImage {
    if image.width() <= MAX_DECODED_WIDTH {
        return image;
    }
    let height = (f64::from(image.height()) * f64::from(MAX_DECODED_WIDTH) / f64::from(image.width())).round();
    imageops::thumbnail(&image, MAX_DECODED_WIDTH, (height as u32).max(1))
}

fn picture(rgba: RgbaImage) -> Picture {
    let (width, height) = rgba.dimensions();
    Picture { width, height, handle: Handle::from_rgba(width, height, rgba.into_raw()) }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::time::Duration;

    use image::codecs::gif::GifEncoder;
    use image::codecs::jpeg::JpegEncoder;
    use image::codecs::webp::WebPEncoder;
    use image::{Delay, DynamicImage, Frame, ImageFormat, Rgba, RgbaImage};
    use url::Url;

    use super::*;
    use crate::services::test_server::{Reply, Server};
    use crate::session::images::limits::{MAX_PIXELS, MAX_SIDE};

    fn png(width: u32, height: u32, image: DynamicImage) -> Vec<u8> {
        assert_eq!((image.width(), image.height()), (width, height));
        let mut bytes = Cursor::new(Vec::new());
        image.write_to(&mut bytes, ImageFormat::Png).unwrap();
        bytes.into_inner()
    }

    fn rgba(width: u32, height: u32) -> DynamicImage {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(width, height, Rgba([200, 30, 30, 128])))
    }

    /// A PNG that is only a header: it claims a size without any pixels, so
    /// limits can be tested without allocating what the header claims.
    fn png_header(width: u32, height: u32) -> Vec<u8> {
        let mut ihdr = b"IHDR".to_vec();
        ihdr.extend(width.to_be_bytes());
        ihdr.extend(height.to_be_bytes());
        ihdr.extend([8, 6, 0, 0, 0]); // 8-bit RGBA, no interlace
        let mut bytes = b"\x89PNG\r\n\x1a\n".to_vec();
        bytes.extend(13u32.to_be_bytes());
        bytes.extend(&ihdr);
        bytes.extend(crc32(&ihdr).to_be_bytes());
        // An empty compressed stream: the decoder reads the header at the first
        // IDAT, and there are no pixels behind it.
        let idat = [b"IDAT".as_slice(), &[0x78, 0x9c, 0x03, 0x00, 0x00, 0x00, 0x00, 0x01]].concat();
        bytes.extend(8u32.to_be_bytes());
        bytes.extend(&idat);
        bytes.extend(crc32(&idat).to_be_bytes());
        bytes.extend(0u32.to_be_bytes());
        bytes.extend(b"IEND");
        bytes.extend(crc32(b"IEND").to_be_bytes());
        bytes
    }

    fn crc32(bytes: &[u8]) -> u32 {
        !bytes.iter().fold(!0u32, |crc, &byte| {
            (0..8).fold(crc ^ u32::from(byte), |crc, _| if crc & 1 == 1 { (crc >> 1) ^ 0xedb8_8320 } else { crc >> 1 })
        })
    }

    fn size(result: Result<Picture, Problem>) -> Result<(u32, u32), Problem> {
        result.map(|picture| (picture.width, picture.height))
    }

    #[test]
    fn pngs_of_every_colour_type_decode() {
        assert_eq!(size(decode(&png(3, 2, rgba(3, 2)))), Ok((3, 2)));
        assert_eq!(size(decode(&png(3, 2, DynamicImage::new_rgb8(3, 2)))), Ok((3, 2)));
        assert_eq!(size(decode(&png(3, 2, DynamicImage::new_luma8(3, 2)))), Ok((3, 2)));
        assert_eq!(size(decode(&png(3, 2, DynamicImage::new_luma_a8(3, 2)))), Ok((3, 2)));
    }

    /// A 2×1 palette PNG, built by hand because the encoder only writes true colour.
    fn indexed_png() -> Vec<u8> {
        use std::io::Write as _;
        let chunk = |name: &[u8], data: &[u8]| {
            let body = [name, data].concat();
            [&(data.len() as u32).to_be_bytes()[..], &body, &crc32(&body).to_be_bytes()].concat()
        };
        let ihdr = [&2u32.to_be_bytes()[..], &1u32.to_be_bytes(), &[8, 3, 0, 0, 0]].concat();
        let mut zlib = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        zlib.write_all(&[0, 0, 1]).unwrap(); // filter byte, then palette indexes 0 and 1
        let idat = zlib.finish().unwrap();
        [
            b"\x89PNG\r\n\x1a\n".as_slice(),
            &chunk(b"IHDR", &ihdr),
            &chunk(b"PLTE", &[255, 0, 0, 0, 0, 255]),
            &chunk(b"IDAT", &idat),
            &chunk(b"IEND", &[]),
        ]
        .concat()
    }

    #[test]
    fn indexed_pngs_decode() {
        assert_eq!(size(decode(&indexed_png())), Ok((2, 1)));
    }

    #[test]
    fn zero_sized_images_are_unsupported() {
        assert_eq!(size(decode(&png_header(0, 4))), Err(Problem::Unsupported));
        assert_eq!(size(decode(&png_header(4, 0))), Err(Problem::Unsupported));
    }

    #[test]
    fn a_very_tall_wide_image_is_downscaled_without_a_wide_buffer() {
        assert_eq!(size(decode(&png(2_049, 3_000, rgba(2_049, 3_000)))), Ok((2_048, 2_999)));
    }

    #[test]
    fn gif_jpeg_and_webp_decode_with_gif_showing_its_first_frame() {
        let mut gif = Vec::new();
        let frames = [rgba(5, 4), rgba(5, 4)].map(|frame| {
            Frame::from_parts(frame.into_rgba8(), 0, 0, Delay::from_saturating_duration(Duration::from_millis(50)))
        });
        GifEncoder::new(&mut gif).encode_frames(frames).unwrap();
        assert_eq!(size(decode(&gif)), Ok((5, 4)));
        let mut jpeg = Vec::new();
        JpegEncoder::new(&mut jpeg).encode_image(&DynamicImage::new_rgb8(6, 4)).unwrap();
        assert_eq!(size(decode(&jpeg)), Ok((6, 4)));
        let mut webp = Vec::new();
        WebPEncoder::new_lossless(&mut webp)
            .encode(rgba(7, 3).as_bytes(), 7, 3, image::ExtendedColorType::Rgba8)
            .unwrap();
        assert_eq!(size(decode(&webp)), Ok((7, 3)));
    }

    #[test]
    fn other_formats_and_garbage_are_unsupported() {
        let svg = b"<svg xmlns='http://www.w3.org/2000/svg' width='4' height='4'/>".to_vec();
        let avif = b"\0\0\0\x1cftypavif\0\0\0\0avifmif1miaf".to_vec();
        for bytes in [svg, avif, b"not an image".to_vec(), Vec::new()] {
            assert_eq!(size(decode(&bytes)), Err(Problem::Unsupported), "{bytes:?}");
        }
    }

    #[test]
    fn sizes_up_to_the_limits_are_allowed_and_beyond_them_refused_before_decoding() {
        assert_ne!(size(decode(&png_header(MAX_SIDE, 1))), Err(Problem::TooLarge));
        assert_eq!(size(decode(&png_header(MAX_SIDE + 1, 1))), Err(Problem::TooLarge));
        assert_eq!(size(decode(&png_header(1, MAX_SIDE + 1))), Err(Problem::TooLarge));
        assert_ne!(size(decode(&png_header(5_000, 5_000))), Err(Problem::TooLarge));
        assert_eq!(MAX_PIXELS, 25_000_000);
        assert_eq!(size(decode(&png_header(5_000, 5_001))), Err(Problem::TooLarge));
        assert_eq!(size(decode(&png_header(16_000, 16_000))), Err(Problem::TooLarge));
    }

    #[test]
    fn wide_images_are_downscaled_when_decoded() {
        assert_eq!(size(decode(&png(4_000, 100, rgba(4_000, 100)))), Ok((2_048, 51)));
        assert_eq!(size(decode(&png(2_048, 10, rgba(2_048, 10)))), Ok((2_048, 10)));
    }

    #[test]
    fn local_images_are_read_by_extension_and_decoded() {
        let dir = tempfile::tempdir().unwrap();
        let file = |name: &str| Location::file(dir.path().join(name));
        std::fs::write(dir.path().join("pic.PNG"), png(3, 2, rgba(3, 2))).unwrap();
        assert_eq!(size(load(file("pic.PNG"), Origin::Trusted)), Ok((3, 2)));
        std::fs::write(dir.path().join("photo.jpeg"), png(3, 2, rgba(3, 2))).unwrap();
        assert_eq!(size(load(file("photo.jpeg"), Origin::Trusted)), Ok((3, 2)), "the bytes decide");
        assert_eq!(size(load(file("missing.png"), Origin::Trusted)), Err(Problem::Unavailable));
    }

    #[test]
    fn local_files_without_an_image_extension_are_never_read() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["notes.txt", "logo.svg", "pic"] {
            let missing = Location::file(dir.path().join(name));
            assert_eq!(size(load(missing, Origin::Trusted)), Err(Problem::Unsupported), "{name}");
        }
    }

    #[test]
    fn web_images_are_fetched_whatever_their_declared_type() {
        let bytes = png(4, 4, rgba(4, 4));
        let typed = bytes.clone();
        let server = Server::start(move |request| match request.path.as_str() {
            "/typed.png" => Reply::new(200).header("Content-Type", "image/png").body(typed.clone()),
            "/untyped.png" => Reply::new(200).body(typed.clone()),
            "/mislabel.png" => Reply::new(200).header("Content-Type", "image/gif").body(typed.clone()),
            "/moved.png" => Reply::redirect(302, "/typed.png"),
            "/page.png" => Reply::new(200).header("Content-Type", "text/html").body("<p>"),
            _ => Reply::new(404),
        });
        let web = |path: &str| Location::web(Url::parse(&server.url(path)).unwrap());
        for path in ["/typed.png", "/untyped.png", "/mislabel.png", "/moved.png"] {
            assert_eq!(size(load(web(path), Origin::Trusted)), Ok((4, 4)), "{path}");
        }
        assert_eq!(size(load(web("/page.png"), Origin::Trusted)), Err(Problem::Unsupported));
        assert_eq!(size(load(web("/missing.png"), Origin::Trusted)), Err(Problem::Unavailable));
        drop(bytes);
    }
}
