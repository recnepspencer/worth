#[cfg(test)]
use crate::capability::{
    CapabilitySnapshotBuilder, CapabilitySnapshotFreezeInput, CapabilitySupportCatalog,
    FrozenAppearanceRoleCapabilities, FrozenCommandCapabilities,
    FrozenCommandProjectionCapabilities, FrozenComponentCapabilities, FrozenIconCapabilities,
    FrozenIntentDefinitionCapabilities, FrozenMosaicPlacementCapabilities,
    FrozenMosaicRegionCapabilities, FrozenMosaicSizingCapabilities, FrozenMosaicStateCapabilities,
    FrozenNativeCapabilities, FrozenPluginSlotCapabilities,
    FrozenRuntimeOutcomeProjectionCapabilities, FrozenSettingCapabilities,
    FrozenSurfaceCapabilities, FrozenTaskPresentationCapabilities, FrozenThemeTokenCapabilities,
    FrozenViewBindingCapabilities, RegisteredCapabilitySet,
};

use super::CapabilitySnapshot;

impl CapabilitySnapshot {
    #[cfg(test)]
    pub(crate) fn from_registered_capabilities(
        registered_capabilities: RegisteredCapabilitySet,
    ) -> Self {
        Self::from_freeze_input(CapabilitySnapshotFreezeInput {
            registered_capabilities,
            appearance_roles: FrozenAppearanceRoleCapabilities::empty(),
            appearance_themes: None,
            commands: FrozenCommandCapabilities::empty(),
            command_projections: FrozenCommandProjectionCapabilities::empty(),
            components: FrozenComponentCapabilities::empty(),
            icons: FrozenIconCapabilities::empty(),
            intent_definitions: FrozenIntentDefinitionCapabilities::empty(),
            surfaces: FrozenSurfaceCapabilities::empty(),
            mosaic_regions: FrozenMosaicRegionCapabilities::empty(),
            mosaic_placement_policies: FrozenMosaicPlacementCapabilities::empty(),
            mosaic_sizing_contracts: FrozenMosaicSizingCapabilities::empty(),
            mosaic_state_slots: FrozenMosaicStateCapabilities::empty(),
            native_capabilities: FrozenNativeCapabilities::empty(),
            plugin_slots: FrozenPluginSlotCapabilities::empty(),
            view_bindings: FrozenViewBindingCapabilities::empty(),
            runtime_outcome_projections: FrozenRuntimeOutcomeProjectionCapabilities::empty(),
            settings: FrozenSettingCapabilities::empty(),
            task_presentations: FrozenTaskPresentationCapabilities::empty(),
            theme_tokens: FrozenThemeTokenCapabilities::empty(),
            support_catalog: CapabilitySupportCatalog::empty(),
        })
    }

    #[cfg(test)]
    pub(crate) fn from_freeze_input(input: CapabilitySnapshotFreezeInput) -> Self {
        CapabilitySnapshotBuilder::new(input).freeze()
    }
}
