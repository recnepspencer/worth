use super::WorthUiActiveApplicationSession;
use crate::facade::mounted::{
    UiMountedLayoutCompletionReceipt, UiMountedOccurrenceGeometryDenial,
    UiMountedSurfaceGeometryBatch,
};

#[path = "mounted_occurrence_geometry/index.rs"]
mod index;
#[path = "mounted_occurrence_geometry/mosaic_clip.rs"]
mod mosaic_clip;
#[path = "mounted_occurrence_geometry/scroll_clip.rs"]
mod scroll_clip;
#[path = "mounted_occurrence_geometry/seam_validation.rs"]
mod seam_validation;
#[path = "mounted_occurrence_geometry/validation_authority.rs"]
mod validation_authority;

pub(crate) use validation_authority::UiMountedOccurrenceGeometryValidationAuthority;

/// Borrowed authority for committing completed mounted layout.
pub struct WorthUiMountedLayout<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
}

impl WorthUiMountedLayout<'_> {
    pub fn basis(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Result<crate::mounting::UiMountedLayoutBasis, UiMountedOccurrenceGeometryDenial> {
        self.session
            .mounted
            .layout_basis(surface, self.session.generation_identity().clone())
    }

    pub fn complete_surface_geometry(
        &mut self,
        batch: UiMountedSurfaceGeometryBatch,
    ) -> Result<UiMountedLayoutCompletionReceipt, UiMountedOccurrenceGeometryDenial> {
        let mut staged_scroll = self.session.scroll.as_ref().cloned();
        let authority = UiMountedOccurrenceGeometryValidationAuthority::active(self.session);
        let validated = authority.validate(batch, staged_scroll.as_mut())?;
        let (batch, work) = validated.into_parts();
        let receipt = self
            .session
            .mounted
            .replace_occurrence_geometry(batch, staged_scroll.as_mut())
            .map(|receipt| {
                receipt
                    .with_region_plan_rows_visited(work.region_plan_rows_visited)
                    .with_occurrence_resolution_work(
                        work.occurrence_index_rows,
                        work.occurrence_ancestry_steps,
                    )
                    .add_region_resolution_work(work.region_index_rows, work.region_lookup_steps)
            })?;
        if let Some(staged) = staged_scroll {
            *self
                .session
                .scroll
                .as_mut()
                .expect("the staged Scroll owner came from an installed service") = staged;
        }
        Ok(receipt)
    }
}

impl WorthUiActiveApplicationSession {
    pub fn begin_mounted_layout(&mut self) -> WorthUiMountedLayout<'_> {
        WorthUiMountedLayout { session: self }
    }
}
