//! Canonical intrinsic-color pixels and deterministic linear-premultiplied math.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use swash::zeno::Placement;
use worth_ui_host_contract::{
    UiGlyphRasterBearing, UiGlyphRasterContentDigest, UiGlyphRasterExtent,
};

use super::super::denial::UiGlyphRasterizationDenial;

#[derive(Clone, Copy)]
pub(super) enum UiColorPixelEncoding {
    StraightRgba,
    PremultipliedBgra,
    LinearPremultipliedRgba,
}

pub(super) struct UiCanonicalColorImage {
    pub(super) bearing: UiGlyphRasterBearing,
    pub(super) extent: UiGlyphRasterExtent,
    pub(super) pixels: Arc<[u8]>,
    pub(super) digest: UiGlyphRasterContentDigest,
}

pub(super) fn canonicalize_pixels(
    pixels: &[u8],
    encoding: UiColorPixelEncoding,
) -> Result<Vec<u8>, UiGlyphRasterizationDenial> {
    if !pixels.len().is_multiple_of(4) {
        return Err(UiGlyphRasterizationDenial::InvalidColorPixels);
    }
    let mut output = Vec::with_capacity(pixels.len());
    for pixel in pixels.as_chunks::<4>().0 {
        let alpha = pixel[3];
        let (channels, premultiplied) = match encoding {
            UiColorPixelEncoding::StraightRgba => ([pixel[0], pixel[1], pixel[2]], false),
            UiColorPixelEncoding::PremultipliedBgra => ([pixel[2], pixel[1], pixel[0]], true),
            UiColorPixelEncoding::LinearPremultipliedRgba => {
                output.extend_from_slice(pixel);
                continue;
            }
        };
        for channel in channels {
            output.push(linear_premultiplied(channel, alpha, premultiplied));
        }
        output.push(alpha);
    }
    Ok(output)
}

pub(super) fn bilinear_resample(
    pixels: &[u8],
    size: UiResampleSize,
) -> Result<Vec<u8>, UiGlyphRasterizationDenial> {
    let source_bytes = pixel_bytes(size.source_width, size.source_height)?;
    let target_bytes = pixel_bytes(size.target_width, size.target_height)?;
    if pixels.len() != source_bytes {
        return Err(UiGlyphRasterizationDenial::InvalidColorPixels);
    }
    let mut output = vec![0; target_bytes];
    for y in 0..size.target_height {
        for x in 0..size.target_width {
            let destination = usize::try_from((y * size.target_width + x) * 4).unwrap();
            output[destination..destination + 4]
                .copy_from_slice(&resampled_pixel(pixels, size, x, y));
        }
    }
    Ok(output)
}

fn resampled_pixel(pixels: &[u8], size: UiResampleSize, x: u32, y: u32) -> [u8; 4] {
    let source_y = source_coordinate(y, size.source_height, size.target_height);
    let y0 = source_y.floor() as u32;
    let y1 = y0.min(size.source_height.saturating_sub(1));
    let y2 = (y0 + 1).min(size.source_height.saturating_sub(1));
    let fy = source_y - y0 as f64;
    let source_x = source_coordinate(x, size.source_width, size.target_width);
    let x0 = source_x.floor() as u32;
    let x1 = x0.min(size.source_width.saturating_sub(1));
    let x2 = (x0 + 1).min(size.source_width.saturating_sub(1));
    let fx = source_x - x0 as f64;
    core::array::from_fn(|channel| {
        let top = interpolate(
            sample(
                pixels,
                UiPixelPosition {
                    width: size.source_width,
                    x: x1,
                    y: y1,
                },
                channel,
            ),
            sample(
                pixels,
                UiPixelPosition {
                    width: size.source_width,
                    x: x2,
                    y: y1,
                },
                channel,
            ),
            fx,
        );
        let bottom = interpolate(
            sample(
                pixels,
                UiPixelPosition {
                    width: size.source_width,
                    x: x1,
                    y: y2,
                },
                channel,
            ),
            sample(
                pixels,
                UiPixelPosition {
                    width: size.source_width,
                    x: x2,
                    y: y2,
                },
                channel,
            ),
            fx,
        );
        interpolate(f64::from(top), f64::from(bottom), fy)
    })
}

