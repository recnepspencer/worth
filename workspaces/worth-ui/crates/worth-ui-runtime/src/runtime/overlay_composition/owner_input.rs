use super::binding_export::UiOverlayPortalBindingExport;
use super::extent::UiOverlaySurfaceExtentSnapshot;
use super::motion_export::UiOverlayMotionOwnerExport;
use super::planner::UiOverlayCompositionInput;
use super::portal_export::UiOverlayPortalOwnerExport;
use super::snapshot::UiOverlayApplicationGeneration;
use crate::runtime::allocation_receipt::UiMountedOverlayExtentOwnerExport;
use crate::runtime::motion::UiMotionOverlayOwnerExport;
use crate::runtime::portal::UiPortalOverlayBindingOwnerExport;

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
    ExtentGenerationMismatch,
    PresentationGenerationMismatch,
    BindingExport,
    BindingGenerationMismatch,
    BindingSurfaceMismatch,
    BindingPortalRevisionMismatch,
    MotionGenerationMismatch,
}

impl UiOverlayOwnerExportVector {
    pub(super) fn from_authoritative(
        generation: crate::facade::prepared_application_authority::
            WorthUiPreparedApplicationGenerationIdentity,
        portal: UiOverlayPortalOwnerExport,
        extent: UiMountedOverlayExtentOwnerExport,
        presentation: &crate::runtime::presentation_state::UiApplicationPresentationOwnerExport,
        bindings: &UiPortalOverlayBindingOwnerExport,
        motion: Option<&UiMotionOverlayOwnerExport>,
    ) -> Result<Self, UiOverlayOwnerExportDenial> {
        let prepared_generation = generation.clone();
        if extent.generation() != &prepared_generation {
            return Err(UiOverlayOwnerExportDenial::ExtentGenerationMismatch);
        }
        if presentation.generation() != &prepared_generation {
            return Err(UiOverlayOwnerExportDenial::PresentationGenerationMismatch);
        }
        let generation = UiOverlayApplicationGeneration::from_prepared(generation);
        let extent = UiOverlaySurfaceExtentSnapshot::from_owner_export(extent)
            .map_err(|_| UiOverlayOwnerExportDenial::InvalidExtentRevision)?;
        let motion = motion
            .map(|source| {
                UiOverlayMotionOwnerExport::from_owner(
                    prepared_generation.clone(),
                    source,
                    bindings,
                )
            })
            .transpose()
            .map_err(|_| UiOverlayOwnerExportDenial::BindingExport)?
            .filter(|motion| !motion.is_empty());
        let bindings = UiOverlayPortalBindingExport::from_owner(prepared_generation, bindings)
            .map_err(|_| UiOverlayOwnerExportDenial::BindingExport)?;
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
            presentation: presentation.presentation(),
            bindings,
            motion,
        })
    }

    #[cfg(test)]
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
