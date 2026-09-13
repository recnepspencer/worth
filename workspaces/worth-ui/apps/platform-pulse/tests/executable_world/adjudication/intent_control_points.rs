use std::fmt;

use super::visual_contract_manifest::{
    action_control, confirmation_control, portal_control, portal_control_for_extent,
    PlatformPulseNativeControlContract, PlatformPulseVisualContractFailure,
};
use crate::external_observation::{NativeClientPixelCapture, NativeClientPixelPoint};

const MINIMUM_INTERIOR_PIXELS: usize = 9;
const MINIMUM_LABEL_INK_PIXELS: usize = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulseActionControlPoint {
    point: NativeClientPixelPoint,
    region: NativeControlPixelRegion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulsePortalControlPoint {
    point: NativeClientPixelPoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulseConfirmationControlPoint {
    point: NativeClientPixelPoint,
    region: NativeControlPixelRegion,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NativeControlPixelRegion {
    minimum: [u32; 2],
    maximum: [u32; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VisibleControlPixelChange {
    differing_pixels: usize,
}

#[derive(Debug)]
pub(crate) enum IntentControlPointFailure {
    VisualContract(PlatformPulseVisualContractFailure),
    InsufficientInteriorPixels {
        control: &'static str,
        matching_pixels: usize,
        interior_pixels: usize,
    },
    LabelInkMissing {
        control: &'static str,
        differing_pixels: usize,
    },
    InvalidSelectedPixel,
    IndistinguishableControls,
    CaptureMismatch,
    VisibleChangeMissing {
        differing_pixels: usize,
        compared_pixels: usize,
    },
}

pub(crate) fn adjudicate_action_control_point(
    capture: &NativeClientPixelCapture,
) -> Result<PlatformPulseActionControlPoint, IntentControlPointFailure> {
    let contract = action_control().map_err(IntentControlPointFailure::VisualContract)?;
    let (point, region) = select_interior_pixel(capture, "action", contract)?;
    Ok(PlatformPulseActionControlPoint { point, region })
}

pub(crate) fn adjudicate_portal_control_point(
    capture: &NativeClientPixelCapture,
) -> Result<PlatformPulsePortalControlPoint, IntentControlPointFailure> {
    let contract = portal_control().map_err(IntentControlPointFailure::VisualContract)?;
    portal_control_point(capture, contract)
}

pub(crate) fn adjudicate_portal_control_point_for_extent(
    capture: &NativeClientPixelCapture,
    logical_client_extent: [u32; 2],
) -> Result<PlatformPulsePortalControlPoint, IntentControlPointFailure> {
    let contract = portal_control_for_extent(logical_client_extent)
        .map_err(IntentControlPointFailure::VisualContract)?;
    portal_control_point(capture, contract)
}

fn portal_control_point(
    capture: &NativeClientPixelCapture,
    contract: PlatformPulseNativeControlContract,
) -> Result<PlatformPulsePortalControlPoint, IntentControlPointFailure> {
    let (point, _) = select_interior_pixel(capture, "portal", contract)?;
    Ok(PlatformPulsePortalControlPoint { point })
}

pub(crate) fn adjudicate_confirmation_control_point(
    capture: &NativeClientPixelCapture,
) -> Result<PlatformPulseConfirmationControlPoint, IntentControlPointFailure> {
    let contract = confirmation_control().map_err(IntentControlPointFailure::VisualContract)?;
    let (point, region) = select_interior_pixel(capture, "confirmation", contract)?;
    Ok(PlatformPulseConfirmationControlPoint { point, region })
}

pub(crate) fn require_distinct_control_points(
    action: PlatformPulseActionControlPoint,
    confirmation: PlatformPulseConfirmationControlPoint,
) -> Result<(), IntentControlPointFailure> {
    if action.point == confirmation.point {
        return Err(IntentControlPointFailure::IndistinguishableControls);
    }
    Ok(())
}

pub(crate) fn adjudicate_visible_control_change(
    baseline: &NativeClientPixelCapture,
    current: &NativeClientPixelCapture,
    region: NativeControlPixelRegion,
) -> Result<VisibleControlPixelChange, IntentControlPointFailure> {
    if baseline.process_id() != current.process_id()
        || baseline.width() != current.width()
        || baseline.height() != current.height()
    {
        return Err(IntentControlPointFailure::CaptureMismatch);
    }
    let mut differing_pixels = 0;
    let mut compared_pixels = 0;
    for y in region.minimum[1]..=region.maximum[1] {
        for x in region.minimum[0]..=region.maximum[0] {
            let before = rgba_at(baseline, x, y);
            let after = rgba_at(current, x, y);
            if before.is_some() && after.is_some() {
                compared_pixels += 1;
                differing_pixels += usize::from(before != after);
            }
        }
    }
    if differing_pixels < MINIMUM_INTERIOR_PIXELS {
        return Err(IntentControlPointFailure::VisibleChangeMissing {
            differing_pixels,
            compared_pixels,
        });
    }
    Ok(VisibleControlPixelChange { differing_pixels })
}

fn select_interior_pixel(
    capture: &NativeClientPixelCapture,
    control: &'static str,
    contract: PlatformPulseNativeControlContract,
) -> Result<(NativeClientPixelPoint, NativeControlPixelRegion), IntentControlPointFailure> {
    let background = contract.background_rgba();
    let expected = [background[0], background[1], background[2]];
    let channel_tolerance = contract.channel_tolerance();
    let target = project_rect(
        contract.target_rect(),
        capture,
        contract.logical_client_extent(),
    )?;
    let label = project_rect(
        contract.label_rect(),
        capture,
        contract.logical_client_extent(),
    )?;
    let (matching_pixels, interior) =
        collect_interior_pixels(capture, target, expected, channel_tolerance);
    if interior.len() < MINIMUM_INTERIOR_PIXELS {
        return Err(IntentControlPointFailure::InsufficientInteriorPixels {
            control,
            matching_pixels,
            interior_pixels: interior.len(),
        });
    }
    let differing_pixels = count_differing_pixels(capture, label, expected, channel_tolerance);
    if differing_pixels < MINIMUM_LABEL_INK_PIXELS {
        return Err(IntentControlPointFailure::LabelInkMissing {
            control,
            differing_pixels,
        });
    }
    let preferred = [
        scale_axis(
            contract.preferred_logical_point()[0],
            capture.width(),
            contract.logical_client_extent()[0],
        ),
        scale_axis(
            contract.preferred_logical_point()[1],
            capture.height(),
            contract.logical_client_extent()[1],
        ),
    ];
    let (x, y) = interior
        .into_iter()
        .min_by_key(|(x, y)| x.abs_diff(preferred[0]) + y.abs_diff(preferred[1]))
        .ok_or(IntentControlPointFailure::InvalidSelectedPixel)?;
    let point = NativeClientPixelPoint::interior(capture, x, y, 1)
        .ok_or(IntentControlPointFailure::InvalidSelectedPixel)?;
    Ok((
        point,
        NativeControlPixelRegion {
            minimum: [target[0], target[1]],
            maximum: [target[2] - 1, target[3] - 1],
        },
    ))
}

fn collect_interior_pixels(
    capture: &NativeClientPixelCapture,
    target: [u32; 4],
    expected: [u8; 3],
    channel_tolerance: u8,
) -> (usize, Vec<(u32, u32)>) {
    let mut matching_pixels = 0;
    let mut interior = Vec::new();
    for y in target[1]..target[3] {
        for x in target[0]..target[2] {
            if !matches_at(capture, x, y, expected, channel_tolerance) {
                continue;
            }
            matching_pixels += 1;
            if x > target[0]
                && y > target[1]
                && x + 1 < target[2]
                && y + 1 < target[3]
                && neighborhood_matches(capture, x, y, expected, channel_tolerance)
            {
                interior.push((x, y));
            }
        }
    }
    (matching_pixels, interior)
}

fn project_rect(
    logical: [u32; 4],
    capture: &NativeClientPixelCapture,
    logical_extent: [u32; 2],
) -> Result<[u32; 4], IntentControlPointFailure> {
    let right = logical[0]
        .checked_add(logical[2])
        .ok_or(IntentControlPointFailure::InvalidSelectedPixel)?;
    let bottom = logical[1]
        .checked_add(logical[3])
        .ok_or(IntentControlPointFailure::InvalidSelectedPixel)?;
    let projected = [
        scale_axis(logical[0], capture.width(), logical_extent[0]),
        scale_axis(logical[1], capture.height(), logical_extent[1]),
        scale_axis(right, capture.width(), logical_extent[0]),
        scale_axis(bottom, capture.height(), logical_extent[1]),
    ];
    if projected[0] >= projected[2]
        || projected[1] >= projected[3]
        || projected[2] > capture.width()
        || projected[3] > capture.height()
    {
        return Err(IntentControlPointFailure::InvalidSelectedPixel);
    }
    Ok(projected)
}

fn count_differing_pixels(
    capture: &NativeClientPixelCapture,
    rect: [u32; 4],
    expected: [u8; 3],
    channel_tolerance: u8,
) -> usize {
    (rect[1]..rect[3])
        .flat_map(|y| (rect[0]..rect[2]).map(move |x| (x, y)))
        .filter(|&(x, y)| !matches_at(capture, x, y, expected, channel_tolerance))
        .count()
}

fn neighborhood_matches(
    capture: &NativeClientPixelCapture,
    x: u32,
    y: u32,
    expected: [u8; 3],
    channel_tolerance: u8,
) -> bool {
    ((y - 1)..=(y + 1)).all(|sample_y| {
        ((x - 1)..=(x + 1))
            .all(|sample_x| matches_at(capture, sample_x, sample_y, expected, channel_tolerance))
    })
}

fn matches_at(
    capture: &NativeClientPixelCapture,
    x: u32,
    y: u32,
    expected: [u8; 3],
    channel_tolerance: u8,
) -> bool {
    let Some(offset) = usize::try_from(y)
        .ok()
        .and_then(|y| y.checked_mul(capture.width() as usize))
        .and_then(|row| row.checked_add(x as usize))
        .and_then(|pixel| pixel.checked_mul(4))
    else {
        return false;
    };
    capture
        .rgba()
        .get(offset..offset + 3)
        .is_some_and(|observed| {
            observed
                .iter()
                .zip(expected)
                .all(|(&channel, expected)| channel.abs_diff(expected) <= channel_tolerance)
        })
}

fn scale_axis(logical: u32, physical: u32, logical_extent: u32) -> u32 {
    ((u64::from(logical) * u64::from(physical) + u64::from(logical_extent / 2))
        / u64::from(logical_extent)) as u32
}

fn rgba_at(capture: &NativeClientPixelCapture, x: u32, y: u32) -> Option<[u8; 4]> {
    let offset = usize::try_from(y)
        .ok()?
        .checked_mul(capture.width() as usize)?
        .checked_add(x as usize)?
        .checked_mul(4)?;
    let rgba = capture.rgba().get(offset..offset + 4)?;
    Some([rgba[0], rgba[1], rgba[2], rgba[3]])
}

impl PlatformPulseActionControlPoint {
    pub(crate) fn point(self) -> NativeClientPixelPoint {
        self.point
    }

    pub(crate) fn region(self) -> NativeControlPixelRegion {
        self.region
    }
}

impl PlatformPulsePortalControlPoint {
    pub(crate) fn point(self) -> NativeClientPixelPoint {
        self.point
    }
}

impl PlatformPulseConfirmationControlPoint {
    pub(crate) fn point(self) -> NativeClientPixelPoint {
        self.point
    }

    pub(crate) fn region(self) -> NativeControlPixelRegion {
        self.region
    }
}

impl VisibleControlPixelChange {
    pub(crate) fn differing_pixels(self) -> usize {
        self.differing_pixels
    }
}

impl fmt::Display for IntentControlPointFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::VisualContract(failure) => {
                write!(formatter, "independent visual contract: {failure:?}")
            }
            Self::InsufficientInteriorPixels {
                control,
                matching_pixels,
                interior_pixels,
            } => write!(
                formatter,
                "{control} control had {matching_pixels} matching pixels but only \
                 {interior_pixels} independently clickable interior pixels"
            ),
            Self::LabelInkMissing {
                control,
                differing_pixels,
            } => write!(
                formatter,
                "{control} control label painted only {differing_pixels} non-background pixels"
            ),
            Self::InvalidSelectedPixel => {
                formatter.write_str("selected control pixel was outside its source capture")
            }
            Self::IndistinguishableControls => {
                formatter.write_str("action and confirmation resolved to the same native pixel")
            }
            Self::CaptureMismatch => {
                formatter.write_str("visible posture captures came from different client images")
            }
            Self::VisibleChangeMissing {
                differing_pixels,
                compared_pixels,
            } => write!(
                formatter,
                "visible posture changed only {differing_pixels}/{compared_pixels} control pixels"
            ),
        }
    }
}
