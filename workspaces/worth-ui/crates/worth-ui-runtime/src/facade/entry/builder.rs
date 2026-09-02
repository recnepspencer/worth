use crate::capability::{
    AppearanceRoleRegistry, CapabilityDiagnosticRichness, CommandDescriptor,
    CommandProjectionDescriptor, CommandProjectionRegistry, CommandRegistry, ComponentDescriptor,
    ComponentRegistry, IconDescriptor, IconRegistry, IntentDefinitionRegistry,
    MosaicPlacementPolicyDescriptor, MosaicPlacementRegistry, MosaicRegionKindDescriptor,
    MosaicRegionRegistry, MosaicSizingContractDescriptor, MosaicSizingRegistry,
    MosaicStateSlotDescriptor, MosaicStateSlotRegistry, NativeCapabilityDescriptor,
    NativeCapabilityRegistry, PluginSlotDescriptor, PluginSlotRegistry, RegistrationCandidate,
    RuntimeOutcomeProjectionDescriptor, RuntimeOutcomeProjectionRegistry, SettingDescriptor,
    SettingsRegistry, SurfaceDescriptor, SurfaceRegistry, TaskPresentationDescriptor,
    TaskPresentationRegistry, ThemeRegistry, ThemeTokenDescriptor, ThemeTokenRegistry, UiIntent,
    UiIntentDefinition, UiIntentDefinitionDestination, UiIntentDefinitionRegistrationError,
    ViewBindingDescriptor, ViewBindingRegistry,
};

mod snapshot_freeze;

/// Low-level capability registration builder for the public Worth UI facade.
pub struct CapabilityRegistrationBuilder {
    registration_candidates: Vec<RegistrationCandidate>,
    appearance_role_registry: AppearanceRoleRegistry,
    appearance_theme_registry: ThemeRegistry,
    command_registry: CommandRegistry,
    command_projection_registry: CommandProjectionRegistry,
    component_registry: ComponentRegistry,
    icon_registry: IconRegistry,
    intent_definition_registry: IntentDefinitionRegistry,
    surface_registry: SurfaceRegistry,
    mosaic_region_registry: MosaicRegionRegistry,
    mosaic_placement_registry: MosaicPlacementRegistry,
    mosaic_sizing_registry: MosaicSizingRegistry,
    mosaic_state_slot_registry: MosaicStateSlotRegistry,
    native_capability_registry: NativeCapabilityRegistry,
    plugin_slot_registry: PluginSlotRegistry,
    view_binding_registry: ViewBindingRegistry,
    runtime_outcome_projection_registry: RuntimeOutcomeProjectionRegistry,
    settings_registry: SettingsRegistry,
    task_presentation_registry: TaskPresentationRegistry,
    theme_token_registry: ThemeTokenRegistry,
    diagnostic_richness: CapabilityDiagnosticRichness,
}

impl Default for CapabilityRegistrationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilityRegistrationBuilder {
    pub fn new() -> Self {
        Self {
            registration_candidates: Vec::new(),
            appearance_role_registry: AppearanceRoleRegistry::empty(),
            appearance_theme_registry: ThemeRegistry::default(),
            command_registry: CommandRegistry::empty(),
            command_projection_registry: CommandProjectionRegistry::empty(),
            component_registry: ComponentRegistry::empty(),
            icon_registry: IconRegistry::empty(),
            intent_definition_registry: IntentDefinitionRegistry::empty(),
            surface_registry: SurfaceRegistry::empty(),
            mosaic_region_registry: MosaicRegionRegistry::empty(),
            mosaic_placement_registry: MosaicPlacementRegistry::empty(),
            mosaic_sizing_registry: MosaicSizingRegistry::empty(),
            mosaic_state_slot_registry: MosaicStateSlotRegistry::empty(),
            native_capability_registry: NativeCapabilityRegistry::empty(),
            plugin_slot_registry: PluginSlotRegistry::empty(),
            view_binding_registry: ViewBindingRegistry::empty(),
            runtime_outcome_projection_registry: RuntimeOutcomeProjectionRegistry::empty(),
            settings_registry: SettingsRegistry::empty(),
            task_presentation_registry: TaskPresentationRegistry::empty(),
            theme_token_registry: ThemeTokenRegistry::empty(),
            diagnostic_richness: CapabilityDiagnosticRichness::Rich,
        }
    }

    /// Register a domain-agnostic application command capability.
    pub fn register_command(mut self, descriptor: CommandDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.command_registry.push(descriptor);
        self
    }

    /// Register a domain-agnostic command-spine projection capability.
    pub fn register_command_projection(mut self, descriptor: CommandProjectionDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.command_projection_registry.push(descriptor);
        self
    }

    /// Register a domain-agnostic renderable component capability.
    pub fn register_component(mut self, descriptor: ComponentDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.component_registry.push(descriptor);
        self
    }

