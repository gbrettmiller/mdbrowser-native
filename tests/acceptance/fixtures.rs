//! Images generated for the scenarios, so no binary fixtures are checked in.

use std::io::Cursor;
use std::time::Duration;

use image::codecs::gif::GifEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::webp::WebPEncoder;
use image::{Delay, DynamicImage, ExtendedColorType, Frame, ImageFormat, Rgba, RgbaImage};

use crate::http::Reply;

fn rgba(width: u32, height: u32, alpha: u8) -> DynamicImage {
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(width, height, Rgba([200, 40, 40, alpha])))
}

pub fn png(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    rgba(width, height, 255).write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

fn png_with_alpha(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    rgba(width, height, 100).write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

fn gif(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let frames = [rgba(width, height, 255), rgba(width, height, 255)].map(|frame| {
        Frame::from_parts(frame.into_rgba8(), 0, 0, Delay::from_saturating_duration(Duration::from_millis(50)))
    });
    GifEncoder::new(&mut bytes).encode_frames(frames).unwrap();
    bytes
}

fn jpeg(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    JpegEncoder::new(&mut bytes).encode_image(&DynamicImage::new_rgb8(width, height)).unwrap();
    bytes
}

fn webp(width: u32, height: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    let pixels = rgba(width, height, 255);
    WebPEncoder::new_lossless(&mut bytes).encode(pixels.as_bytes(), width, height, ExtendedColorType::Rgba8).unwrap();
    bytes
}

/// A PNG that only claims a size: its header, an empty image stream, the end.
fn png_header(width: u32, height: u32) -> Vec<u8> {
    let chunk = |name: &[u8], data: &[u8]| {
        let body = [name, data].concat();
        [&(data.len() as u32).to_be_bytes()[..], &body, &crc32(&body).to_be_bytes()].concat()
    };
    let ihdr = [&width.to_be_bytes()[..], &height.to_be_bytes(), &[8, 6, 0, 0, 0]].concat();
    let idat = [0x78, 0x9c, 0x03, 0x00, 0x00, 0x00, 0x00, 0x01];
    [b"\x89PNG\r\n\x1a\n".as_slice(), &chunk(b"IHDR", &ihdr), &chunk(b"IDAT", &idat), &chunk(b"IEND", &[])].concat()
}

fn crc32(bytes: &[u8]) -> u32 {
    !bytes.iter().fold(!0u32, |crc, &byte| {
        (0..8).fold(crc ^ u32::from(byte), |crc, _| if crc & 1 == 1 { (crc >> 1) ^ 0xedb8_8320 } else { crc >> 1 })
    })
}

/// The image routes listed at the top of features/images.feature (`/img/flaky.png`
/// is served by the world, which counts its requests).
pub fn image_routes() -> Vec<(&'static str, Reply)> {
    let typed = |content_type: &str, body: Vec<u8>| Reply::new(200).header("Content-Type", content_type).body(body);
    vec![
        ("/img/red.png", typed("image/png", png(40, 20))),
        ("/img/alpha.png", typed("image/png", png_with_alpha(8, 8))),
        ("/img/wide.png", typed("image/png", png(4_000, 100))),
        ("/img/anim.gif", typed("image/gif", gif(30, 30))),
        ("/img/photo.jpg", typed("image/jpeg", jpeg(24, 24))),
        ("/img/photo.webp", typed("image/webp", webp(24, 24))),
        ("/img/untyped.png", Reply::new(200).body(png(10, 10))),
        ("/img/mislabel.png", typed("image/gif", png(10, 10))),
        ("/img/logo.svg", typed("image/svg+xml", b"<svg xmlns='http://www.w3.org/2000/svg'/>".to_vec())),
        ("/img/page.png", typed("text/html", b"<p>not an image</p>".to_vec())),
        ("/img/moved.png", Reply::redirect(302, "/img/red.png")),
        ("/img/to-file.png", Reply::redirect(302, "file:///etc/hosts")),
        ("/img/tall.png", typed("image/png", png_header(16_385, 1))),
        ("/img/huge.png", typed("image/png", png_header(6_000, 6_000))),
    ]
}
