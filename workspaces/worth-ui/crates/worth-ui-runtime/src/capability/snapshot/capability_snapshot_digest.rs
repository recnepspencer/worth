use super::{CapabilitySnapshotFreezeInput, SnapshotMetrics};

/// Deterministic identity for a frozen capability snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapabilitySnapshotDigest {
    value: u64,
}

impl CapabilitySnapshotDigest {
    pub(crate) fn from_freeze_input(
        metrics: SnapshotMetrics,
        input: &CapabilitySnapshotFreezeInput,
    ) -> Self {
        Self {
            value: 0x9e37_79b9_7f4a_7c15
                ^ metrics.registered_family_count() as u64
                ^ ((metrics.total_width() as u64) << 32)
                ^ input.appearance_roles.digest_basis().rotate_left(2)
                ^ input
                    .appearance_themes
                    .as_ref()
                    .map_or(0, |themes| themes.digest_basis())
                    .rotate_left(19)
                ^ input.commands.digest_basis().rotate_left(17)
                ^ input.command_projections.digest_basis().rotate_left(13)
                ^ input.components.digest_basis().rotate_left(29)
                ^ input.icons.digest_basis().rotate_left(41)
                ^ input.intent_definitions.digest_basis().rotate_left(47)
                ^ input.surfaces.digest_basis().rotate_left(53)
                ^ input.mosaic_regions.digest_basis().rotate_left(7)
                ^ input
                    .mosaic_placement_policies
                    .digest_basis()
                    .rotate_left(19)
                ^ input.mosaic_sizing_contracts.digest_basis().rotate_left(31)
                ^ input.mosaic_state_slots.digest_basis().rotate_left(43)
                ^ input.native_capabilities.digest_basis().rotate_left(59)
                ^ input.plugin_slots.digest_basis().rotate_left(3)
                ^ input.view_bindings.digest_basis().rotate_left(5)
                ^ input
                    .runtime_outcome_projections
                    .digest_basis()
                    .rotate_left(23)
                ^ input.settings.digest_basis().rotate_left(37)
                ^ input.task_presentations.digest_basis().rotate_left(43)
                ^ input.theme_tokens.digest_basis().rotate_left(11),
        }
    }

    /// Stable numeric digest value for equality and later inspection.
    pub fn as_u64(self) -> u64 {
        self.value
    }
}
