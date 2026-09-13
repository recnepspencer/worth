use super::runtime_service_story_pixels::is_intentionally_mutable_story_pixel;
use super::visual_contract_manifest::checked_in_adjudication_contract as checked_in;
use crate::external_observation::{NativeClientPixelCapture, NativeClientPixelPoint};

mod authored_surface;
mod failure;
mod focus_fallback;
pub(crate) use authored_surface::{
    adjudicate_authored_portal_pixels, PlatformPulseAuthoredPortalPixelEvidence,
};
pub(crate) use failure::PlatformPulsePortalPixelFailure;
pub(crate) use focus_fallback::{
    adjudicate_focus_fallback_portal_pixels, PlatformPulsePortalFocusFallbackPixelEvidence,
};

const MINIMUM_OVERLAY_MATCH_RATIO: usize = 4;
const MINIMUM_OVERLAY_CHANGE_RATIO: usize = 2;
const MINIMUM_SEMANTIC_INK_PIXELS: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulsePortalPixelEvidence {
    changed_pixels: usize,
    overlay_matching_pixels: usize,
    sampled_pixels: usize,
    authored_surface_matching_pixels: usize,
    semantic_ink_pixels: usize,
}
pub(crate) fn adjudicate_open_portal_pixels(
    baseline: &NativeClientPixelCapture,
    opened: &NativeClientPixelCapture,
) -> Result<PlatformPulsePortalPixelEvidence, PlatformPulsePortalPixelFailure> {
    let manifest = checked_in().map_err(PlatformPulsePortalPixelFailure::Manifest)?;
    require_same_capture(baseline, opened)?;
    let region = project_region(
        manifest.portal_overlay_region(),
        manifest.logical_client_extent(),
        [opened.width(), opened.height()],
    );
    let expected = manifest.portal_overlay_rgba();
    let tolerance = manifest.channel_tolerance();
    let mut changed = 0;
    let mut matching = 0;
    let mut sampled = 0;
    for y in region[1]..region[3] {
        for x in region[0]..region[2] {
            let before = rgba_at(baseline, x, y);
            let after = rgba_at(opened, x, y);
            if let (Some(before), Some(after)) = (before, after) {
                sampled += 1;
                changed += usize::from(before != after);
                matching += usize::from(
                    after[..3]
                        .iter()
                        .zip(expected[..3].iter())
                        .all(|(&observed, &expected)| observed.abs_diff(expected) <= tolerance),
                );
            }
        }
    }
    if sampled == 0
        || changed * MINIMUM_OVERLAY_CHANGE_RATIO < sampled
        || matching * MINIMUM_OVERLAY_MATCH_RATIO < sampled * 3
    {
        return Err(PlatformPulsePortalPixelFailure::OverlayMissing {
            changed,
            matching,
            sampled,
        });
    }
    let authored_surface_matching_pixels = [
        require_surface(
            opened,
            manifest.portal_accent_region(),
            manifest.logical_client_extent(),
            manifest.principal_accent_rgba(),
            tolerance,
            "accent",
        )?,
        require_surface(
            opened,
            manifest.portal_cancel_region(),
            manifest.logical_client_extent(),
            manifest.raised_surface_rgba(),
            tolerance,
            "Cancel action",
        )?,
        require_surface(
            opened,
            manifest.portal_primary_region(),
            manifest.logical_client_extent(),
            manifest.target_rgba(),
            tolerance,
            "primary action",
        )?,
    ]
    .into_iter()
    .sum();
    let semantic_ink_pixels = [
        require_ink(
            opened,
            manifest.portal_title_region(),
            manifest.logical_client_extent(),
            manifest.primary_text_rgba(),
            tolerance,
            "title",
        )?,
        require_ink(
            opened,
            manifest.portal_body_region(),
            manifest.logical_client_extent(),
            manifest.secondary_text_rgba(),
            tolerance,
            "body",
        )?,
        require_ink(
            opened,
            manifest.portal_cancel_label_region(),
            manifest.logical_client_extent(),
            manifest.secondary_text_rgba(),
            tolerance,
            "Cancel label",
        )?,
        require_ink(
            opened,
            manifest.portal_primary_label_region(),
            manifest.logical_client_extent(),
            manifest.action_text_rgba(),
            tolerance,
            "primary label",
        )?,
    ]
    .into_iter()
    .sum();
    Ok(PlatformPulsePortalPixelEvidence {
        changed_pixels: changed,
        overlay_matching_pixels: matching,
        sampled_pixels: sampled,
        authored_surface_matching_pixels,
        semantic_ink_pixels,
    })
}

