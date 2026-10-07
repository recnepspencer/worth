use crate::basis::ExecutionPreflightBundle;

use super::super::{
    BoundedMaterializationFrontierPreflight, BundleResolvedBasisDigest, FrontierAwarePlan,
    FrontierBreadthPrediction, FrontierComplexityContract, FrontierPerformanceStatus,
    FrontierPlanFamily, FrontierPlanningCounters, FrontierPlanningError, FrontierPlanningReport,
    FrontierPredictionDriftOutcome, FrontierPreflightAdmissionError, PacketEquivalenceContract,
    PacketMergeBoundary, PacketMergeContract, PlannedWorkPacket, PlannedWorkPacketFamily,
    PlannedWorkPacketSet,
};

pub fn admit_bounded_materialization_frontier_preflight(
    preflight: ExecutionPreflightBundle,
) -> Result<BoundedMaterializationFrontierPreflight, FrontierPreflightAdmissionError> {
    let collection = preflight
        .plan()
        .collection()
        .ok_or(FrontierPreflightAdmissionError::UnsupportedFrontierFamily)?;
    if collection.traversal_bound().edge_classes().is_empty() {
        Err(FrontierPreflightAdmissionError::BoundedMaterializationRequired)
    } else {
        Ok(BoundedMaterializationFrontierPreflight::new(preflight))
    }
}

pub(crate) fn lower_preflight_to_frontier_plan(
    preflight: &ExecutionPreflightBundle,
) -> Result<FrontierAwarePlan, FrontierPlanningError> {
    let collection = preflight
        .plan()
        .collection()
        .ok_or(FrontierPlanningError::UnsupportedFrontierFamily)?;
    let basis_digest =
        BundleResolvedBasisDigest::from_basis_digest(preflight.basis().proof().digest());
    let (
        family,
        packet_family,
        equivalence_contract,
        merge_contract,
        complexity_contract,
        performance_status,
        scope_summary,
        predicted_breadth,
    ) = if collection.traversal_bound().edge_classes().is_empty() {
        (
            FrontierPlanFamily::OrderedCollection,
            PlannedWorkPacketFamily::OrderedCollectionRoot,
            PacketEquivalenceContract::CollectionDigestAndBasis,
            PacketMergeContract::OrderedCollectionResultBoundary,
            FrontierComplexityContract::ordered_collection(),
            FrontierPerformanceStatus::Verified,
            format!(
                "collection:{}:result_family:{}:ordering:{}",
                collection.digest().as_str(),
                collection
                    .post_read_shaping()
                    .result_family()
                    .digest_label(),
                collection.ordering_basis().entries().len()
            ),
            FrontierBreadthPrediction::new(
                preflight.plan().counters().planned_read_surface_count(),
            ),
        )
    } else {
        (
            FrontierPlanFamily::BoundedMaterialization,
            PlannedWorkPacketFamily::BoundedMaterializationRoot,
            PacketEquivalenceContract::BoundedTraversalDigestAndBasis,
            PacketMergeContract::BoundedMaterializationResultBoundary,
            FrontierComplexityContract::bounded_materialization(),
            FrontierPerformanceStatus::Debt,
            format!(
                "collection:{}:edge_classes:{}:depth:{}",
                collection.digest().as_str(),
                collection.traversal_bound().edge_classes().len(),
                collection.traversal_bound().depth_limit().value()
            ),
            FrontierBreadthPrediction::new(
                preflight.plan().counters().planned_read_surface_count()
                    + preflight
                        .plan()
                        .counters()
                        .planned_materialization_edge_class_count(),
            ),
        )
    };

    let packet_merge_boundary =
        PacketMergeBoundary::new(merge_contract, &scope_summary, &basis_digest);
    let packet = PlannedWorkPacket::new(
        preflight.plan().query().plan_digest().clone(),
        packet_family,
        0,
        scope_summary,
        packet_merge_boundary,
        &basis_digest,
    );
    let packet_set = PlannedWorkPacketSet::new(vec![packet], equivalence_contract);
    let report = FrontierPlanningReport::new(
        family.clone(),
        preflight.plan().query().plan_digest().clone(),
        basis_digest.clone(),
        predicted_breadth.clone(),
        &packet_set,
    );
    let counters = FrontierPlanningCounters::single_route(
        predicted_breadth.value(),
        packet_set.packets().len(),
    );

    Ok(FrontierAwarePlan {
        query_digest: preflight.plan().query().validated_query_digest().clone(),
        source_plan_digest: preflight.plan().query().plan_digest().clone(),
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