pub(super) fn scaled_dimension(value: u32, scale: f32) -> Result<u32, UiGlyphRasterizationDenial> {
    let scaled = f64::from(value) * f64::from(scale);
    if !scaled.is_finite() || scaled <= 0.0 || scaled > f64::from(u32::MAX) {
        return Err(UiGlyphRasterizationDenial::ExtentExceeded);
    }
    Ok((scaled as u32).max(1))
}

pub(super) fn finish_image(
    placement: Placement,
    mut pixels: Vec<u8>,
) -> Result<UiCanonicalColorImage, UiGlyphRasterizationDenial> {
    let extent = UiGlyphRasterExtent::new(placement.width, placement.height)
        .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?;
    let bearing = UiGlyphRasterBearing::from_sixty_fourths(
        placement
            .left
            .checked_mul(64)
            .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?,
        placement
            .top
            .checked_mul(64)
            .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?,
    );
    encode_linear_premultiplied_for_srgb_texture(&mut pixels)?;
    let digest = UiGlyphRasterContentDigest::from_text_mechanics(Sha256::digest(&pixels).into());
    Ok(UiCanonicalColorImage {
        bearing,
        extent,
        pixels: Arc::from(pixels),
        digest,
    })
}

pub(super) fn finish_linear_image(
    placement: UiLinearImagePlacement,
    mut pixels: Vec<u8>,
) -> Result<UiCanonicalColorImage, UiGlyphRasterizationDenial> {
    let expected = pixel_bytes(placement.width, placement.height)?;
    if pixels.len() != expected {
        return Err(UiGlyphRasterizationDenial::InvalidColorPixels);
    }
    let extent = UiGlyphRasterExtent::new(placement.width, placement.height)
        .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?;
    let bearing = UiGlyphRasterBearing::from_sixty_fourths(
        placement
            .left
            .checked_mul(64)
            .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?,
        placement
            .top
            .checked_mul(64)
            .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?,
    );
    encode_linear_premultiplied_for_srgb_texture(&mut pixels)?;
    let digest = UiGlyphRasterContentDigest::from_text_mechanics(Sha256::digest(&pixels).into());
    Ok(UiCanonicalColorImage {
        bearing,
        extent,
        pixels: Arc::from(pixels),
        digest,
    })
}

pub(super) fn srgb_channel_to_linear(value: u8) -> f64 {
    f64::from(srgb_to_linear_u16(value)) / 65_535.0
}