pub(crate) fn portal_action_points(
    capture: &NativeClientPixelCapture,
) -> Result<[NativeClientPixelPoint; 2], PlatformPulsePortalPixelFailure> {
    let manifest = checked_in().map_err(PlatformPulsePortalPixelFailure::Manifest)?;
    let physical = [capture.width(), capture.height()];
    let primary = project_region(
        manifest.portal_primary_region(),
        manifest.logical_client_extent(),
        physical,
    );
    let cancel = project_region(
        manifest.portal_cancel_region(),
        manifest.logical_client_extent(),
        physical,
    );
    Ok([
        interior_center(capture, primary)?,
        interior_center(capture, cancel)?,
    ])
}

pub(crate) fn portal_occupancy_point(
    capture: &NativeClientPixelCapture,
) -> Result<NativeClientPixelPoint, PlatformPulsePortalPixelFailure> {
    let manifest = checked_in().map_err(PlatformPulsePortalPixelFailure::Manifest)?;
    let body = project_region(
        manifest.portal_body_region(),
        manifest.logical_client_extent(),
        [capture.width(), capture.height()],
    );
    let point = [
        body[0] + (body[2] - body[0]) * 3 / 4,
        body[1] + (body[3] - body[1]) / 2,
    ];
    NativeClientPixelPoint::interior(capture, point[0], point[1], 2)
        .ok_or(PlatformPulsePortalPixelFailure::CaptureMismatch)
}

fn require_surface(
    capture: &NativeClientPixelCapture,
    logical: [u32; 4],
    authored: [u32; 2],
    expected: [u8; 4],
    tolerance: u8,
    identity: &'static str,
) -> Result<usize, PlatformPulsePortalPixelFailure> {
    let region = project_region(logical, authored, [capture.width(), capture.height()]);
    let (matching, sampled) = matching_pixels(capture, region, expected, tolerance);
    if sampled == 0 || matching * 4 < sampled * 3 {
        return Err(PlatformPulsePortalPixelFailure::AuthoredSurfaceMissing {
            identity,
            matching,
            sampled,
        });
    }
    Ok(matching)
}

fn require_ink(
    capture: &NativeClientPixelCapture,
    logical: [u32; 4],
    authored: [u32; 2],
    expected: [u8; 4],
    tolerance: u8,
    identity: &'static str,
) -> Result<usize, PlatformPulsePortalPixelFailure> {
    let region = project_region(logical, authored, [capture.width(), capture.height()]);
    let (matching, _) = matching_pixels(capture, region, expected, tolerance.saturating_mul(2));
    if matching < MINIMUM_SEMANTIC_INK_PIXELS {
        return Err(PlatformPulsePortalPixelFailure::SemanticInkMissing { identity, matching });
    }
    Ok(matching)
}

fn interior_center(
    capture: &NativeClientPixelCapture,
    region: [u32; 4],
) -> Result<NativeClientPixelPoint, PlatformPulsePortalPixelFailure> {
    let x = region[0] + (region[2] - region[0]) / 2;
    let y = region[1] + (region[3] - region[1]) / 2;
    NativeClientPixelPoint::interior(capture, x, y, 2)
        .ok_or(PlatformPulsePortalPixelFailure::CaptureMismatch)
}

