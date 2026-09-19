use super::CapabilitySnapshot;
use crate::capability::{
    AppearanceRoleRegistrationDenial, CapabilitySnapshotBuilder, CapabilitySnapshotFreezeInput,
};

impl CapabilitySnapshot {
    pub(crate) fn refreeze_authored_appearance_roles<'a>(
        &self,
        declarations: impl IntoIterator<Item = &'a worth_ui_dsl::UiAppearanceRoleDeclaration>,
    ) -> Result<Option<Self>, AppearanceRoleRegistrationDenial> {
        let Some(appearance_roles) = self
            .appearance_roles
            .prepare_authored_succession(declarations)?
        else {
            return Ok(None);
        };
        Ok(Some(
            CapabilitySnapshotBuilder::new(CapabilitySnapshotFreezeInput {
                registered_capabilities: self.registered_capabilities.clone(),
                appearance_roles,
                appearance_themes: self.appearance_themes.clone(),
                commands: self.commands.clone(),
                command_projections: self.command_projections.clone(),
                components: self.components.clone(),
                icons: self.icons.clone(),
                intent_definitions: self.intent_definitions.clone(),
                surfaces: self.surfaces.clone(),
                mosaic_regions: self.mosaic_regions.clone(),
                mosaic_placement_policies: self.mosaic_placement_policies.clone(),
                mosaic_sizing_contracts: self.mosaic_sizing_contracts.clone(),
                mosaic_state_slots: self.mosaic_state_slots.clone(),
                native_capabilities: self.native_capabilities.clone(),
                plugin_slots: self.plugin_slots.clone(),
                view_bindings: self.view_bindings.clone(),
                runtime_outcome_projections: self.runtime_outcome_projections.clone(),
                settings: self.settings.clone(),
                task_presentations: self.task_presentations.clone(),
                theme_tokens: self.theme_tokens.clone(),
                support_catalog: self.support_catalog.clone(),
            })
            .freeze(),
        ))
    }
}