pub(super) fn pixels_per_em(key: worth_ui_host_contract::UiGlyphRasterKey) -> f32 {
    (key.size().millipoints() as f64 * f64::from(key.dpi_milli()) / 1_000_000.0) as f32
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

fn linear_premultiplied(channel: u8, alpha: u8, input_is_premultiplied: bool) -> u8 {
    let channel = if input_is_premultiplied {
        unpremultiply_srgb(channel, alpha)
    } else {
        channel
    };
    let linear = srgb_to_linear_u16(channel);
    let premultiplied = u32::from(linear) * u32::from(alpha);
    u8::try_from((premultiplied + 32_767) / 65_535).unwrap_or(u8::MAX)
}

fn unpremultiply_srgb(channel: u8, alpha: u8) -> u8 {
    if alpha == 0 {
        return 0;
    }
    let restored = (u32::from(channel) * 255 + u32::from(alpha) / 2) / u32::from(alpha);
    u8::try_from(restored.min(255)).unwrap_or(u8::MAX)
}

fn srgb_to_linear_u16(value: u8) -> u16 {
    let normalized = f64::from(value) / 255.0;
    let linear = if normalized <= 0.04045 {
        normalized / 12.92
    } else {
        ((normalized + 0.055) / 1.055).powf(2.4)
    };
    (linear * 65_535.0 + 0.5) as u16
}

fn encode_linear_premultiplied_for_srgb_texture(
    pixels: &mut [u8],
) -> Result<(), UiGlyphRasterizationDenial> {
    if !pixels.len().is_multiple_of(4) {
        return Err(UiGlyphRasterizationDenial::InvalidColorPixels);
    }
    for pixel in pixels.as_chunks_mut::<4>().0 {
        for channel in &mut pixel[..3] {
            *channel = linear_to_srgb(*channel);
        }
    }
    Ok(())
}

fn linear_to_srgb(value: u8) -> u8 {
    let normalized = f64::from(value) / 255.0;
    let encoded = if normalized <= 0.003_130_8 {
        normalized * 12.92
    } else {
        1.055 * normalized.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0 + 0.5) as u8
}

fn source_coordinate(index: u32, source: u32, target: u32) -> f64 {
    ((f64::from(index) + 0.5) * f64::from(source) / f64::from(target) - 0.5)
        .clamp(0.0, f64::from(source.saturating_sub(1)))
}

fn sample(pixels: &[u8], position: UiPixelPosition, channel: usize) -> f64 {
    let index = usize::try_from((position.y * position.width + position.x) * 4).unwrap() + channel;
    f64::from(pixels[index])
}

#[derive(Clone, Copy)]
pub(super) struct UiResampleSize {
    pub(super) source_width: u32,
    pub(super) source_height: u32,
    pub(super) target_width: u32,
    pub(super) target_height: u32,
}

#[derive(Clone, Copy)]
pub(super) struct UiLinearImagePlacement {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) left: i32,
    pub(super) top: i32,
}

#[derive(Clone, Copy)]
struct UiPixelPosition {
    width: u32,
    x: u32,
    y: u32,
}

fn interpolate(first: f64, second: f64, factor: f64) -> u8 {
    (first + (second - first) * factor)
        .round()
        .clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_rgba_becomes_linear_premultiplied_rgba() {
        assert_eq!(
            canonicalize_pixels(&[255, 0, 0, 128], UiColorPixelEncoding::StraightRgba).unwrap(),
            vec![128, 0, 0, 128]
        );
    }

    #[test]
    fn linearization_uses_the_srgb_transfer_curve_before_alpha() {
        assert!((srgb_channel_to_linear(128) - 0.2158541238).abs() < 1e-10);
        assert_eq!(
            canonicalize_pixels(&[128, 64, 32, 128], UiColorPixelEncoding::StraightRgba).unwrap(),
            vec![28, 7, 2, 128]
        );
    }

    #[test]
    fn premultiplied_bgra_is_unmixed_before_linearization() {
        let premultiplied =
            canonicalize_pixels(&[0, 0, 128, 128], UiColorPixelEncoding::PremultipliedBgra)
                .unwrap();
        let straight =
            canonicalize_pixels(&[128, 0, 0, 128], UiColorPixelEncoding::StraightRgba).unwrap();
        assert_eq!(premultiplied, vec![128, 0, 0, 128]);
        assert_ne!(premultiplied, straight);
    }

    #[test]
    fn bilinear_resampling_interpolates_premultiplied_channels_and_alpha() {
        let pixels = bilinear_resample(
            &[255, 0, 0, 255, 0, 0, 0, 0],
            UiResampleSize {
                source_width: 2,
                source_height: 1,
                target_width: 3,
                target_height: 1,
            },
        )
        .unwrap();
        assert_eq!(pixels, vec![255, 0, 0, 255, 128, 0, 0, 128, 0, 0, 0, 0]);
    }

    #[test]
    fn final_storage_encodes_linear_premultiplied_channels_for_srgb_sampling() {
        let mut pixels = [128, 55, 0, 128];
        encode_linear_premultiplied_for_srgb_texture(&mut pixels).unwrap();
        assert_eq!(pixels, [188, 128, 0, 128]);
    }
}
