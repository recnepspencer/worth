use super::{
    checked_in,
    model::{LayoutContract, PlatformPulseVisualContractManifest},
    PlatformPulseVisualContractFailure,
};

pub(in crate::adjudication) struct PlatformPulseVisualAdjudicationContract {
    manifest: PlatformPulseVisualContractManifest,
}

pub(in crate::adjudication) fn checked_in_adjudication_contract(
) -> Result<PlatformPulseVisualAdjudicationContract, PlatformPulseVisualContractFailure> {
    checked_in().map(|manifest| PlatformPulseVisualAdjudicationContract { manifest })
}

impl PlatformPulseVisualAdjudicationContract {
    pub(in crate::adjudication) fn logical_client_extent(&self) -> [u32; 2] {
        self.default_layout().logical_client_extent
    }

    pub(in crate::adjudication) fn background_logical_point(&self) -> [u32; 2] {
        self.control_point("canvas")
    }

    pub(in crate::adjudication) fn target_logical_point(&self) -> [u32; 2] {
        self.control_point("live-action")
    }

    pub(in crate::adjudication) fn target_region(&self) -> [u32; 4] {
        self.target_rect("platform.pulse.target.run_live_action")
    }

    pub(in crate::adjudication) fn portal_overlay_region(&self) -> [u32; 4] {
        self.pixel_region("portal-overlay")
    }

    pub(in crate::adjudication) fn portal_accent_region(&self) -> [u32; 4] {
        self.pixel_region("portal-accent")
    }

    pub(in crate::adjudication) fn portal_title_region(&self) -> [u32; 4] {
        self.pixel_region("portal-title")
    }

    pub(in crate::adjudication) fn portal_body_region(&self) -> [u32; 4] {
        self.pixel_region("portal-body")
    }

    pub(in crate::adjudication) fn portal_cancel_region(&self) -> [u32; 4] {
        self.pixel_region("portal-cancel")
    }

    pub(in crate::adjudication) fn portal_cancel_label_region(&self) -> [u32; 4] {
        self.pixel_region("portal-cancel-label")
    }

    pub(in crate::adjudication) fn portal_primary_region(&self) -> [u32; 4] {
        self.pixel_region("portal-primary")
    }

    pub(in crate::adjudication) fn portal_primary_label_region(&self) -> [u32; 4] {
        self.pixel_region("portal-primary-label")
    }

    pub(in crate::adjudication) fn canvas_rgba(&self) -> [u8; 4] {
        self.token("canvas")
    }

    pub(in crate::adjudication) fn target_rgba(&self) -> [u8; 4] {
        self.token("action-fill")
    }

    pub(in crate::adjudication) fn overlay_rgba(&self) -> [u8; 4] {
        self.token("visual-inspection-overlay")
    }

    pub(in crate::adjudication) fn portal_overlay_rgba(&self) -> [u8; 4] {
        self.token("elevated-surface")
    }

    pub(in crate::adjudication) fn raised_surface_rgba(&self) -> [u8; 4] {
        self.token("raised-surface")
    }

    pub(in crate::adjudication) fn principal_accent_rgba(&self) -> [u8; 4] {
        self.token("principal-accent")
    }

    pub(in crate::adjudication) fn primary_text_rgba(&self) -> [u8; 4] {
        self.token("primary-text")
    }

    pub(in crate::adjudication) fn secondary_text_rgba(&self) -> [u8; 4] {
        self.token("secondary-text")
    }

    pub(in crate::adjudication) fn action_text_rgba(&self) -> [u8; 4] {
        self.token("action-text")
    }

    pub(in crate::adjudication) fn channel_tolerance(&self) -> u8 {
        self.manifest.limits.channel_tolerance
    }

    pub(in crate::adjudication) fn maximum_capture_scale(&self) -> u32 {
        self.manifest.limits.maximum_capture_scale
    }

    pub(in crate::adjudication) fn maximum_pixel_bytes(&self) -> u64 {
        self.manifest.limits.maximum_capture_rgba_bytes
    }

    pub(in crate::adjudication) fn visible_region_count(&self) -> u64 {
        self.manifest.inspection.visible_region_count
    }

    pub(in crate::adjudication) fn hit_test_region_count(&self) -> u64 {
        self.manifest.inspection.hit_test_region_count
    }

    pub(in crate::adjudication) fn target_authored_name(&self) -> &str {
        &self.manifest.inspection.target_authored_name
    }

    fn default_layout(&self) -> &LayoutContract {
        self.manifest
            .layouts
            .iter()
            .find(|layout| layout.name == "default")
            .expect("validated default layout")
    }

    fn control_point(&self, identity: &str) -> [u32; 2] {
        self.default_layout()
            .control_points
            .iter()
            .find(|point| point.identity == identity)
            .expect("validated adjudication control point")
            .logical_point
    }

    fn target_rect(&self, identity: &str) -> [u32; 4] {
        self.default_layout()
            .minimum_targets
            .iter()
            .find(|target| target.identity == identity)
            .expect("validated adjudication target")
            .rect
    }

    fn pixel_region(&self, identity: &str) -> [u32; 4] {
        self.default_layout()
            .pixel_regions
            .iter()
            .find(|region| region.identity == identity)
            .expect("validated adjudication pixel region")
            .rect
    }

    fn token(&self, role: &str) -> [u8; 4] {
        self.manifest
            .tokens
            .iter()
            .find(|token| token.role == role)
            .expect("validated adjudication token")
            .rgba
    }
}