    pub fn register_mosaic_seam_paint_contract(
        mut self,
        contract: crate::capability::MosaicSeamPaintContract,
    ) -> Result<Self, crate::capability::MosaicSeamPaintContractDenial> {
        let candidate = self.mosaic_region_registry.install_seam_paint(contract)?;
        self.registration_candidates.push(candidate);
        Ok(self)
    }

    pub fn register_appearance_role(
        mut self,
        role: worth_ui_dsl::UiAppearanceRoleDeclaration,
    ) -> Result<Self, crate::capability::AppearanceRoleRegistrationDenial> {
        self.registration_candidates
            .push(self.appearance_role_registry.push(role)?);
        Ok(self)
    }

    #[cfg(any(test, feature = "certification-support"))]
    #[allow(
        dead_code,
        reason = "Gate 0 keeps theme registration certification-only"
    )]
    pub(crate) fn register_appearance_theme_bundle(
        mut self,
        bundle: crate::capability::FrozenAppearanceThemeCapabilities,
    ) -> Result<Self, crate::capability::FrozenAppearanceThemeCapabilitiesDenial> {
        let candidate = self.appearance_theme_registry.install(bundle)?;
        self.registration_candidates.push(candidate);
        Ok(self)
    }

    /// Register a domain-agnostic stable icon capability.
    pub fn register_icon(mut self, descriptor: IconDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.icon_registry.push(descriptor);
        self
    }

    pub fn register_intent_definition<I: UiIntent, D: UiIntentDefinitionDestination>(
        mut self,
        definition: UiIntentDefinition<I, D>,
    ) -> Result<Self, UiIntentDefinitionRegistrationError> {
        let candidate = self.intent_definition_registry.push(definition.erase())?;
        self.registration_candidates.push(candidate);
        Ok(self)
    }

    /// Register a domain-agnostic product-facing shell surface capability.
    pub fn register_surface(mut self, descriptor: SurfaceDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.surface_registry.push(descriptor);
        self
    }

    /// Register a domain-agnostic mosaic-owned structural region kind capability.
    pub fn register_mosaic_region_kind(mut self, descriptor: MosaicRegionKindDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.mosaic_region_registry.push(descriptor);
        self
    }

    /// Register a domain-agnostic runtime-owned mosaic placement policy capability.
    pub fn register_mosaic_placement_policy(
        mut self,
        descriptor: MosaicPlacementPolicyDescriptor,
    ) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.mosaic_placement_registry.push(descriptor);
        self
    }

    /// Register a domain-agnostic runtime-owned mosaic sizing contract capability.
    pub fn register_mosaic_sizing_contract(
        mut self,
        descriptor: MosaicSizingContractDescriptor,
    ) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.mosaic_sizing_registry.push(descriptor);
        self
    }

    /// Register a runtime-owned mosaic state slot preservation capability.
    pub fn register_mosaic_state_slot(mut self, descriptor: MosaicStateSlotDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.mosaic_state_slot_registry.push(descriptor);
        self
    }

    /// Register an explicit native platform capability seam.
    pub fn register_native_capability(mut self, descriptor: NativeCapabilityDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.native_capability_registry.push(descriptor);
        self
    }

    /// Register a runtime-owned plugin contribution-slot capability.
    pub fn register_plugin_slot(mut self, descriptor: PluginSlotDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.plugin_slot_registry.push(descriptor);
        self
    }

    /// Register a Query-owned view binding presentation capability.
    pub(crate) fn register_view_binding(mut self, descriptor: ViewBindingDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.view_binding_registry.push(descriptor);
        self
    }

    /// Register a runtime-owned outcome presentation projection capability.
    pub fn register_runtime_outcome_projection(
        mut self,
        descriptor: RuntimeOutcomeProjectionDescriptor,
    ) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.runtime_outcome_projection_registry.push(descriptor);
        self
    }

    /// Register a typed runtime/UI setting metadata capability.
    pub fn register_setting(mut self, descriptor: SettingDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.settings_registry.push(descriptor);
        self
    }

    /// Register domain-agnostic task presentation posture metadata.
    pub fn register_task_presentation(mut self, descriptor: TaskPresentationDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.task_presentation_registry.push(descriptor);
        self
    }

    /// Register a named semantic theme token capability.
    pub fn register_theme_token(mut self, descriptor: ThemeTokenDescriptor) -> Self {
        self.registration_candidates
            .push(descriptor.registration_candidate());
        self.theme_token_registry.push(descriptor);
        self
    }

    /// Materialize only stable diagnostic codes and required report topology.
    pub fn with_minimal_registration_diagnostics(mut self) -> Self {
        self.diagnostic_richness = CapabilityDiagnosticRichness::Minimal;
        self
    }

    /// Materialize richer diagnostic context without changing snapshot meaning.
    pub fn with_rich_registration_diagnostics(mut self) -> Self {
        self.diagnostic_richness = CapabilityDiagnosticRichness::Rich;
        self
    }
}
