use super::WorthUiApplicationSessionState;

impl WorthUiApplicationSessionState {
    pub(crate) fn appearance_state_consumers(
        &self,
        axis: worth_ui_dsl::UiAppearanceStateAxis,
    ) -> crate::runtime::appearance::UiAppearanceConsumerSelection {
        let index = self.app.prepared_authority().consumed_fact_index();
        crate::runtime::appearance::UiAppearanceConsumerSelection::for_state(index, axis)
    }

    pub(crate) fn appearance_role_consumers(
        &self,
        role: &worth_ui_dsl::UiAppearanceRoleIdentity,
    ) -> crate::runtime::appearance::UiAppearanceConsumerSelection {
        let index = self.app.prepared_authority().consumed_fact_index();
        crate::runtime::appearance::UiAppearanceConsumerSelection::for_role(index, role)
    }

    pub(crate) fn appearance_slot_consumers(
        &self,
        slot: &worth_ui_dsl::UiThemeSlotIdentity,
    ) -> crate::runtime::appearance::UiAppearanceConsumerSelection {
        let prepared = self.app.prepared_authority();
        let declarations = prepared.authored_declaration_lookup();
        let authored_identity = declarations
            .theme_token_declaration_identity(slot.as_str())
            .unwrap_or(slot.as_str());
        let index = prepared.consumed_fact_index();
        crate::runtime::appearance::UiAppearanceConsumerSelection::for_slot(
            index,
            slot.as_str(),
            authored_identity,
        )
    }
}
