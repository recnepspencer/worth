use super::dependency_index::UiOverlayChangeSet;
use super::owner_input::UiOverlayOwnerExportVector;
use super::planner::{
    UiOverlayCommitDenial, UiOverlayCompositionDenial, UiOverlayCompositionState,
    UiPreparedOverlayComposition,
};
use super::snapshot::UiOverlayStackSnapshot;

pub(super) struct UiOverlayCompositionCoordinator {
    state: UiOverlayCompositionState,
}

impl UiOverlayCompositionCoordinator {
    pub(super) fn new(state: UiOverlayCompositionState) -> Self {
        Self { state }
    }

    pub(super) fn prepare_initial(
        &self,
        exports: &UiOverlayOwnerExportVector,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        exports.with_composition_input(|input| self.state.prepare_initial(input))
    }

    pub(super) fn prepare_successor(
        &self,
        exports: &UiOverlayOwnerExportVector,
        changes: &UiOverlayChangeSet,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        exports.with_composition_input(|input| self.state.prepare_successor(input, changes))
    }

    pub(super) fn reconstruct(
        &self,
        exports: &UiOverlayOwnerExportVector,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        exports.with_composition_input(|input| self.state.reconstruct(input))
    }

    pub(super) fn retain_prepared(
        &mut self,
        prepared: UiPreparedOverlayComposition,
    ) -> Result<(), UiOverlayCommitDenial> {
        self.state.publish(prepared)
    }

    pub(super) fn current(&self) -> Option<&UiOverlayStackSnapshot> {
        self.state.current()
    }
}
