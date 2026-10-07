use crate::live::promote_preflight_bundle_to_live;
use crate::planning::{
    lower_execution_preflight_to_frontier_plan, lower_frontier_planning_bundle,
    lower_live_plan_to_frontier_plan, FrontierPlanFamily, FrontierPlanningError,
    FrontierPlanningInput, FrontierPredictionDriftOutcome, PacketMergeContract,
    PlannedWorkPacketFamily,
};
#[test]
fn ordered_collection_preflight_lowers_to_stable_frontier_plan() {
    let preflight =
        crate::harness::fixtures::execution_preflights::ordered_collection_without_traversal_preflight();

    let first = lower_execution_preflight_to_frontier_plan(&preflight)
        .expect("ordered collection preflight should lower");
    let second = lower_execution_preflight_to_frontier_plan(&preflight)
        .expect("repeat lowering should stay stable");

    assert_eq!(
        first.query_digest(),
        preflight.plan().query().validated_query_digest(),
        "ordered preflight lowering must preserve validated query identity exactly"
    );
    assert_eq!(first.family(), &FrontierPlanFamily::OrderedCollection);
    assert_eq!(
        first.packet_set().packets()[0].family(),
        &PlannedWorkPacketFamily::OrderedCollectionRoot
    );
    assert_eq!(
        first.report().packet_merge_contract(),
        &PacketMergeContract::OrderedCollectionResultBoundary
    );
    assert_eq!(
        first.drift_outcome(),
        &FrontierPredictionDriftOutcome::WithinBudget
    );
    assert_eq!(
        first.bundle_basis_digest().as_str(),
        preflight.basis().proof().digest().as_str()
    );
    assert_eq!(first.packet_set().packets().len(), 1);
    assert_eq!(
        first.packet_set().packets()[0].digest(),
        second.packet_set().packets()[0].digest()
    );
    assert_eq!(
        first.report().posture_digest(),
        second.report().posture_digest()
    );
    assert_eq!(
        first.counters().frontier_planning_invocation_count(),
        1,
        "single-route lowering should emit one planning invocation"
    );
}
#[test]
fn bounded_materialization_lowers_with_distinct_packet_family_and_merge_contract() {
    let ordered =
        crate::harness::fixtures::execution_preflights::ordered_collection_without_traversal_preflight();
    let bounded = crate::harness::fixtures::execution_preflights::ordered_collection_preflight();

    let ordered_plan = lower_execution_preflight_to_frontier_plan(&ordered)
        .expect("ordered collection preflight should lower");
    let bounded_plan = lower_execution_preflight_to_frontier_plan(&bounded)
        .expect("bounded materialization preflight should lower");

    assert_eq!(
        bounded_plan.family(),
        &FrontierPlanFamily::BoundedMaterialization
    );
    assert_eq!(
        bounded_plan.packet_set().packets()[0].family(),
        &PlannedWorkPacketFamily::BoundedMaterializationRoot
    );
    assert_eq!(
        bounded_plan.report().packet_merge_contract(),
        &PacketMergeContract::BoundedMaterializationResultBoundary
    );
    assert_ne!(
        ordered_plan.packet_set().packets()[0].digest(),
        bounded_plan.packet_set().packets()[0].digest()
    );
    assert_ne!(
        ordered_plan.report().packet_merge_contract(),
        bounded_plan.report().packet_merge_contract()
    );
}

#[test]
fn live_plan_lowering_preserves_descriptor_identity_and_uses_planner_packets() {
    let preflight =
        crate::harness::fixtures::execution_preflights::ordered_collection_without_traversal_preflight();
    let live = promote_preflight_bundle_to_live(&preflight)
        .expect("ordered collection preflight should promote to live");

    let plan = lower_live_plan_to_frontier_plan(&live).expect("live plan should lower");

    assert_eq!(
        plan.source_plan_digest(),
        live.descriptor().plan_digest(),
        "frontier lowering should preserve live descriptor plan identity"
    );
    assert_eq!(
        plan.query_digest(),
        live.descriptor().query_digest(),
        "frontier lowering should preserve live descriptor query identity"
    );
    assert_eq!(plan.family(), &FrontierPlanFamily::LiveOrderedCollection);
    assert_eq!(
        plan.packet_set().packets()[0].family(),
        &PlannedWorkPacketFamily::LiveOrderedCollectionRoot
    );
    assert_eq!(
        plan.bundle_basis_digest().as_str(),
        live.progress_basis()
            .current_basis()
            .proof()
            .digest()
            .as_str()
    );
}

