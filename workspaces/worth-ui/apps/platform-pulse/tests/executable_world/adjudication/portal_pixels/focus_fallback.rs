use crate::external_observation::NativeClientPixelCapture;

use super::{
    checked_in, project_region, require_same_capture, rgba_at, PlatformPulsePortalPixelFailure,
    MINIMUM_SEMANTIC_INK_PIXELS,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulsePortalFocusFallbackPixelEvidence {
    removed_primary_pixels: usize,
    fallback_action_pixels: usize,
}

pub(crate) fn adjudicate_focus_fallback_portal_pixels(
    before: &NativeClientPixelCapture,
    after: &NativeClientPixelCapture,
    logical_client_extent: [u32; 2],
) -> Result<PlatformPulsePortalFocusFallbackPixelEvidence, PlatformPulsePortalPixelFailure> {
    let manifest = checked_in().map_err(PlatformPulsePortalPixelFailure::Manifest)?;
    require_same_capture(before, after)?;
    let physical = [after.width(), after.height()];
    let primary = project_region(
        manifest.portal_primary_region(),
        logical_client_extent,
        physical,
    );
    let mut changed = 0;
    let mut background_matching = 0;
    let mut sampled = 0;
    for y in primary[1]..primary[3] {
        for x in primary[0]..primary[2] {
            if let (Some(prior), Some(current)) = (rgba_at(before, x, y), rgba_at(after, x, y)) {
                sampled += 1;
                changed += usize::from(prior != current);
                background_matching += usize::from(
                    current[..3]
                        .iter()
                        .zip(manifest.portal_overlay_rgba()[..3].iter())
                        .all(|(&observed, &expected)| {
                            observed.abs_diff(expected) <= manifest.channel_tolerance()
                        }),
                );
            }
        }
    }
    if sampled == 0
        || changed < MINIMUM_SEMANTIC_INK_PIXELS
        || background_matching * 4 < sampled * 3
    {
        return Err(
            PlatformPulsePortalPixelFailure::PreferredFocusParticipantRetained {
                changed,
                background_matching,
                sampled,
            },
        );
    }
    let cancel = project_region(
        manifest.portal_cancel_region(),
        logical_client_extent,
        physical,
    );
    let mut differing = 0;
    let mut fallback_action_pixels = 0;
    for y in cancel[1]..cancel[3] {
        for x in cancel[0]..cancel[2] {
            if let (Some(prior), Some(current)) = (rgba_at(before, x, y), rgba_at(after, x, y)) {
                fallback_action_pixels += 1;
                differing += usize::from(prior != current);
            }
        }
    }
    if fallback_action_pixels == 0 || differing != 0 {
        return Err(PlatformPulsePortalPixelFailure::FallbackActionChanged {
            differing,
            sampled: fallback_action_pixels,
        });
    }
    let portal = project_region(
        manifest.portal_overlay_region(),
        logical_client_extent,
        physical,
    );
    let mut surviving = 0;
    let mut changed_surviving = 0;
    for y in portal[1]..portal[3] {
        for x in portal[0]..portal[2] {
            if x >= primary[0] && x < primary[2] && y >= primary[1] && y < primary[3] {
                continue;
            }
            if let (Some(prior), Some(current)) = (rgba_at(before, x, y), rgba_at(after, x, y)) {
                surviving += 1;
                changed_surviving += usize::from(prior != current);
            }
        }
    }
    if surviving == 0 || changed_surviving != 0 {
        return Err(PlatformPulsePortalPixelFailure::SurvivingPortalChanged {
            differing: changed_surviving,
            sampled: surviving,
        });
    }
    Ok(PlatformPulsePortalFocusFallbackPixelEvidence {
        removed_primary_pixels: changed,
        fallback_action_pixels,
    })
}

impl PlatformPulsePortalFocusFallbackPixelEvidence {
    pub(crate) const fn removed_primary_pixels(self) -> usize {
        self.removed_primary_pixels
    }

    pub(crate) const fn fallback_action_pixels(self) -> usize {
        self.fallback_action_pixels
    }
}
