pub(super) fn bulk_workload_digest_basis(planned_routes: &[BridgePlannedRoute]) -> String {
    let mut basis = format!("bulk-workload|route-count={}", planned_routes.len());
    for route in planned_routes {
        basis.push_str("|route=");
        basis.push_str(route.route_identity().as_str());
        basis.push_str("|source-digest=");
        basis.push_str(route.source_digest().as_str());
        basis.push_str("|mapping-context=");
        basis.push_str(route.mapping_context().digest());
    }
    basis
}

pub(super) fn canonical_planning_digest_basis(
    runtime: &RuntimeBridge,
    workload_identity: &BridgeWorkloadIdentity,
    planned_routes: &[BridgePlannedRoute],
) -> String {
    let mut basis = format!(
        "bulk-planning-identity|workload={}|mapping-registry={}",
        workload_identity.as_str(),
        mapping_registry_digest(&runtime.mapping_registry),
    );
    for route in planned_routes {
        basis.push_str("|route=");
        basis.push_str(route.route_identity().as_str());
        basis.push_str("|planning=");
        basis.push_str(route.planning_provenance().digest());
        basis.push_str("|lowering=");
        basis.push_str(route.lowering_provenance().digest());
        basis.push_str("|truth-view=");
        basis.push_str(&route.source_branch().canonical_label());
        basis.push(':');
        basis.push_str(&route.source_snapshot().canonical_label());
        basis.push(':');
        basis.push_str(&route.source_commit().canonical_label());
        if let Some(lineage_context) = route.mapping_context().lineage_context() {
            basis.push_str("|continuity=");
            basis.push_str(lineage_context.authority_basis().digest());
        }
    }
    basis.push_str("|packetization-semantics=v1|reduction-semantics=v1");
    basis
}

pub(super) fn admission_profile_digest_basis(runtime: &RuntimeBridge) -> String {
    format!(
        "bulk-admission-profile|diagnostics-tier={}|route-record-limit={}|failure-record-limit={}|allow-replay={}|has-branch-head-source={}|has-lineage-source={}|has-reader-pool={}",
        diagnostics_tier_label(runtime.policy.diagnostics_tier()),
        runtime.policy.retention_budget().route_record_limit(),
        runtime.policy.retention_budget().failure_record_limit(),
        runtime.policy.allow_replay_artifacts(),
        runtime.truth_branch_head_source.is_some(),
        runtime.continuity_lineage_source.is_some(),
        runtime.snapshot_reader_pool.is_some(),
    )
}

fn mapping_registry_digest(registry: &FrozenMappingRegistry) -> Arc<str> {
    let mut basis = format!(
        "mapping-registry|registration-count={}",
        registry.registrations().len()
    );
    for registration in registry.registrations() {
        basis.push_str("|registration=");
        basis.push_str(registration.registration_identity().as_str());
    }
    digest_string("mapping-registry", &basis)
}

fn diagnostics_tier_label(tier: BridgeDiagnosticsTier) -> &'static str {
    match tier {
        BridgeDiagnosticsTier::Minimal => "minimal",
        BridgeDiagnosticsTier::Standard => "standard",
        BridgeDiagnosticsTier::Exhaustive => "exhaustive",
    }
}

pub(super) fn route_region_identities(
    packet_set: &PlannedBridgePacketSet,
) -> Vec<BulkPacketRegionIdentity> {
    let mut regions = packet_set
        .routing_packets()
        .iter()
        .map(route_packet_region_identity)
        .collect::<Vec<_>>();
    regions.extend(
        packet_set
            .truth_view_packets()
            .iter()
            .map(truth_view_packet_region_identity),
    );
    regions.extend(
        packet_set
            .continuity_packets()
            .iter()
            .map(continuity_packet_region_identity),
    );
    regions
}

pub(super) fn packet_region_identities_are_disjoint(packet_set: &PlannedBridgePacketSet) -> bool {
    let mut observed_regions = std::collections::BTreeSet::<BulkPacketRegionIdentity>::new();
    for region in route_region_identities(packet_set) {
        if !observed_regions.insert(region) {
            return false;
        }
    }
    true
}

fn route_packet_region_identity(packet: &TruthDeltaRoutingPacket) -> BulkPacketRegionIdentity {
    let basis = format!(
        "bulk-packet-region|family=route|route={}|snapshot={}|subscription-slice={}",
        packet.route_identity().as_str(),
        packet.source_snapshot(),
        packet.subscription_slice_identity().as_str(),
    );
    BulkPacketRegionIdentity::admit_bridge_owned(digest_string("bulk-packet-region", &basis))
}

