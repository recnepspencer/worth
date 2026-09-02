use super::dependency_index::UiOverlayChangeSet;
use super::owner_input::UiOverlayCompositionOwnerInput;
use super::planner::{
    UiOverlayCommitDenial, UiOverlayCompositionDenial, UiOverlayCompositionState,
    UiPreparedOverlayComposition,
};
use super::snapshot::UiOverlayStackSnapshot;

pub(crate) struct UiOverlayCompositionCoordinator {
    state: UiOverlayCompositionState,
}

impl UiOverlayCompositionCoordinator {
    pub(super) fn new(state: UiOverlayCompositionState) -> Self {
        Self { state }
    }

    pub(super) fn prepare_initial(
        &self,
        owner_input: &UiOverlayCompositionOwnerInput<'_>,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        owner_input.with_composition_input(|input| self.state.prepare_initial(input))
    }

    pub(super) fn prepare_successor(
        &self,
        owner_input: &UiOverlayCompositionOwnerInput<'_>,
        changes: &UiOverlayChangeSet,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        owner_input.with_composition_input(|input| self.state.prepare_successor(input, changes))
    }

    pub(super) fn reconstruct(
        &self,
        owner_input: &UiOverlayCompositionOwnerInput<'_>,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        owner_input.with_composition_input(|input| self.state.reconstruct(input))
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
