#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiActiveThemeBinding {
    pub(super) surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(super) binding_generation: u64,
    pub(super) capability: super::UiThemeCapabilityReceipt,
}

impl UiActiveThemeBinding {
    pub(crate) const fn surface(&self) -> worth_ui_host_contract::UiSemanticSurfaceIdentity {
        self.surface
    }

    pub(crate) const fn binding_generation(&self) -> u64 {
        self.binding_generation
    }

    pub(crate) const fn capability(&self) -> &super::UiThemeCapabilityReceipt {
        &self.capability
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        application: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) -> Self {
        Self {
            surface,
            binding_generation: 1,
            capability: super::UiThemeCapabilityReceipt::for_test(surface, application),
        }
    }
}