fn truth_view_packet_region_identity(
    packet: &TruthViewMaterializationPacket,
) -> BulkPacketRegionIdentity {
    let basis = format!(
        "bulk-packet-region|family=truth-view|branch={}|snapshot={}|commit={}",
        packet.source_branch(),
        packet.source_snapshot(),
        packet.source_commit(),
    );
    BulkPacketRegionIdentity::admit_bridge_owned(digest_string("bulk-packet-region", &basis))
}

fn continuity_packet_region_identity(packet: &ContinuityRemapPacket) -> BulkPacketRegionIdentity {
    let basis = format!(
        "bulk-packet-region|family=continuity|continuity-member={}|branch={}|snapshot={}",
        packet.continuity_member_identity().as_str(),
        packet.branch_identity(),
        packet.snapshot_identity(),
    );
    BulkPacketRegionIdentity::admit_bridge_owned(digest_string("bulk-packet-region", &basis))
}

fn normalized_workload_width(summary: &NormalizedBridgeWorkloadSummary) -> usize {
    summary.route_count()
        + summary.invalidation_target_count()
        + summary.subscription_slice_count()
        + summary.snapshot_read_count()
        + summary.truth_view_member_count()
        + summary.continuity_member_count()
        + summary.branch_scope_count()
        + summary.snapshot_scope_count()
}

fn packet_entry_count(packet_set: &PlannedBridgePacketSet) -> usize {
    let reduction_entry_count = packet_set
        .reduction_packets()
        .iter()
        .map(|packet| {
            packet_set
                .routing_packets()
                .iter()
                .filter(|route| {
                    route.subscription_slice_identity()
                        == packet.reduced_subscription_slice_identity()
                })
                .count()
        })
        .sum::<usize>();
    packet_set
        .routing_packets()
        .iter()
        .map(|packet| packet.invalidation_target_count())
        .sum::<usize>()
        + packet_set
            .truth_view_packets()
            .iter()
            .map(|packet| packet.planned_route_count())
            .sum::<usize>()
        + packet_set
            .continuity_packets()
            .iter()
            .map(|packet| packet.prior_slice_count())
            .sum::<usize>()
        + packet_set.widening_packets().len()
        + reduction_entry_count
}

fn reduction_output_count(packet_set: &PlannedBridgePacketSet) -> usize {
    let continuity_output_count = packet_set
        .continuity_packets()
        .iter()
        .map(|packet| packet.continuity_member_identity())
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    let widening_output_count = packet_set
        .widening_packets()
        .iter()
        .map(|packet| (packet.widening_class(), packet.bounded_scope_identity()))
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    packet_set.reduction_packets().len()
        + packet_set.truth_view_packets().len()
        + continuity_output_count
        + widening_output_count
}

pub(super) fn planning_counters(
    normalized_summary: &NormalizedBridgeWorkloadSummary,
    packet_set: &PlannedBridgePacketSet,
    legality_class: BridgeParallelLegalityClass,
    profitability_class: BridgeParallelProfitabilityClass,
    admission_class: BridgeParallelAdmissionClass,
) -> BridgeBulkPlanningCounters {
    let bulk_packet_count = packet_set.routing_packets().len()
        + packet_set.truth_view_packets().len()
        + packet_set.continuity_packets().len()
        + packet_set.widening_packets().len()
        + packet_set.reduction_packets().len();
    let bulk_reduction_input_count = packet_set.reduction_packets().len()
        + packet_set.truth_view_packets().len()
        + packet_set.continuity_packets().len()
        + packet_set.widening_packets().len();
    BridgeBulkPlanningCounters::new(
        packet_set
            .routing_packets()
            .iter()
            .map(|packet| packet.invalidation_target_count())
            .sum(),
        normalized_workload_width(normalized_summary),
        bulk_packet_count,
        packet_entry_count(packet_set),
        bulk_reduction_input_count,
        reduction_output_count(packet_set),
        packet_set.widening_packets().len(),
        bulk_packet_count,
        bulk_reduction_input_count,
        0,
        0,
        legality_class,
        profitability_class,
        admission_class,
    )
}

pub(super) fn bulk_reducer_input_buffer_ceiling() -> usize {
    4096
}

pub(super) fn bulk_diagnostics_fragment_ceiling() -> usize {
    128
}

pub(super) fn diagnostics_fragment_count(failures: &[BridgeBulkPlanningFailure]) -> usize {
    failures.len() + 3
}

use super::*;
