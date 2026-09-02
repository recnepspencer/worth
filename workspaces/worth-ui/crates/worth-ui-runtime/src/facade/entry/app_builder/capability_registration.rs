use crate::facade::registry::descriptor::{
    CommandDescriptor, CommandProjectionDescriptor, ComponentDescriptor, IconDescriptor,
    MosaicPlacementPolicyDescriptor, MosaicRegionKindDescriptor, MosaicSizingContractDescriptor,
    MosaicStateSlotDescriptor, NativeCapabilityDescriptor, PluginSlotDescriptor,
    RuntimeOutcomeProjectionDescriptor, SettingDescriptor, SurfaceDescriptor,
    TaskPresentationDescriptor, ThemeTokenDescriptor,
};

use super::WorthUiApplicationBuilder;

impl<ChangeProfileState, IntentWiringState>
    WorthUiApplicationBuilder<ChangeProfileState, IntentWiringState>
{
    pub fn register_command(mut self, descriptor: CommandDescriptor) -> Self {
        self.inner = self.inner.register_command(descriptor);
        self
    }

    pub fn register_command_projection(mut self, descriptor: CommandProjectionDescriptor) -> Self {
        self.inner = self.inner.register_command_projection(descriptor);
        self
    }

    pub fn register_component(mut self, descriptor: ComponentDescriptor) -> Self {
        self.inner = self.inner.register_component(descriptor);
        self
    }

    pub fn register_appearance_role(
        mut self,
        role: worth_ui_dsl::UiAppearanceRoleDeclaration,
    ) -> Result<Self, crate::capability::AppearanceRoleRegistrationDenial> {
        self.inner = self.inner.register_appearance_role(role)?;
        Ok(self)
    }

    pub fn register_icon(mut self, descriptor: IconDescriptor) -> Self {
        self.inner = self.inner.register_icon(descriptor);
        self
    }

    pub fn register_surface(mut self, descriptor: SurfaceDescriptor) -> Self {
        self.inner = self.inner.register_surface(descriptor);
        self
    }

    pub fn register_mosaic_region_kind(mut self, descriptor: MosaicRegionKindDescriptor) -> Self {
        self.inner = self.inner.register_mosaic_region_kind(descriptor);
        self
    }

    pub fn register_mosaic_placement_policy(
        mut self,
        descriptor: MosaicPlacementPolicyDescriptor,
    ) -> Self {
        self.inner = self.inner.register_mosaic_placement_policy(descriptor);
        self
    }

    pub fn register_mosaic_sizing_contract(
        mut self,
        descriptor: MosaicSizingContractDescriptor,
    ) -> Self {
        self.inner = self.inner.register_mosaic_sizing_contract(descriptor);
        self
    }

    pub fn register_mosaic_state_slot(mut self, descriptor: MosaicStateSlotDescriptor) -> Self {
        self.inner = self.inner.register_mosaic_state_slot(descriptor);
        self
    }

    pub fn register_native_capability(mut self, descriptor: NativeCapabilityDescriptor) -> Self {
        self.inner = self.inner.register_native_capability(descriptor);
        self
    }

    pub fn register_plugin_slot(mut self, descriptor: PluginSlotDescriptor) -> Self {
        self.inner = self.inner.register_plugin_slot(descriptor);
        self
    }

    #[cfg(test)]
    pub(crate) fn register_view_binding(
        mut self,
        descriptor: crate::facade::registry::descriptor::ViewBindingDescriptor,
    ) -> Self {
        self.inner = self.inner.register_view_binding(descriptor);
        self
    }

    pub fn register_runtime_outcome_projection(
        mut self,
        descriptor: RuntimeOutcomeProjectionDescriptor,
    ) -> Self {
        self.inner = self.inner.register_runtime_outcome_projection(descriptor);
        self
    }

    pub fn register_setting(mut self, descriptor: SettingDescriptor) -> Self {
        self.inner = self.inner.register_setting(descriptor);
        self
    }

    pub fn register_task_presentation(mut self, descriptor: TaskPresentationDescriptor) -> Self {
        self.inner = self.inner.register_task_presentation(descriptor);
        self
    }

    pub fn register_theme_token(mut self, descriptor: ThemeTokenDescriptor) -> Self {
        self.inner = self.inner.register_theme_token(descriptor);
        self
    }
}