#[test]
fn source_plan_digest_changes_frontier_posture_digest() {
    let ascending =
        crate::harness::fixtures::execution_preflights::ordered_collection_without_traversal_preflight();
    let descending =
        crate::harness::fixtures::execution_preflights::descending_collection_preflight();

    let ascending_plan = lower_execution_preflight_to_frontier_plan(&ascending)
        .expect("ascending collection preflight should lower");
    let descending_plan = lower_execution_preflight_to_frontier_plan(&descending)
        .expect("descending collection preflight should lower");

    assert_ne!(
        ascending.plan().query().plan_digest(),
        descending.plan().query().plan_digest()
    );
    assert_ne!(
        ascending_plan.report().posture_digest(),
        descending_plan.report().posture_digest()
    );
}

#[test]
fn same_basis_bundle_lowers_with_exact_bundle_basis_digest() {
    let ordered =
        crate::harness::fixtures::execution_preflights::ordered_collection_without_traversal_preflight();
    let bounded = crate::harness::fixtures::execution_preflights::ordered_collection_preflight();

    let bundle = lower_frontier_planning_bundle(&[
        FrontierPlanningInput::from(ordered.clone()),
        FrontierPlanningInput::from(bounded.clone()),
    ])
    .expect("same-basis frontier bundle should lower");

    assert_eq!(bundle.route_plans().len(), 2);
    assert_eq!(
        bundle.bundle_basis_digest().as_str(),
        ordered.basis().proof().digest().as_str()
    );
    assert!(bundle
        .route_plans()
        .iter()
        .all(|route| route.bundle_basis_digest() == bundle.bundle_basis_digest()));
    assert_eq!(bundle.counters().planned_bundle_route_count(), 2);
}

#[test]
fn mixed_basis_bundle_is_denied_even_when_basis_class_matches() {
    let ordered =
        crate::harness::fixtures::execution_preflights::ordered_collection_without_traversal_preflight();
    let alternate =
        crate::harness::fixtures::execution_preflights::alternate_basis_ordered_collection_preflight();

    let error = lower_frontier_planning_bundle(&[
        FrontierPlanningInput::from(ordered.clone()),
        FrontierPlanningInput::from(alternate.clone()),
    ])
    .expect_err("mixed basis bundle should be rejected");

    match error {
        FrontierPlanningError::MixedBasisBundle {
            expected_basis_digest,
            found_basis_digest,
        } => {
            assert_eq!(
                expected_basis_digest.as_str(),
                ordered.basis().proof().digest().as_str()
            );
            assert_eq!(
                found_basis_digest.as_str(),
                alternate.basis().proof().digest().as_str()
            );
            assert_ne!(expected_basis_digest, found_basis_digest);
        }
        other => panic!("expected mixed basis denial, got {other:?}"),
    }
}

#[test]
fn unsupported_bundle_composition_is_distinguishable_from_mixed_basis_denial() {
    let preflight =
        crate::harness::fixtures::execution_preflights::ordered_collection_without_traversal_preflight();
    let live = promote_preflight_bundle_to_live(&preflight)
        .expect("ordered collection preflight should promote to live");

    let error = lower_frontier_planning_bundle(&[
        FrontierPlanningInput::from(preflight),
        FrontierPlanningInput::from(live),
    ])
    .expect_err("mixed execution/live bundle should be rejected as unsupported composition");

    assert_eq!(error, FrontierPlanningError::UnsupportedBundleComposition);
}
