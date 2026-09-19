use crate::capability::{
    CapabilitySnapshot, CapabilitySnapshotDigest, CapabilitySnapshotFreezeInput,
    FrozenCapabilityFamily, SnapshotFreezeReport, SnapshotMetrics, APPEARANCE_ROLE_FAMILY_NAME,
    APPEARANCE_THEME_FAMILY_NAME, COMMAND_FAMILY_NAME, COMMAND_PROJECTION_FAMILY_NAME,
    COMPONENT_FAMILY_NAME, ICON_FAMILY_NAME, INTENT_DEFINITION_FAMILY_NAME,
    MOSAIC_PLACEMENT_POLICY_FAMILY_NAME, MOSAIC_REGION_KIND_FAMILY_NAME,
    MOSAIC_SEAM_PAINT_FAMILY_NAME, MOSAIC_SIZING_CONTRACT_FAMILY_NAME,
    MOSAIC_STATE_SLOT_FAMILY_NAME, NATIVE_CAPABILITY_FAMILY_NAME, PLUGIN_SLOT_FAMILY_NAME,
    RUNTIME_OUTCOME_PROJECTION_FAMILY_NAME, SETTING_FAMILY_NAME, SURFACE_FAMILY_NAME,
    TASK_PRESENTATION_FAMILY_NAME, THEME_TOKEN_FAMILY_NAME, VIEW_BINDING_FAMILY_NAME,
};

use super::super::validate_snapshot_references;

pub(crate) struct CapabilitySnapshotBuilder {
    input: CapabilitySnapshotFreezeInput,
}

impl CapabilitySnapshotBuilder {
    pub(crate) fn new(input: CapabilitySnapshotFreezeInput) -> Self {
        Self { input }
    }

    pub(crate) fn freeze(self) -> CapabilitySnapshot {
        let metrics = self.input.registered_capabilities.snapshot_metrics();
        let digest = digest_for_input(metrics, &self.input);
        let freeze_report = freeze_report_for_input(&self.input);
        let validation_report = validate_snapshot_references(&self.input);
        CapabilitySnapshot::from_freeze_parts(
            self.input,
            digest,
            metrics,
            freeze_report,
            validation_report,
        )
    }
}

fn digest_for_input(
    metrics: SnapshotMetrics,
    input: &CapabilitySnapshotFreezeInput,
) -> CapabilitySnapshotDigest {
    CapabilitySnapshotDigest::from_freeze_input(metrics, input)
}

fn freeze_report_for_input(input: &CapabilitySnapshotFreezeInput) -> SnapshotFreezeReport {
    SnapshotFreezeReport::new(vec![
        family(
            APPEARANCE_ROLE_FAMILY_NAME,
            input.appearance_roles.len(),
            input.appearance_roles.digest_basis(),
        ),
        family(
            APPEARANCE_THEME_FAMILY_NAME,
            usize::from(input.appearance_themes.is_some()),
            input
                .appearance_themes
                .as_ref()
                .map_or(0, |themes| themes.digest_basis()),
        ),
        family(
            COMMAND_FAMILY_NAME,
            input.commands.len(),
            input.commands.digest_basis(),
        ),
        family(
            COMMAND_PROJECTION_FAMILY_NAME,
            input.command_projections.len(),
            input.command_projections.digest_basis(),
        ),
        family(
            COMPONENT_FAMILY_NAME,
            input.components.len(),
            input.components.digest_basis(),
        ),
        family(
            ICON_FAMILY_NAME,
            input.icons.len(),
            input.icons.digest_basis(),
        ),
        family(
            INTENT_DEFINITION_FAMILY_NAME,
            input.intent_definitions.len(),
            input.intent_definitions.digest_basis(),
        ),
        family(
            SURFACE_FAMILY_NAME,
            input.surfaces.len(),
            input.surfaces.digest_basis(),
        ),
        family(
            MOSAIC_REGION_KIND_FAMILY_NAME,
            input.mosaic_regions.len(),
            input.mosaic_regions.region_kind_digest_basis(),
        ),
        family(
            MOSAIC_SEAM_PAINT_FAMILY_NAME,
            usize::from(input.mosaic_regions.seam_paint().is_some()),
            input.mosaic_regions.seam_paint_digest_basis(),
        ),
        family(
            MOSAIC_PLACEMENT_POLICY_FAMILY_NAME,
            input.mosaic_placement_policies.len(),
            input.mosaic_placement_policies.digest_basis(),
        ),
        family(
            MOSAIC_SIZING_CONTRACT_FAMILY_NAME,
            input.mosaic_sizing_contracts.len(),
            input.mosaic_sizing_contracts.digest_basis(),
        ),
        family(
            MOSAIC_STATE_SLOT_FAMILY_NAME,
            input.mosaic_state_slots.len(),
            input.mosaic_state_slots.digest_basis(),
        ),
        family(
            NATIVE_CAPABILITY_FAMILY_NAME,
            input.native_capabilities.len(),
            input.native_capabilities.digest_basis(),
        ),
        family(
            PLUGIN_SLOT_FAMILY_NAME,
            input.plugin_slots.len(),
            input.plugin_slots.digest_basis(),
        ),
        family(
            VIEW_BINDING_FAMILY_NAME,
            input.view_bindings.len(),
            input.view_bindings.digest_basis(),
        ),
        family(
            RUNTIME_OUTCOME_PROJECTION_FAMILY_NAME,
            input.runtime_outcome_projections.len(),
            input.runtime_outcome_projections.digest_basis(),
        ),
        family(
            SETTING_FAMILY_NAME,
            input.settings.len(),
            input.settings.digest_basis(),
        ),
        family(
            TASK_PRESENTATION_FAMILY_NAME,
            input.task_presentations.len(),
            input.task_presentations.digest_basis(),
        ),
        family(
            THEME_TOKEN_FAMILY_NAME,
            input.theme_tokens.len(),
            input.theme_tokens.digest_basis(),
        ),
    ])
}

fn family(family_name: &'static str, width: usize, digest_basis: u64) -> FrozenCapabilityFamily {
    FrozenCapabilityFamily::new(family_name, width, digest_basis)
}
