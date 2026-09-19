use crate::capability::{
    CapabilitySnapshot, CapabilitySnapshotFreezeInput, CapabilitySupportCatalog,
    RegisteredCapabilitySet, RegistrationCandidate,
};

pub(super) fn snapshot_with_support_catalog(
    base: &CapabilitySnapshot,
    support_catalog: CapabilitySupportCatalog,
) -> CapabilitySnapshot {
    CapabilitySnapshot::from_freeze_input(CapabilitySnapshotFreezeInput {
        appearance_roles: base.appearance_roles().clone(),
        appearance_themes: base.appearance_themes().cloned(),
        registered_capabilities: clone_registered_capabilities(base),
        commands: base.commands().clone(),
        command_projections: base.command_projections().clone(),
        components: base.components().clone(),
        icons: base.icons().clone(),
        intent_definitions: base.intent_definitions().clone(),
        surfaces: base.surfaces().clone(),
        mosaic_regions: base.mosaic_regions().clone(),
        mosaic_placement_policies: base.mosaic_placement_policies().clone(),
        mosaic_sizing_contracts: base.mosaic_sizing_contracts().clone(),
        mosaic_state_slots: base.mosaic_state_slots().clone(),
        native_capabilities: base.native_capabilities().clone(),
        plugin_slots: base.plugin_slots().clone(),
        view_bindings: base.view_bindings().clone(),
        runtime_outcome_projections: base.runtime_outcome_projections().clone(),
        settings: base.settings().clone(),
        task_presentations: base.task_presentations().clone(),
        theme_tokens: base.theme_tokens().clone(),
        support_catalog,
    })
}

pub(super) fn merge_support_candidates(
    mut base: Vec<RegistrationCandidate>,
    extra: impl IntoIterator<Item = RegistrationCandidate>,
) -> CapabilitySupportCatalog {
    for candidate in extra {
        base.retain(|existing| {
            existing.family_name() != candidate.family_name()
                || existing.identity_text() != candidate.identity_text()
        });
        base.push(candidate);
    }
    CapabilitySupportCatalog::from_registration_candidates(&base)
}

fn clone_registered_capabilities(snapshot: &CapabilitySnapshot) -> RegisteredCapabilitySet {
    let registered = snapshot.registered_capabilities();
    RegisteredCapabilitySet::from_counts(
        registered.registered_family_count(),
        registered.total_width(),
    )
}
