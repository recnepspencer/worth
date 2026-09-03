use super::WorthUiApplicationSessionState;

impl WorthUiApplicationSessionState {
    #[allow(
        dead_code,
        reason = "Gate 1 retains the state-axis appearance projection for later mounting publication"
    )]
    pub(crate) fn appearance_state_consumers(
        &self,
        axis: worth_ui_dsl::UiAppearanceStateAxis,
    ) -> crate::runtime::appearance::UiAppearanceConsumerSelection {
        let index = self.app.prepared_authority().consumed_fact_index();
        crate::runtime::appearance::UiAppearanceConsumerSelection::for_state(index, axis)
    }

    #[allow(
        dead_code,
        reason = "Gate 1 retains the role appearance projection for later mounting publication"
    )]
    pub(crate) fn appearance_role_consumers(
        &self,
        role: &worth_ui_dsl::UiAppearanceRoleIdentity,
    ) -> crate::runtime::appearance::UiAppearanceConsumerSelection {
        let index = self.app.prepared_authority().consumed_fact_index();
        crate::runtime::appearance::UiAppearanceConsumerSelection::for_role(index, role)
    }

    pub(crate) fn appearance_slot_consumers(
        &self,
        slot: &crate::capability::ThemeTokenId,
    ) -> Result<
        crate::runtime::appearance::UiAppearanceConsumerSelection,
        crate::graph::UiGraphFactLookupDenial,
    > {
        let prepared = self.app.prepared_authority();
        let declarations = prepared.authored_declaration_lookup();
        let authored_identity = declarations
            .theme_token_declaration_identity(slot.as_str())
            .unwrap_or(slot.as_str());
        let index = prepared.consumed_fact_index();
        crate::runtime::appearance::UiAppearanceConsumerSelection::try_for_slot(
            index,
            slot.as_str(),
            authored_identity,
        )
    }
}
