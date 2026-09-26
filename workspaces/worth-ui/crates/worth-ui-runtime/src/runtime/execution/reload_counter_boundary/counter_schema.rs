use crate::runtime::{WorthUiMeasurementCounterPacket, WorthUiRuntimeCounterFamily};

use super::denial::{WorthUiReloadCounterBoundaryDenial, WorthUiReloadCounterBoundaryDenialReason};

pub(crate) const ADMISSION_NAMES: &[&str] = &[
    "reload.candidate_admission.candidate_proof_checks",
    "reload.candidate_admission.snapshot_compatibility_checks",
    "reload.candidate_admission.runtime_posture_checks",
];

pub(crate) const ARTIFACT_COMPARISON_NAMES: &[&str] =
    &["reload.artifact_comparison.artifact_comparisons"];

pub(crate) const IMPACT_NARROWING_NAMES: &[&str] = &[
    "reload.impact_narrowing.impact_classifications_consumed",
    "reload.impact_narrowing.dependency_metadata_reads",
    "reload.impact_narrowing.module_impact_lookups",
    "reload.impact_narrowing.subtree_impact_lookups",
    "reload.impact_narrowing.runtime_hook_lookups",
    "reload.impact_narrowing.subtree_digest_lookups",
    "reload.impact_narrowing.full_artifact_scans",
];

pub(crate) const IDENTITY_NAMES: &[&str] = &[
    "reload.identity_replacement.active_nodes_indexed",
    "reload.identity_replacement.candidate_nodes_indexed",
    "reload.identity_replacement.stable_seed_lookups",
    "reload.identity_replacement.matches_emitted",
];

pub(crate) const RECONCILIATION_NAMES: &[&str] = &[
    "reload.durable_state_reconciliation.families_reconciled",
    "reload.durable_state_reconciliation.nodes_reconciled",
    "reload.durable_state_reconciliation.receipts",
    "reload.durable_state_reconciliation.query_posture_required",
];

pub(crate) const QUERY_REBIND_NAMES: &[&str] = &[
    "reload.query_rebind_planning.bindings_planned",
    "reload.query_rebind_planning.bindings_preserved",
    "reload.query_rebind_planning.bindings_rebound",
    "reload.query_rebind_planning.bindings_retired",
];

pub(crate) const PLAN_LOWERING_NAMES: &[&str] = &[
    "plan.lowering.staged_node_inputs",
    "plan.lowering.query_binding_inputs",
    "plan.lowering.reconciliation_receipt_inputs",
    "plan.lowering.component_hook_inputs",
    "plan.lowering.readiness_verifications",
    "plan.lowering.epoch_verifications",
    "plan.lowering.source_parse_count",
    "plan.lowering.registry_string_lookup_count",
];

pub(crate) const PLAN_ASSEMBLY_NAMES: &[&str] = &[
    "plan.assembly.handle_plan_node_inputs",
    "plan.assembly.component_handles",
    "plan.assembly.command_handles",
    "plan.assembly.token_handles",
    "plan.assembly.collision_checks",
    "plan.assembly.handle_broad_registry_scans",
    "plan.assembly.topology_nodes",
    "plan.assembly.lookup_entries",
    "plan.assembly.topology_validations",
    "plan.assembly.topology_artifact_tree_scans",
    "plan.assembly.topology_broad_registry_scans",
    "plan.assembly.plan_digest_count",
    "plan.assembly.plan_node_digest_count",
    "plan.assembly.plan_equivalence_comparisons",
    "plan.assembly.equivalence_artifact_tree_scans",
];

pub(crate) fn validate_receipt_packet_schema(
    packets: &[WorthUiMeasurementCounterPacket],
) -> Result<(), WorthUiReloadCounterBoundaryDenial> {
    validate_unique_phase_packets(packets)?;
    for packet in packets {
        validate_packet_schema(packet)?;
    }
    Ok(())
}

