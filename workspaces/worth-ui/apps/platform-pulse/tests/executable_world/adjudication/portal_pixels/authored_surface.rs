use super::{
    checked_in, matching_pixels, project_region, require_ink, require_surface,
    PlatformPulsePortalPixelFailure, MINIMUM_OVERLAY_MATCH_RATIO,
};
use crate::external_observation::NativeClientPixelCapture;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlatformPulseAuthoredPortalPixelEvidence {
    overlay_matching_pixels: usize,
    sampled_pixels: usize,
    authored_surface_matching_pixels: usize,
    semantic_ink_pixels: usize,
}

pub(crate) fn adjudicate_authored_portal_pixels(
    opened: &NativeClientPixelCapture,
    logical_client_extent: [u32; 2],
) -> Result<PlatformPulseAuthoredPortalPixelEvidence, PlatformPulsePortalPixelFailure> {
    let manifest = checked_in().map_err(PlatformPulsePortalPixelFailure::Manifest)?;
    let region = project_region(
        manifest.portal_overlay_region(),
        logical_client_extent,
        [opened.width(), opened.height()],
    );
    let (matching, sampled) = matching_pixels(
        opened,
        region,
        manifest.portal_overlay_rgba(),
        manifest.channel_tolerance(),
    );
    if sampled == 0 || matching * MINIMUM_OVERLAY_MATCH_RATIO < sampled * 3 {
        return Err(PlatformPulsePortalPixelFailure::OverlayMissing {
            changed: sampled,
            matching,
            sampled,
        });
    }
    let authored_surface_matching_pixels = [
        require_surface(
            opened,
            manifest.portal_accent_region(),
            logical_client_extent,
            manifest.principal_accent_rgba(),
            manifest.channel_tolerance(),
            "accent",
        )?,
        require_surface(
            opened,
            manifest.portal_cancel_region(),
            logical_client_extent,
            manifest.raised_surface_rgba(),
            manifest.channel_tolerance(),
            "Cancel action",
        )?,
        require_surface(
            opened,
            manifest.portal_primary_region(),
            logical_client_extent,
            manifest.target_rgba(),
            manifest.channel_tolerance(),
            "primary action",
        )?,
    ]
    .into_iter()
    .sum();
    let tolerance = manifest.channel_tolerance();
    let semantic_ink_pixels = [
        require_ink(
            opened,
            manifest.portal_title_region(),
            logical_client_extent,
            manifest.primary_text_rgba(),
            tolerance,
            "title",
        )?,
        require_ink(
            opened,
            manifest.portal_body_region(),
            logical_client_extent,
            manifest.secondary_text_rgba(),
            tolerance,
            "body",
        )?,
        require_ink(
            opened,
            manifest.portal_cancel_label_region(),
            logical_client_extent,
            manifest.secondary_text_rgba(),
            tolerance,
            "Cancel label",
        )?,
        require_ink(
            opened,
            manifest.portal_primary_label_region(),
            logical_client_extent,
            manifest.action_text_rgba(),
            tolerance,
            "primary label",
        )?,
    ]
    .into_iter()
    .sum();
    Ok(PlatformPulseAuthoredPortalPixelEvidence {
        overlay_matching_pixels: matching,
        sampled_pixels: sampled,
        authored_surface_matching_pixels,
        semantic_ink_pixels,
    })
}

impl PlatformPulseAuthoredPortalPixelEvidence {
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
