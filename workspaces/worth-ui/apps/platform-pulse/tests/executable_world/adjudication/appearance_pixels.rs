use std::fmt;

use crate::external_observation::NativeClientPixelCapture;

use super::dashboard_visual_oracle as oracle;

const WHITE: [u8; 3] = [255, 255, 255];
const CANVAS: [u8; 3] = [246, 244, 239];
const MINIMUM_BRAND_PIXELS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FirstFrameAppearanceFailure {
    MissingBrandForeground,
    MetricCardCorner {
        point: [u32; 2],
        observed: Option<[u8; 4]>,
    },
    MissingMetricCardFill {
        point: [u32; 2],
        observed: Option<[u8; 4]>,
    },
    IncorrectMetricCardGap {
        point: [u32; 2],
        observed: Option<[u8; 4]>,
    },
}

/// Independent checkpoints from the layout spec at 1536 × 1024: white brand
/// lettering on the navigation rail, a rounded metric card, and a canvas gap
/// between adjacent cards. The card row starts at the rail (236) plus the
/// 24-point gutter and below the masthead (57), gutter, greeting (66) and a
/// 20-point gap; its cards share the 1192 points left after three gaps by
/// their concept widths, so the first card spans x 260..569.3 and y 167..265
/// and the second starts at x 589.3. No product layout values are imported.
pub(crate) fn adjudicate_first_frame_appearance(
    pixels: &NativeClientPixelCapture,
) -> Result<(), FirstFrameAppearanceFailure> {
    let brand_pixels = (26..230)
        .flat_map(|x| (26..66).filter_map(move |y| logical_pixel(pixels, [x, y])))
        .filter(|pixel| matches_rgb(*pixel, WHITE))
        .count();
    if brand_pixels < MINIMUM_BRAND_PIXELS {
        return Err(FirstFrameAppearanceFailure::MissingBrandForeground);
    }

    for point in [[261, 169], [568, 169], [261, 263], [568, 263]] {
        let observed = logical_pixel(pixels, point);
        if observed.is_none_or(|pixel| !matches_rgb(pixel, CANVAS)) {
            return Err(FirstFrameAppearanceFailure::MetricCardCorner { point, observed });
        }
    }
    for point in [[269, 178], [551, 215], [601, 215]] {
        let observed = logical_pixel(pixels, point);
        if observed.is_none_or(|pixel| !matches_rgb(pixel, WHITE)) {
            return Err(FirstFrameAppearanceFailure::MissingMetricCardFill { point, observed });
        }
    }
    let point = [579, 215];
    let observed = logical_pixel(pixels, point);
    if observed.is_none_or(|pixel| !matches_rgb(pixel, CANVAS)) {
        return Err(FirstFrameAppearanceFailure::IncorrectMetricCardGap { point, observed });
    }
    Ok(())
}

fn logical_pixel(pixels: &NativeClientPixelCapture, point: [u32; 2]) -> Option<[u8; 4]> {
    let [width, height] = oracle::LOGICAL_EXTENT;
    let x = (u64::from(point[0]) * u64::from(pixels.width()) / u64::from(width)) as usize;
    let y = (u64::from(point[1]) * u64::from(pixels.height()) / u64::from(height)) as usize;
    let offset = y
        .checked_mul(pixels.width() as usize)?
        .checked_add(x)?
        .checked_mul(4)?;
    let pixel = pixels.rgba().get(offset..offset + 4)?;
    Some([pixel[0], pixel[1], pixel[2], pixel[3]])
}

fn matches_rgb(observed: [u8; 4], expected: [u8; 3]) -> bool {
    observed[..3]
        .iter()
        .zip(expected)
        .all(|(left, right)| left.abs_diff(right) <= oracle::CHANNEL_TOLERANCE)
}

impl fmt::Display for FirstFrameAppearanceFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "dashboard appearance: {self:?}")
    }
}
