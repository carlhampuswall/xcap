use core_graphics::{
    display::{kCGWindowImageDefault, CGWindowID, CGWindowListOption},
    geometry::CGRect,
    window::create_image,
};
use image::{RgbImage, RgbaImage};

use crate::error::{XCapError, XCapResult};

pub fn capture(
    cg_rect: CGRect,
    list_option: CGWindowListOption,
    window_id: CGWindowID,
) -> XCapResult<RgbaImage> {
    let cg_image = create_image(cg_rect, list_option, window_id, kCGWindowImageDefault)
        .ok_or_else(|| XCapError::new(format!("Capture failed {} {:?}", window_id, cg_rect)))?;

    let width = cg_image.width();
    let height = cg_image.height();
    let bytes = Vec::from(cg_image.data().bytes());

    // Some platforms e.g. MacOS can have extra bytes at the end of each row.
    // See
    // https://github.com/nashaofu/xcap/issues/29
    // https://github.com/nashaofu/xcap/issues/38
    let mut buffer = Vec::with_capacity(width * height * 4);
    for row in bytes.chunks_exact(cg_image.bytes_per_row()) {
        buffer.extend_from_slice(&row[..width * 4]);
    }

    for bgra in buffer.chunks_exact_mut(4) {
        bgra.swap(0, 2);
    }

    RgbaImage::from_raw(width as u32, height as u32, buffer)
        .ok_or_else(|| XCapError::new("RgbaImage::from_raw failed"))
}

pub fn capture_rgb(
    cg_rect: CGRect,
    list_option: CGWindowListOption,
    window_id: CGWindowID,
) -> XCapResult<RgbImage> {
    let cg_image = create_image(cg_rect, list_option, window_id, kCGWindowImageDefault)
        .ok_or_else(|| XCapError::new(format!("Capture failed {} {:?}", window_id, cg_rect)))?;

    let width = cg_image.width();
    let height = cg_image.height();
    let bytes = Vec::from(cg_image.data().bytes());

    // Adjust row bytes to handle platform-specific padding
    let mut rgba_buffer = Vec::with_capacity(width * height * 4);
    for row in bytes.chunks_exact(cg_image.bytes_per_row()) {
        rgba_buffer.extend_from_slice(&row[..width * 4]); // Extract valid pixel data
    }

    // Convert RGBA (4 bytes per pixel) to RGB (3 bytes per pixel)
    let mut rgb_buffer = Vec::with_capacity(width * height * 3);
    for bgra in rgba_buffer.chunks_exact(4) {
        rgb_buffer.push(bgra[2]); // R
        rgb_buffer.push(bgra[1]); // G
        rgb_buffer.push(bgra[0]); // B
    }

    // Create an RgbImage with the RGB buffer
    RgbImage::from_raw(width as u32, height as u32, rgb_buffer)
        .ok_or_else(|| XCapError::new("RgbImage::from_raw failed"))
}