pub(crate) fn validate_packet_schema(
    packet: &WorthUiMeasurementCounterPacket,
) -> Result<(), WorthUiReloadCounterBoundaryDenial> {
    let expected = expected_names_for_family(packet.family());
    if expected.is_empty() {
        return Err(WorthUiReloadCounterBoundaryDenial::new(
            WorthUiReloadCounterBoundaryDenialReason::UnexpectedCounterRow,
        ));
    }
    validate_no_duplicate_rows(packet)?;
    validate_required_rows(packet, expected)?;
    validate_no_unexpected_rows(packet, expected)
}

fn validate_unique_phase_packets(
    packets: &[WorthUiMeasurementCounterPacket],
) -> Result<(), WorthUiReloadCounterBoundaryDenial> {
    let mut families: Vec<_> = packets.iter().map(|packet| packet.family()).collect();
    families.sort();
    if families.windows(2).any(|window| window[0] == window[1]) {
        return Err(WorthUiReloadCounterBoundaryDenial::new(
            WorthUiReloadCounterBoundaryDenialReason::DuplicateCounterPacket,
        ));
    }
    Ok(())
}

fn expected_names_for_family(family: WorthUiRuntimeCounterFamily) -> &'static [&'static str] {
    match family {
        WorthUiRuntimeCounterFamily::ReloadCandidateAdmission => ADMISSION_NAMES,
        WorthUiRuntimeCounterFamily::ArtifactComparison => ARTIFACT_COMPARISON_NAMES,
        WorthUiRuntimeCounterFamily::ImpactNarrowing => IMPACT_NARROWING_NAMES,
        WorthUiRuntimeCounterFamily::IdentityReplacement => IDENTITY_NAMES,
        WorthUiRuntimeCounterFamily::DurableStateReconciliation => RECONCILIATION_NAMES,
        WorthUiRuntimeCounterFamily::QueryRebindPlanning => QUERY_REBIND_NAMES,
        WorthUiRuntimeCounterFamily::PlanLowering => PLAN_LOWERING_NAMES,
        WorthUiRuntimeCounterFamily::PlanAssembly => PLAN_ASSEMBLY_NAMES,
        WorthUiRuntimeCounterFamily::SourceIngress
        | WorthUiRuntimeCounterFamily::LaneAdmission
        | WorthUiRuntimeCounterFamily::OrdinaryLaneExecution
        | WorthUiRuntimeCounterFamily::VirtualizedDataExecution
        | WorthUiRuntimeCounterFamily::CanvasSpatialExecution
        | WorthUiRuntimeCounterFamily::RealtimeOverlayExecution
        | WorthUiRuntimeCounterFamily::Activation
        | WorthUiRuntimeCounterFamily::CommittedAllocationActivation
        | WorthUiRuntimeCounterFamily::SteadyFrameRendering
        | WorthUiRuntimeCounterFamily::DiagnosticsProjection => &[],
    }
}

fn validate_no_duplicate_rows(
    packet: &WorthUiMeasurementCounterPacket,
) -> Result<(), WorthUiReloadCounterBoundaryDenial> {
    if packet
        .counters()
        .windows(2)
        .any(|window| window[0].name() == window[1].name())
    {
        return Err(WorthUiReloadCounterBoundaryDenial::new(
            WorthUiReloadCounterBoundaryDenialReason::DuplicateCounterRow,
        ));
    }
    Ok(())
}

fn validate_required_rows(
    packet: &WorthUiMeasurementCounterPacket,
    expected: &[&str],
) -> Result<(), WorthUiReloadCounterBoundaryDenial> {
    if expected.iter().any(|name| {
        packet
            .counters()
            .iter()
            .all(|counter| counter.name() != *name)
    }) {
        return Err(WorthUiReloadCounterBoundaryDenial::new(
            WorthUiReloadCounterBoundaryDenialReason::MissingRequiredCounterRow,
        ));
    }
    Ok(())
}

fn validate_no_unexpected_rows(
    packet: &WorthUiMeasurementCounterPacket,
    expected: &[&str],
) -> Result<(), WorthUiReloadCounterBoundaryDenial> {
    if packet
        .counters()
        .iter()
        .any(|counter| expected.iter().all(|name| *name != counter.name()))
    {
        return Err(WorthUiReloadCounterBoundaryDenial::new(
            WorthUiReloadCounterBoundaryDenialReason::UnexpectedCounterRow,
        ));
    }
    Ok(())
}
