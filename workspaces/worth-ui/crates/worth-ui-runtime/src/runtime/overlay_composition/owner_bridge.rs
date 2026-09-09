use worth_ui_dsl::UiBackdropDeclaration;

use super::dependency_index::UiOverlayChangeSet;
use super::owner::UiOverlayCompositionOwner;
use super::owner_input::{UiOverlayOwnerExportDenial, UiOverlayOwnerExportVector};
use super::planner::{
    UiOverlayCommitDenial, UiOverlayCompositionDenial, UiPreparedOverlayComposition,
};
use super::portal_export::UiOverlayPortalOwnerExport;
use crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity;
use crate::runtime::allocation_receipt::UiMountedOverlayExtentOwner;
use crate::runtime::motion::UiMotionRuntimeState;
use crate::runtime::portal::{
    UiPortalOverlayBindingDenial, UiPortalOverlayBindingOwner, UiPortalRuntimeState,
    UiPortalStackSnapshot,
};
use crate::runtime::presentation_state::UiApplicationPresentationOwnerExport;

pub(crate) struct UiOverlayOwnerSources<'a> {
    pub(super) generation: &'a WorthUiPreparedApplicationGenerationIdentity,
    pub(super) portal: UiPortalStackSnapshot,
    pub(super) extent: &'a UiMountedOverlayExtentOwner,
    pub(super) presentation: &'a UiApplicationPresentationOwnerExport,
    pub(super) bindings: &'a UiPortalOverlayBindingOwner,
    pub(super) motion: Option<crate::runtime::motion::UiMotionOverlayOwnerExport>,
}

impl<'a> UiOverlayOwnerSources<'a> {
    pub(crate) fn new(
        generation: &'a WorthUiPreparedApplicationGenerationIdentity,
        portal: Option<&'a UiPortalRuntimeState>,
        extent: &'a UiMountedOverlayExtentOwner,
        presentation: &'a UiApplicationPresentationOwnerExport,
        bindings: &'a UiPortalOverlayBindingOwner,
        motion: Option<&'a UiMotionRuntimeState>,
    ) -> Self {
        Self {
            generation,
            portal: portal
                .map(UiPortalRuntimeState::stack_snapshot)
                .unwrap_or_else(UiPortalStackSnapshot::empty),
            extent,
            presentation,
            bindings,
            motion: motion.map(UiMotionRuntimeState::overlay_owner_export),
        }
    }

    pub(crate) const fn from_exports(
        generation: &'a WorthUiPreparedApplicationGenerationIdentity,
        portal: UiPortalStackSnapshot,
        extent: &'a UiMountedOverlayExtentOwner,
        presentation: &'a UiApplicationPresentationOwnerExport,
        bindings: &'a UiPortalOverlayBindingOwner,
        motion: Option<crate::runtime::motion::UiMotionOverlayOwnerExport>,
    ) -> Self {
        Self {
            generation,
            portal,
            extent,
            presentation,
            bindings,
            motion,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum UiOverlayOwnerBridgeDenial {
    PortalBindings(UiPortalOverlayBindingDenial),
    OwnerExports(UiOverlayOwnerExportDenial),
    Composition(UiOverlayCompositionDenial),
    Commit(UiOverlayCommitDenial),
}

pub(crate) struct UiOverlayCompositionOwnerBridge {
    owner: UiOverlayCompositionOwner,
}

impl UiOverlayCompositionOwnerBridge {
    pub(super) fn admit(
        declarations: impl IntoIterator<Item = UiBackdropDeclaration>,
        declaration_revision: u64,
    ) -> Result<Self, UiOverlayOwnerBridgeDenial> {
        Ok(Self {
            owner: UiOverlayCompositionOwner::admit(declarations, declaration_revision)
                .map_err(UiOverlayOwnerBridgeDenial::Composition)?,
        })
    }

    pub(super) fn prepare_initial(
        &self,
        sources: UiOverlayOwnerSources<'_>,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayOwnerBridgeDenial> {
        let exports = assemble_owner_exports(sources)?;
        self.owner
            .prepare_initial(&exports)
            .map_err(UiOverlayOwnerBridgeDenial::Composition)
    }

    pub(super) fn prepare_successor(
        &self,
        sources: UiOverlayOwnerSources<'_>,
        changes: &UiOverlayChangeSet,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayOwnerBridgeDenial> {
        let exports = assemble_owner_exports(sources)?;
        self.owner
            .prepare_successor(&exports, changes)
            .map_err(UiOverlayOwnerBridgeDenial::Composition)
    }

    pub(super) fn reconstruct(
        &self,
        sources: UiOverlayOwnerSources<'_>,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayOwnerBridgeDenial> {
        let exports = assemble_owner_exports(sources)?;
        self.owner
            .reconstruct(&exports)
            .map_err(UiOverlayOwnerBridgeDenial::Composition)
    }

    pub(super) fn retain_prepared(
        &mut self,
        prepared: UiPreparedOverlayComposition,
    ) -> Result<(), UiOverlayOwnerBridgeDenial> {
        self.owner
            .retain_prepared(prepared)
            .map_err(UiOverlayOwnerBridgeDenial::Commit)
    }

    pub(super) fn current(&self) -> Option<&super::snapshot::UiOverlayStackSnapshot> {
        self.owner.current()
    }
}

fn assemble_owner_exports(
    sources: UiOverlayOwnerSources<'_>,
) -> Result<UiOverlayOwnerExportVector, UiOverlayOwnerBridgeDenial> {
    let bindings = sources
        .bindings
        .export(&sources.portal)
        .map_err(UiOverlayOwnerBridgeDenial::PortalBindings)?;
    UiOverlayOwnerExportVector::from_authoritative(
        sources.generation.clone(),
        UiOverlayPortalOwnerExport::from_snapshot(sources.portal),
        sources.extent.export(),
        sources.presentation,
        &bindings,
        sources.motion.as_ref(),
    )
    .map_err(UiOverlayOwnerBridgeDenial::OwnerExports)
}
