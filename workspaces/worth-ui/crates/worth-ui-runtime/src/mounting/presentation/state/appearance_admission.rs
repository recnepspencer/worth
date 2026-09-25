use super::{
    UiMountedPresentationAdmission, UiMountedPresentationAdmissionRejection,
    UiMountedPresentationAttempt,
};
use crate::runtime::appearance::UiAppearanceInspectionAttemptBatch;

/// Owns the actual lowering result, including accepted Motion and overlay composition.
pub(crate) struct UiMountedAppearanceAdmission {
    admission: UiMountedPresentationAdmission,
    inspection: UiAppearanceInspectionAttemptBatch,
    refresh_visual_regions: bool,
}

impl UiMountedPresentationAdmission {
    pub(crate) fn lower_appearance_with_overlays(
        mut self,
        profile: Option<&worth_ui_host_contract::UiHostAppearanceProfileContract>,
        derived: &crate::mounting::UiMountedAppearanceDerivedInput,
    ) -> UiMountedAppearanceAdmission {
        let overlays = derived.overlays.as_slice();
        let requires_complete = self.candidates.requires_complete_appearance_projection();
        if requires_complete && self.frame.prepare_appearance_reconstruction().is_err() {
            return self.deny_appearance_output();
        }
        let Ok(targets) = self.frame.appearance_motion_targets(overlays) else {
            return self.deny_appearance_output();
        };
        let motion = self.candidates.accepted_appearance_motion(&targets);
        // Chrome is derived where each region is laid out; this frame paints
        // it, and samples its Motion, where the frame presents the region.
        let Ok(scroll_chrome) = self
            .frame
            .semantic_projection()
            .place_scroll_chrome(&derived.scroll_chrome)
        else {
            return self.deny_appearance_output();
        };
        let refresh_visual_regions = !targets.is_empty()
            || !overlays.is_empty()
            || !derived.scroll_chrome.is_empty()
            || !self.frame.retired_appearance_instances().is_empty();
        self.frame
            .record_accepted_motion_commands_visited(motion.commands_visited());
        let inspection = self.frame.lower_appearance_with_motion_and_overlays(
            self.attempt,
            profile,
            motion,
            overlays,
            &scroll_chrome,
        );
        self.candidates
            .bind_appearance_sample_targets(&self.frame, &targets);
        if self
            .candidates
            .bind_scroll_motion_groups(&self.frame, derived, &scroll_chrome)
            .is_err()
        {
            return self.deny_appearance_output();
        }
        UiMountedAppearanceAdmission {
            admission: self,
            inspection,
            refresh_visual_regions,
        }
    }

    pub(crate) fn deny_appearance_output(mut self) -> UiMountedAppearanceAdmission {
        let inspection = self.frame.deny_appearance_output();
        UiMountedAppearanceAdmission {
            admission: self,
            inspection,
            refresh_visual_regions: false,
        }
    }
}

impl UiMountedAppearanceAdmission {
    pub(crate) fn admit_appearance_retention(
        self,
    ) -> Result<
        (
            UiMountedPresentationAttempt,
            UiAppearanceInspectionAttemptBatch,
        ),
        Box<(
            UiMountedPresentationAdmissionRejection,
            UiAppearanceInspectionAttemptBatch,
        )>,
    > {
        let Self {
            mut admission,
            inspection,
            refresh_visual_regions,
        } = self;
        if !admission.frame.appearance_output_available() {
            return Err(Box::new((admission.reject_appearance_output(), inspection)));
        }
        let indexed_motion_bytes = admission.candidates.indexed_motion_reserved_bytes();
        // Unchanged visual regions are refreshed from what retention already
        // holds, so a reservation alone never rebuilds them from the frame.
        let visual_regions = if refresh_visual_regions {
            Some(admission.frame.visual_region_basis())
        } else if indexed_motion_bytes != Some(0) {
            Some(
                admission
                    .retention
                    .retained_visual_regions()
                    .unwrap_or_else(|| admission.frame.visual_region_basis()),
            )
        } else {
            None
        };
        if let Some(visual_regions) = visual_regions {
            match admission.retention.refresh_visual_regions(
                visual_regions.with_indexed_motion_reservation(indexed_motion_bytes),
                admission.frame.manifest().surfaces(),
            ) {
                Ok(work) => admission.frame.record_scroll_hit_succession(work),
                Err(denial) => {
                    return Err(Box::new((
                        admission.reject_retained_visual_basis(denial),
                        inspection,
                    )))
                }
            }
        }
        Ok((UiMountedPresentationAttempt { admission }, inspection))
    }
}