fn matching_pixels(
    capture: &NativeClientPixelCapture,
    region: [u32; 4],
    expected: [u8; 4],
    tolerance: u8,
) -> (usize, usize) {
    let mut matching = 0;
    let mut sampled = 0;
    for y in region[1]..region[3] {
        for x in region[0]..region[2] {
            if let Some(observed) = rgba_at(capture, x, y) {
                sampled += 1;
                matching += usize::from(
                    observed[..3]
                        .iter()
                        .zip(expected[..3].iter())
                        .all(|(&observed, &expected)| observed.abs_diff(expected) <= tolerance),
                );
            }
        }
    }
    (matching, sampled)
}

pub(crate) fn adjudicate_closed_portal_pixels(
    baseline: &NativeClientPixelCapture,
    closed: &NativeClientPixelCapture,
) -> Result<(), PlatformPulsePortalPixelFailure> {
    let manifest = checked_in().map_err(PlatformPulsePortalPixelFailure::Manifest)?;
    require_same_capture(baseline, closed)?;
    let region = project_region(
        manifest.portal_overlay_region(),
        manifest.logical_client_extent(),
        [closed.width(), closed.height()],
    );
    let mut differing: usize = 0;
    let mut sampled: usize = 0;
    for y in region[1]..region[3] {
        for x in region[0]..region[2] {
            if is_intentionally_mutable_story_pixel(closed, x, y) {
                continue;
            }
            if let (Some(before), Some(after)) = (rgba_at(baseline, x, y), rgba_at(closed, x, y)) {
                sampled += 1;
                if before != after {
                    differing += 1;
                }
            }
        }
    }
    if sampled == 0 || differing != 0 {
        return Err(PlatformPulsePortalPixelFailure::RestorationMissing { differing, sampled });
    }
    Ok(())
}

fn require_same_capture(
    left: &NativeClientPixelCapture,
    right: &NativeClientPixelCapture,
) -> Result<(), PlatformPulsePortalPixelFailure> {
    if left.process_id() != right.process_id()
        || left.width() != right.width()
        || left.height() != right.height()
    {
        Err(PlatformPulsePortalPixelFailure::CaptureMismatch)
    } else {
        Ok(())
    }
}

fn project_region(logical: [u32; 4], authored: [u32; 2], physical: [u32; 2]) -> [u32; 4] {
    [
        scale(logical[0], authored[0], physical[0]),
        scale(logical[1], authored[1], physical[1]),
        scale(logical[0] + logical[2], authored[0], physical[0]),
        scale(logical[1] + logical[3], authored[1], physical[1]),
    ]
}

fn scale(value: u32, authored: u32, physical: u32) -> u32 {
    ((u64::from(value) * u64::from(physical)) / u64::from(authored)) as u32
}

fn rgba_at(capture: &NativeClientPixelCapture, x: u32, y: u32) -> Option<[u8; 4]> {
    let offset = (y as usize)
        .checked_mul(capture.width() as usize)?
        .checked_add(x as usize)?
        .checked_mul(4)?;
    let rgba = capture.rgba().get(offset..offset + 4)?;
    Some([rgba[0], rgba[1], rgba[2], rgba[3]])
}

impl PlatformPulsePortalPixelEvidence {
    pub(crate) const fn changed_pixels(self) -> usize {
        self.changed_pixels
    }

    pub(crate) const fn overlay_matching_pixels(self) -> usize {
        self.overlay_matching_pixels
    }

    pub(crate) const fn sampled_pixels(self) -> usize {
        self.sampled_pixels
    }

    pub(crate) const fn authored_surface_matching_pixels(self) -> usize {
        self.authored_surface_matching_pixels
    }

    pub(crate) const fn semantic_ink_pixels(self) -> usize {
        self.semantic_ink_pixels
    }
}
