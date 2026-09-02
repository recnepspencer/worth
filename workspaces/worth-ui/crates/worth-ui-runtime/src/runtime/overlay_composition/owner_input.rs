use super::binding_export::UiOverlayPortalBindingExport;
use super::extent::UiOverlaySurfaceExtentSnapshot;
use super::motion_export::UiOverlayMotionOwnerExport;
use super::planner::UiOverlayCompositionInput;
use super::portal_export::UiOverlayPortalOwnerExport;
use super::snapshot::UiOverlayApplicationGeneration;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiOverlayOwnerExportVector {
    generation: UiOverlayApplicationGeneration,
    portal_snapshot: crate::runtime::portal::UiPortalStackSnapshot,
    extent: UiOverlaySurfaceExtentSnapshot,
    presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    bindings: UiOverlayPortalBindingExport,
    motion: Option<UiOverlayMotionOwnerExport>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiOverlayOwnerExportDenial {
    InvalidExtentRevision,
    BindingGenerationMismatch,
    BindingSurfaceMismatch,
    BindingPortalRevisionMismatch,
    MotionGenerationMismatch,
}

impl UiOverlayOwnerExportVector {
    pub(super) fn from_prepared(
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        portal: UiOverlayPortalOwnerExport,
        extent: UiOverlaySurfaceExtentSnapshot,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
        bindings: UiOverlayPortalBindingExport,
        motion: Option<UiOverlayMotionOwnerExport>,
    ) -> Result<Self, UiOverlayOwnerExportDenial> {
        let generation = UiOverlayApplicationGeneration::from_prepared(generation);
        let portal_snapshot = portal.into_snapshot();
        if extent.revision() == 0 {
            return Err(UiOverlayOwnerExportDenial::InvalidExtentRevision);
        }
        if bindings.generation() != &generation {
            return Err(UiOverlayOwnerExportDenial::BindingGenerationMismatch);
        }
        if bindings.runtime_surface() != extent.runtime_surface() {
            return Err(UiOverlayOwnerExportDenial::BindingSurfaceMismatch);
        }
        if bindings.portal_revision() != portal_snapshot.owner_revision() {
            return Err(UiOverlayOwnerExportDenial::BindingPortalRevisionMismatch);
        }
        if motion
            .as_ref()
            .is_some_and(|motion| motion.generation() != &generation)
        {
            return Err(UiOverlayOwnerExportDenial::MotionGenerationMismatch);
        }
        Ok(Self {
            generation,
            portal_snapshot,
            extent,
            presentation,
            bindings,
            motion,
        })
    }

    pub(super) fn with_composition_input<T>(
        &self,
        compose: impl FnOnce(UiOverlayCompositionInput<'_>) -> T,
    ) -> T {
        let motion = self
            .motion
            .as_ref()
            .map(UiOverlayMotionOwnerExport::snapshot);
        compose(UiOverlayCompositionInput {
            generation: self.generation.clone(),
            presentation: self.presentation,
            extent: &self.extent,
            portal_snapshot: &self.portal_snapshot,
            portal_bindings: self.bindings.rows(),
            motion,
        })
    }
}
