use super::WorthUiApplicationSessionState;

impl WorthUiApplicationSessionState {
    pub(crate) fn appearance_initial_invalidation_batch(
        &self,
    ) -> crate::runtime::appearance::UiAppearanceInvalidationBatch {
        crate::runtime::appearance::UiAppearanceInvalidationBatch::initial(
            self.app.prepared_authority().consumed_fact_index(),
        )
    }

    pub(crate) fn appearance_role_replacement_batch(
        &self,
        role: &worth_ui_dsl::UiAppearanceRoleIdentity,
    ) -> crate::runtime::appearance::UiAppearanceInvalidationBatch {
        crate::runtime::appearance::UiAppearanceInvalidationBatch::role_replacement(
            self.app.prepared_authority().consumed_fact_index(),
            role,
        )
    }

    pub(crate) fn appearance_state_invalidation_batch(
        &self,
        axis: worth_ui_dsl::UiAppearanceStateAxis,
    ) -> crate::runtime::appearance::UiAppearanceInvalidationBatch {
        crate::runtime::appearance::UiAppearanceInvalidationBatch::owner_state(
            self.app.prepared_authority().consumed_fact_index(),
            axis,
        )
    }

    pub(crate) fn appearance_theme_invalidation_batch(
        &self,
        capability_identity: &str,
        authored_identity: &str,
    ) -> Result<
        crate::runtime::appearance::UiAppearanceInvalidationBatch,
        crate::graph::UiGraphFactLookupDenial,
    > {
        crate::runtime::appearance::UiAppearanceInvalidationBatch::theme_slot(
            self.app.prepared_authority().consumed_fact_index(),
            capability_identity,
            authored_identity,
        )
    }

    pub(crate) fn appearance_theme_invalidation_batch_for_slot(
        &self,
        slot: &crate::capability::ThemeTokenId,
    ) -> Result<
        crate::runtime::appearance::UiAppearanceInvalidationBatch,
        crate::graph::UiGraphFactLookupDenial,
    > {
        let prepared = self.app.prepared_authority();
        let declarations = prepared.authored_declaration_lookup();
        let authored_identity = declarations
            .theme_token_declaration_identity(slot.as_str())
            .unwrap_or(slot.as_str());
        crate::runtime::appearance::UiAppearanceInvalidationBatch::theme_slot(
            prepared.consumed_fact_index(),
            slot.as_str(),
            authored_identity,
        )
    }
}
