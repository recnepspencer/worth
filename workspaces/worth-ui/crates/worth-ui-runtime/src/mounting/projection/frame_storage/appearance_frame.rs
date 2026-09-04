use super::{appearance_state, UiMountedProjectionFrame};

impl UiMountedProjectionFrame {
    pub(crate) fn inherit_appearance_state(&mut self, predecessor: Option<&Self>) {
        self.appearance_state
            .inherit_from(predecessor.map(|frame| &frame.appearance_state));
    }

    pub(crate) fn set_appearance_invalidation_batch(
        &mut self,
        batch: crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) {
        self.appearance_state.set_batch(batch);
    }

    pub(crate) fn appearance_invalidation_batch(
        &self,
    ) -> Option<crate::runtime::appearance::UiAppearanceInvalidationBatch> {
        self.appearance_state.batch().cloned()
    }

    pub(crate) fn begin_appearance_reconstruction(&mut self) {
        self.appearance_state.begin_reconstruction();
    }

    pub(crate) fn prune_appearance_state(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) {
        let nodes = self.appearance_node_inputs();
        self.appearance_state
            .prune_to_current_nodes(session, generation, &nodes);
    }

    pub(crate) fn reserve_appearance_state(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> Result<(), appearance_state::UiAppearanceStateCapacityExceeded> {
        self.appearance_state.reserve(context)
    }

    pub(crate) fn stage_appearance_projection(
        &mut self,
        attempt: crate::runtime::appearance::UiAppearanceProjectionAttempt,
    ) -> Result<(), appearance_state::UiAppearanceStateCapacityExceeded> {
        self.appearance_state.stage(attempt)
    }

    pub(crate) const fn appearance_state_capacity_error(
        &self,
    ) -> Option<appearance_state::UiAppearanceStateCapacityExceeded> {
        self.appearance_state.capacity_error()
    }

    pub(crate) fn lower_appearance(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        self.appearance_state.lower(presentation)
    }
}
