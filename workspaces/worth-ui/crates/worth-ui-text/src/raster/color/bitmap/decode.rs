//! Bounded PNG/raw bitmap decoding for the qualified color lane.

use std::io::Cursor;

use super::super::super::capacity::MAX_RASTER_EDGE;
use super::super::super::denial::UiGlyphRasterizationDenial;

pub(super) enum UiBitmapPixels<'a> {
    Png(&'a [u8]),
    Bgra(&'a [u8]),
    LinearPremultipliedRgba(Vec<u8>),
}

pub(super) fn validate_bitmap(
    pixels: &UiBitmapPixels<'_>,
    width: u32,
    height: u32,
) -> Result<(), UiGlyphRasterizationDenial> {
    validate_bitmap_dimensions(width, height)?;
    match pixels {
        UiBitmapPixels::Png(bytes) => {
            if png_dimensions(bytes)? != (width, height) {
                return Err(UiGlyphRasterizationDenial::InvalidColorPixels);
            }
        }
        UiBitmapPixels::Bgra(bytes) => validate_bgra(bytes, width, height)?,
        UiBitmapPixels::LinearPremultipliedRgba(bytes) => validate_bgra(bytes, width, height)?,
    }
    Ok(())
}

pub(super) fn validate_bitmap_dimensions(
    width: u32,
    height: u32,
) -> Result<(), UiGlyphRasterizationDenial> {
    if width == 0 || height == 0 || width > MAX_RASTER_EDGE || height > MAX_RASTER_EDGE {
        return Err(UiGlyphRasterizationDenial::ExtentExceeded);
    }
    Ok(())
}

pub(super) fn validate_bgra(
    bytes: &[u8],
    width: u32,
    height: u32,
) -> Result<(), UiGlyphRasterizationDenial> {
    let expected = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?;
    if bytes.len() != expected {
        return Err(UiGlyphRasterizationDenial::InvalidColorPixels);
    }
    Ok(())
}

pub(super) fn png_dimensions(bytes: &[u8]) -> Result<(u32, u32), UiGlyphRasterizationDenial> {
    let mut decoder = png::Decoder::new_with_limits(
        Cursor::new(bytes),
        png::Limits {
            bytes: 2 * 1024 * 1024,
        },
    );
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let reader = decoder
        .read_info()
        .map_err(|_| UiGlyphRasterizationDenial::InvalidColorPixels)?;
    let info = reader.info();
    if info.width == 0
        || info.height == 0
        || info.width > MAX_RASTER_EDGE
        || info.height > MAX_RASTER_EDGE
        || info.animation_control.is_some()
        || reader.output_buffer_size().is_none()
    {
        return Err(UiGlyphRasterizationDenial::InvalidColorPixels);
    }
    Ok((info.width, info.height))
}

pub(super) fn decode_png(
    bytes: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, UiGlyphRasterizationDenial> {
    let mut decoder = png::Decoder::new_with_limits(
        Cursor::new(bytes),
        png::Limits {
            bytes: 2 * 1024 * 1024,
        },
    );
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder
        .read_info()
        .map_err(|_| UiGlyphRasterizationDenial::InvalidColorPixels)?;
    let mut output = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or(UiGlyphRasterizationDenial::InvalidColorPixels)?
    ];
    let info = reader
        .next_frame(&mut output)
        .map_err(|_| UiGlyphRasterizationDenial::InvalidColorPixels)?;
    if info.width != width || info.height != height {
        return Err(UiGlyphRasterizationDenial::InvalidColorPixels);
    }
    let data = &output[..info.buffer_size()];
    let mut rgba = Vec::with_capacity(pixel_bytes(width, height)?);
    match info.color_type {
        png::ColorType::Rgba => rgba.extend_from_slice(data),
        png::ColorType::Rgb => {
            for pixel in data.as_chunks::<3>().0 {
                rgba.extend_from_slice(&[pixel[0], pixel[1], pixel[2], 255]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for pixel in data.as_chunks::<2>().0 {
                rgba.extend_from_slice(&[pixel[0], pixel[0], pixel[0], pixel[1]]);
            }
        }
        png::ColorType::Grayscale => {
            for value in data {
                rgba.extend_from_slice(&[*value, *value, *value, 255]);
            }
        }
        png::ColorType::Indexed => return Err(UiGlyphRasterizationDenial::UnsupportedBitmapFormat),
    }
    reader
        .finish()
        .map_err(|_| UiGlyphRasterizationDenial::InvalidColorPixels)?;
    Ok(rgba)
}

fn pixel_bytes(width: u32, height: u32) -> Result<usize, UiGlyphRasterizationDenial> {
    usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)
}
