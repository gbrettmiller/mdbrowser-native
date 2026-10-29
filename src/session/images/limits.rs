//! The image limits (ADR-004). The plan, the tests and the ADR quote these.

/// Largest file or response, counted after decompression (as for pages, ADR-003).
pub const MAX_BYTES: u64 = 16 * 1024 * 1024;
/// Largest side, in pixels, before decoding.
pub const MAX_SIDE: u32 = 16_384;
/// Most pixels, before decoding.
pub const MAX_PIXELS: u64 = 25_000_000;
/// Wider images are downscaled to this width when decoded.
pub const MAX_DECODED_WIDTH: u32 = 2_048;
/// Image loads running at once, across the app.
pub const MAX_LOADING: usize = 4;
/// Decoded pixels kept in memory, in bytes of RGBA.
pub const CACHE_BYTES: u64 = 256 * 1024 * 1024;
