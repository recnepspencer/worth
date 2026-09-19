use super::WorthUiApplicationSessionState;

impl WorthUiApplicationSessionState {
    pub(crate) fn appearance_mounted_state_invalidation_batch(
        &self,
        mounted: &crate::mounting::WorthUiMountedSessionState,
        axis: worth_ui_dsl::UiAppearanceStateAxis,
        instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) -> crate::runtime::appearance::UiAppearanceInvalidationBatch {
        crate::runtime::appearance::UiAppearanceInvalidationBatch::mounted_owner_state(
            self.app.prepared_authority().consumed_fact_index(),
            mounted,
            axis,
            instances,
        )
    }

    pub(crate) fn appearance_initial_invalidation_batch(
        &self,
    ) -> crate::runtime::appearance::UiAppearanceInvalidationBatch {
        crate::runtime::appearance::UiAppearanceInvalidationBatch::initial(
            self.app.prepared_authority().consumed_fact_index(),
        )
    }
}
