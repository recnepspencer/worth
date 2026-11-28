use crate::error::BridgeRouteError;
use crate::facade::RuntimeBridge;
use crate::routing::canonicalization::{
    canonical_route_entry_order, digest_string, lowering_provenance_digest_basis,
    planning_provenance_digest_basis, planning_summary_digest_basis, route_digest_basis,
    SnapshotReadRequestSetView,
};
use crate::routing::lowering::{
    BridgeLoweringPlan, BridgeLoweringProvenance, ValidatedBridgeLoweringPlan,
};

use super::super::canonical::{
    canonical_invalidation_targets, canonical_read_packet, canonical_route_record_entries,
    canonical_subscription_slices,
};
use super::super::ingestion::{into_eligible_bridge_routing, IngestedBridgePatch};
use super::super::summaries::{
    BridgeExecutionCounts, BridgePlanningProvenance, BridgePlanningSummary,
    BridgeRouteSourceSummary, BridgeRoutingSummary,
};
use super::super::BridgeRouteIdentity;
use super::types::{BridgePlannedExecution, BridgePlannedRoute, BridgePlanningArtifacts};

pub(crate) fn plan_ingested_patch(
    runtime: &RuntimeBridge,
    ingested: IngestedBridgePatch,
) -> Result<BridgePlannedRoute, BridgeRouteError> {
    let eligible = into_eligible_bridge_routing(runtime, ingested)?;
    let envelope = eligible.envelope();
    let mapping_context = eligible.mapping_context();

    let mut entries = eligible.entries().to_vec();
    entries.sort_by(canonical_route_entry_order);
    let route_basis = route_digest_basis(envelope, mapping_context, &entries);
    let route_identity =
        BridgeRouteIdentity::admit_bridge_owned(digest_string("route", &route_basis));

    let invalidation_targets = canonical_invalidation_targets(&entries);
    let subscription_slices = canonical_subscription_slices(&entries);
    let invalidation_target_count = invalidation_targets.len();
    let subscription_slice_count = subscription_slices.len();
    let read_packet = canonical_read_packet(&subscription_slices, &entries);
    let route_record_entries = if eligible.route_scope().omit_record_entries() {
        Default::default()
    } else {
        canonical_route_record_entries(&entries)
    };
    let read_packet_view = SnapshotReadRequestSetView::new(read_packet.reads());
    let planning_provenance_basis = planning_provenance_digest_basis(
        &route_identity,
        envelope,
        mapping_context,
        &entries,
        &read_packet_view,
    );
    let planning_provenance = BridgePlanningProvenance::new(
        route_identity.clone(),
        envelope.digest().clone(),
        digest_string("planning-provenance", &planning_provenance_basis),
    );
    let planning_summary_basis = planning_summary_digest_basis(
        &route_identity,
        entries.len(),
        invalidation_target_count,
        subscription_slice_count,
        read_packet.reads().len(),
    );
    let planning_summary = BridgePlanningSummary::new(
        route_identity.clone(),
        entries.len(),
        BridgeExecutionCounts::new(
            invalidation_target_count,
            subscription_slice_count,
            read_packet.reads().len(),
        ),
        digest_string("planning-summary", &planning_summary_basis),
    );

    let source_summary = BridgeRouteSourceSummary::new(
        envelope.branch_identity().clone(),
        envelope.commit_identity().clone(),
        envelope.patch_identity().clone(),
        envelope.snapshot_identity().clone(),
    );
    let routing_summary = BridgeRoutingSummary::new(
        route_identity.clone(),
        source_summary,
        envelope.producer_metadata().clone(),
        entries.len(),
        invalidation_target_count,
    );
    let lowering_plan = BridgeLoweringPlan::new(
        route_identity.clone(),
        envelope.branch_identity().clone(),
        envelope.commit_identity().clone(),
        envelope.patch_identity().clone(),
        envelope.snapshot_identity().clone(),
        invalidation_targets,
        subscription_slices,
        read_packet.reads().len(),
        BridgeLoweringProvenance::new(
            route_identity.clone(),
            planning_provenance.clone(),
            digest_string(
                "lowering-provenance",
                &lowering_provenance_digest_basis(
                    &route_identity,
                    planning_provenance.digest(),
                    envelope.commit_identity().as_str(),
                    envelope.patch_identity().as_str(),
                    envelope.snapshot_identity().as_str(),
                ),
            ),
        ),
    );
    let validated_lowering_plan = ValidatedBridgeLoweringPlan::from_plan(&lowering_plan)?;
    let counters = eligible
        .counters()
        .with_routing_entry_count(entries.len())
        .with_invalidation_target_count(invalidation_target_count)
        .with_snapshot_packet(read_packet.reads().len())
        .with_sort_input_width(
            entries.len()
                + invalidation_target_count
                + subscription_slice_count
                + read_packet.reads().len(),
        )
        .with_digest_computations(4 + lowering_plan.digest_computation_count())
        .with_digest_input_bytes(
            route_basis.len()
                + planning_provenance_basis.len()
                + planning_summary_basis.len()
                + lowering_provenance_digest_basis(
                    &route_identity,
                    planning_provenance.digest(),
                    envelope.commit_identity().as_str(),
                    envelope.patch_identity().as_str(),
                    envelope.snapshot_identity().as_str(),
                )
                .len()
                + lowering_plan.digest_input_bytes(),
        );

    Ok(BridgePlannedRoute::new(
        eligible.route_scope().clone(),
        mapping_context.clone(),
        route_identity,
        BridgeRouteSourceSummary::new(
            envelope.branch_identity().clone(),
            envelope.commit_identity().clone(),
            envelope.patch_identity().clone(),
            envelope.snapshot_identity().clone(),
        ),
        envelope.producer_metadata().clone(),
        envelope.digest().clone(),
        BridgePlanningArtifacts::new(planning_provenance, planning_summary),
        BridgePlannedExecution::new(
            routing_summary,
            read_packet,
            counters,
            lowering_plan,
            validated_lowering_plan,
            route_record_entries,
        ),
    ))
}

#[cfg(test)]
#[path = "planner_tests.rs"]
mod planner_tests;
