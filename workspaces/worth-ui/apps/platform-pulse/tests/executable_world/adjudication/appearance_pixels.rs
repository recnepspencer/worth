use std::fmt;

use crate::external_observation::NativeClientPixelCapture;

use super::platform_pulse_control_points::{checked_in, PlatformPulseControlPointManifestFailure};

const MINIMUM_ACCENT_GLYPH_PIXELS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FirstFrameAppearanceFailure {
    Manifest(PlatformPulseControlPointManifestFailure),
    MissingAccentForeground,
    MissingCardInterior,
    SquareCardCorner {
        corner: usize,
        observed: Option<[u8; 4]>,
    },
    MissingCurvedCardBorder,
}

pub(crate) fn adjudicate_first_frame_appearance(
    pixels: &NativeClientPixelCapture,
) -> Result<(), FirstFrameAppearanceFailure> {
    let manifest = checked_in().map_err(FirstFrameAppearanceFailure::Manifest)?;
    let accent_pixels = pixels_in_logical_region(pixels, manifest.brand_region())
        .filter(|pixel| {
            matches_rgb(
                *pixel,
                manifest.principal_accent_rgba(),
                manifest.channel_tolerance(),
            )
        })
        .count();
    if accent_pixels < MINIMUM_ACCENT_GLYPH_PIXELS {
        return Err(FirstFrameAppearanceFailure::MissingAccentForeground);
    }

    let card = manifest.query_card_region();
    let radius = manifest.query_card_radius();
    for (index, corner) in rounded_corner_oracle(card, radius).into_iter().enumerate() {
        let outside = logical_pixel(pixels, corner.outside);
        if outside.is_none_or(|pixel| {
            closest_to(
                pixel,
                manifest.raised_surface_rgba(),
                [manifest.canvas_rgba(), manifest.structural_rule_rgba()],
            )
        }) {
            return Err(FirstFrameAppearanceFailure::SquareCardCorner {
                corner: index,
                observed: outside,
            });
        }
        let interior = logical_pixel(pixels, corner.interior)
            .ok_or(FirstFrameAppearanceFailure::MissingCardInterior)?;
        if !closest_to(
            interior,
            manifest.raised_surface_rgba(),
            [manifest.canvas_rgba(), manifest.structural_rule_rgba()],
        ) {
            return Err(FirstFrameAppearanceFailure::MissingCardInterior);
        }
        let arc_region = [
            corner.arc[0] - 2,
            corner.arc[1] - 2,
            corner.arc[0] + 3,
            corner.arc[1] + 3,
        ];
        if !pixels_in_logical_region(pixels, arc_region).any(|pixel| {
            matches_rgb(
                pixel,
                manifest.structural_rule_rgba(),
                manifest.channel_tolerance(),
            )
        }) {
            return Err(FirstFrameAppearanceFailure::MissingCurvedCardBorder);
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct RoundedCornerOracle {
    outside: [u32; 2],
    arc: [u32; 2],
    interior: [u32; 2],
}

fn rounded_corner_oracle(card: [u32; 4], radius: u32) -> [RoundedCornerOracle; 4] {
    let right = card[2] - 1;
    let bottom = card[3] - 1;
    // For a 24-unit radius, seven units on each axis is the independently
    // rounded 45-degree circle intercept: r - r/sqrt(2).
    let diagonal = ((f64::from(radius) * (1.0 - std::f64::consts::FRAC_1_SQRT_2)).round()) as u32;
    let inside = diagonal + 3;
    [
        RoundedCornerOracle {
            outside: [card[0], card[1]],
            arc: [card[0] + diagonal, card[1] + diagonal],
            interior: [card[0] + inside, card[1] + inside],
        },
        RoundedCornerOracle {
            outside: [right, card[1]],
            arc: [right - diagonal, card[1] + diagonal],
            interior: [right - inside, card[1] + inside],
        },
        RoundedCornerOracle {
            outside: [card[0], bottom],
            arc: [card[0] + diagonal, bottom - diagonal],
            interior: [card[0] + inside, bottom - inside],
        },
        RoundedCornerOracle {
            outside: [right, bottom],
            arc: [right - diagonal, bottom - diagonal],
            interior: [right - inside, bottom - inside],
        },
    ]
}

fn pixels_in_logical_region(
    pixels: &NativeClientPixelCapture,
    region: [u32; 4],
) -> impl Iterator<Item = [u8; 4]> + '_ {
    let extent = [960, 600];
    let start = scale_point(pixels, [region[0], region[1]], extent);
    let end = scale_point(pixels, [region[2], region[3]], extent);
    (start[1]..end[1])
        .flat_map(move |y| (start[0]..end[0]).filter_map(move |x| physical_pixel(pixels, [x, y])))
}

fn logical_pixel(pixels: &NativeClientPixelCapture, point: [u32; 2]) -> Option<[u8; 4]> {
    physical_pixel(pixels, scale_point(pixels, point, [960, 600]))
}

fn scale_point(
    pixels: &NativeClientPixelCapture,
    point: [u32; 2],
    logical_extent: [u32; 2],
) -> [u32; 2] {
    [
        (u64::from(point[0]) * u64::from(pixels.width()) / u64::from(logical_extent[0])) as u32,
        (u64::from(point[1]) * u64::from(pixels.height()) / u64::from(logical_extent[1])) as u32,
    ]
}

fn physical_pixel(pixels: &NativeClientPixelCapture, point: [u32; 2]) -> Option<[u8; 4]> {
    let offset = usize::try_from(point[1])
        .ok()?
        .checked_mul(usize::try_from(pixels.width()).ok()?)?
        .checked_add(usize::try_from(point[0]).ok()?)?
        .checked_mul(4)?;
    let pixel = pixels.rgba().get(offset..offset + 4)?;
    Some([pixel[0], pixel[1], pixel[2], pixel[3]])
}

fn matches_rgb(observed: [u8; 4], expected: [u8; 4], tolerance: u8) -> bool {
    observed[..3]
        .iter()
        .zip(expected[..3].iter())
        .all(|(left, right)| left.abs_diff(*right) <= tolerance)
}

fn closest_to(observed: [u8; 4], expected: [u8; 4], alternatives: [[u8; 4]; 2]) -> bool {
    let expected_distance = rgb_distance(observed, expected);
    alternatives
        .into_iter()
        .all(|alternative| expected_distance < rgb_distance(observed, alternative))
}

fn rgb_distance(left: [u8; 4], right: [u8; 4]) -> u16 {
    left[..3]
        .iter()
        .zip(right[..3].iter())
        .map(|(left, right)| u16::from(left.abs_diff(*right)))
        .sum()
}

impl fmt::Display for FirstFrameAppearanceFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Manifest(failure) => write!(formatter, "control-point manifest: {failure:?}"),
            Self::MissingAccentForeground => {
                formatter.write_str("accent foreground is absent from the Brand glyph region")
            }
            Self::MissingCardInterior => {
                formatter.write_str("raised appearance fill is absent inside QueryCard")
            }
            Self::SquareCardCorner { corner, observed } => write!(
                formatter,
                "QueryCard corner {corner} is filled as a square instead of respecting its radius: {observed:?}"
            ),
            Self::MissingCurvedCardBorder => formatter
                .write_str("QueryCard has no visible structural border along its rounded corner"),
        }
    }
}
