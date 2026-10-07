use crate::live::{LiveQueryFamily, LiveQueryPlan};

use super::super::{
    BundleResolvedBasisDigest, FrontierAwarePlan, FrontierBreadthPrediction,
    FrontierComplexityContract, FrontierPerformanceStatus, FrontierPlanFamily,
    FrontierPlanningCounters, FrontierPlanningError, FrontierPlanningReport,
    FrontierPredictionDriftOutcome, PacketEquivalenceContract, PacketMergeBoundary,
    PacketMergeContract, PlannedWorkPacket, PlannedWorkPacketFamily, PlannedWorkPacketSet,
};

pub(crate) fn lower_live_plan_to_frontier_plan(
    live: &LiveQueryPlan,
) -> Result<FrontierAwarePlan, FrontierPlanningError> {
    let basis_digest = BundleResolvedBasisDigest::from_basis_digest(
        live.progress_basis().current_basis().proof().digest(),
    );
    let relevance = live.descriptor().relevance_contract();
    let (
        family,
        packet_family,
        equivalence_contract,
        merge_contract,
        complexity_contract,
        performance_status,
        scope_summary,
        predicted_breadth,
    ) = match live.descriptor().family() {
        LiveQueryFamily::Detail => (
            FrontierPlanFamily::LiveDetail,
            PlannedWorkPacketFamily::LiveDetailRoot,
            PacketEquivalenceContract::LiveDescriptorAndProgressBasis,
            PacketMergeContract::LiveDetailResultBoundary,
            FrontierComplexityContract::live_detail(),
            FrontierPerformanceStatus::Verified,
            format!(
                "live_detail:{}:fields:{}",
                live.descriptor().plan_digest().as_str(),
                relevance.projected_fields().len()
            ),
            FrontierBreadthPrediction::new(relevance.projected_fields().len()),
        ),
        LiveQueryFamily::OrderedCollection => (
            FrontierPlanFamily::LiveOrderedCollection,
            PlannedWorkPacketFamily::LiveOrderedCollectionRoot,
            PacketEquivalenceContract::LiveDescriptorAndProgressBasis,
            PacketMergeContract::LiveOrderedCollectionResultBoundary,
            FrontierComplexityContract::live_ordered_collection(),
            FrontierPerformanceStatus::Verified,
            format!(
                "live_ordered_collection:{}:projected:{}:ordering:{}",
                live.descriptor().plan_digest().as_str(),
                relevance.projected_fields().len(),
                relevance.ordering_fields().len()
            ),
            FrontierBreadthPrediction::new(
                relevance.projected_fields().len() + relevance.ordering_fields().len(),
            ),
        ),
        LiveQueryFamily::BoundedMaterialization => (
            FrontierPlanFamily::LiveBoundedMaterialization,
            PlannedWorkPacketFamily::LiveBoundedMaterializationRoot,
            PacketEquivalenceContract::LiveDescriptorAndProgressBasis,
            PacketMergeContract::LiveBoundedMaterializationResultBoundary,
            FrontierComplexityContract::live_bounded_materialization(),
            FrontierPerformanceStatus::Debt,
            format!(
                "live_bounded_materialization:{}:projected:{}:ordering:{}:relations:{}",
                live.descriptor().plan_digest().as_str(),
                relevance.projected_fields().len(),
                relevance.ordering_fields().len(),
                relevance.traversal_relations().len()
            ),
            FrontierBreadthPrediction::new(
                relevance.projected_fields().len()
                    + relevance.ordering_fields().len()
                    + relevance.traversal_relations().len(),
            ),
        ),
    };

    let packet_merge_boundary =
        PacketMergeBoundary::new(merge_contract, &scope_summary, &basis_digest);
    let packet = PlannedWorkPacket::new(
        live.descriptor().plan_digest().clone(),
        packet_family,
        0,
        scope_summary,
        packet_merge_boundary,
        &basis_digest,
    );
    let packet_set = PlannedWorkPacketSet::new(vec![packet], equivalence_contract);
    let report = FrontierPlanningReport::new(
        family.clone(),
        live.descriptor().plan_digest().clone(),
        basis_digest.clone(),
        predicted_breadth.clone(),
        &packet_set,
    );
    let counters = FrontierPlanningCounters::single_route(
        predicted_breadth.value(),
        packet_set.packets().len(),
    );

    Ok(FrontierAwarePlan {
        query_digest: live.descriptor().query_digest().clone(),
        source_plan_digest: live.descriptor().plan_digest().clone(),
        family,
        bundle_basis_digest: basis_digest,
        packet_set,
        predicted_breadth,
        drift_outcome: FrontierPredictionDriftOutcome::WithinBudget,
        complexity_contract,
        performance_status,
        report,
        counters,
    })
}
