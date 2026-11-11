use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn detail_live_family_selects_exact_subscription_with_budgeted_counters() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::Direct,
    )
    .expect("fixture digest must fit");

    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();

    assert_eq!(selection.family(), &QuerySubscriptionFamily::DetailExact);
    assert_eq!(
        selection.cost_posture(),
        &QuerySubscriptionCostPosture::BoundedExact
    );
    assert_eq!(
        selection.live_graph_access_posture(),
        &QuerySubscriptionLiveGraphAccessPosture::IncrementalMaintenancePlanned
    );
    assert_eq!(
        selection.bridge_posture(),
        &QuerySubscriptionBridgePosture::BridgeDeclarationAdmitted
    );
    assert_eq!(selection.required_slice_count(), 2);
    assert_eq!(selection.counters().family_selection_count(), 1);
    assert_eq!(selection.counters().family_registry_lookup_count(), 1);
    assert_eq!(selection.counters().view_family_registry_lookup_count(), 0);
    assert_eq!(selection.counters().equivalence_digest_part_count(), 23);
    assert_eq!(selection.counters().admission_dimension_denial_count(), 0);
    assert_eq!(selection.counters().work_budget_denial_count(), 0);
    assert_eq!(selection.counters().unknown_cost_denial_count(), 0);
    assert_eq!(selection.counters().raw_cdc_fallback_denial_count(), 0);
    assert_eq!(
        selection.counters().host_observer_inference_denial_count(),
        0
    );
    assert_eq!(
        selection.counters().relationship_proof_drift_denial_count(),
        0
    );
    assert_eq!(selection.counters().scratch_allocation_count(), 1);
    assert_eq!(
        selection
            .counters()
            .forbidden_heap_allocation_denial_count(),
        0
    );
}

#[test]
fn grouped_and_plain_collection_are_distinct_query_subscription_meanings() {
    let collection = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let grouped = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::KanbanGrouped),
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");

    let collection_selection =
        select_query_subscription_family(collection, roomy_budget()).unwrap();
    let grouped_selection = select_query_subscription_family(grouped, roomy_budget()).unwrap();

    assert_eq!(
        collection_selection.family(),
        &QuerySubscriptionFamily::CollectionMembership
    );
    assert_eq!(
        grouped_selection.family(),
        &QuerySubscriptionFamily::GroupedCollectionMembership
    );
    assert_eq!(
        grouped_selection.live_graph_access_posture(),
        &QuerySubscriptionLiveGraphAccessPosture::SnapshotRefreshSupportRequired
    );
    assert_eq!(grouped_selection.required_slice_count(), 6);
    assert_ne!(
        collection_selection
            .equivalence_basis()
            .equivalence_projection()
            .label(),
        grouped_selection
            .equivalence_basis()
            .equivalence_projection()
            .label()
    );
}

#[test]
fn inspector_detail_is_distinct_from_plain_detail() {
    let detail = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::Detail),
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let inspector = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::InspectorDetailFocused),
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");

    let detail_selection = select_query_subscription_family(detail, roomy_budget()).unwrap();
    let inspector_selection = select_query_subscription_family(inspector, roomy_budget()).unwrap();

    assert_eq!(
        detail_selection.family(),
        &QuerySubscriptionFamily::DetailExact
    );
    assert_eq!(
        inspector_selection.family(),
        &QuerySubscriptionFamily::InspectorDetailExact
    );
    assert_ne!(
        detail_selection
            .equivalence_basis()
            .equivalence_projection()
            .label(),
        inspector_selection
            .equivalence_basis()
            .equivalence_projection()
            .label()
    );
}

#[test]
fn bounded_materialization_family_keeps_its_own_subscription_vocabulary() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::BoundedMaterialization,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");

    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();

    assert_eq!(
        selection.family(),
        &QuerySubscriptionFamily::BoundedMaterialization
    );
    assert_eq!(
        selection.cost_posture(),
        &QuerySubscriptionCostPosture::BoundedMembership
    );
    assert_eq!(selection.required_slice_count(), 5);
}
