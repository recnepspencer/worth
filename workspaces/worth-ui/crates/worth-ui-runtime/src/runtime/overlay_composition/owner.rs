use worth_ui_dsl::UiBackdropDeclaration;

use super::coordinator::UiOverlayCompositionCoordinator;
use super::dependency_index::UiOverlayChangeSet;
use super::owner_input::UiOverlayOwnerExportVector;
use super::planner::{
    UiOverlayCapacityProfile, UiOverlayCommitDenial, UiOverlayCompositionDenial,
    UiOverlayCompositionState, UiPreparedOverlayComposition,
};
use super::snapshot::UiOverlayStackSnapshot;

pub(crate) struct UiOverlayCompositionOwner {
    coordinator: UiOverlayCompositionCoordinator,
}

impl UiOverlayCompositionOwner {
    pub(super) fn admit(
        declarations: impl IntoIterator<Item = UiBackdropDeclaration>,
        declaration_revision: u64,
    ) -> Result<Self, UiOverlayCompositionDenial> {
        let state = UiOverlayCompositionState::admit(
            declarations,
            declaration_revision,
            UiOverlayCapacityProfile::qualified(),
        )?;
        Ok(Self {
            coordinator: UiOverlayCompositionCoordinator::new(state),
        })
    }

    pub(super) fn prepare_initial(
        &self,
        exports: &UiOverlayOwnerExportVector,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        self.coordinator.prepare_initial(exports)
    }

    pub(super) fn prepare_successor(
        &self,
        exports: &UiOverlayOwnerExportVector,
        changes: &UiOverlayChangeSet,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        self.coordinator.prepare_successor(exports, changes)
    }

    pub(super) fn reconstruct(
        &self,
        exports: &UiOverlayOwnerExportVector,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        self.coordinator.reconstruct(exports)
    }

    pub(super) fn retain_prepared(
        &mut self,
        prepared: UiPreparedOverlayComposition,
    ) -> Result<(), UiOverlayCommitDenial> {
        self.coordinator.retain_prepared(prepared)
    }

    pub(super) fn current(&self) -> Option<&UiOverlayStackSnapshot> {
        self.coordinator.current()
    }
}
