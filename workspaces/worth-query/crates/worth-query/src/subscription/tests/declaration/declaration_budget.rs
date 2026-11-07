use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn slice_budget_denies_before_declaration_artifact_exists() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::KanbanGrouped),
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
    let tight_budget = QuerySubscriptionSliceBudget::scratch_buffer_only(8, 8, 0, 8, 8, 8, 8, 8);

    let error = declare_query_subscription(selection, tight_budget).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionDeclarationDenialKind::SliceBudgetExceeded
    );
    assert_eq!(error.counters().declaration_count(), 0);
    assert_eq!(error.counters().declaration_denial_count(), 1);
    assert_eq!(error.counters().work_budget_denial_count(), 1);
    assert_eq!(error.counters().declaration_digest_part_count(), 0);
}

#[test]
fn no_allocation_denies_before_declaration_scratch_sort_or_digest() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
    let no_allocation_budget = QuerySubscriptionSliceBudget::no_allocation(8, 8, 8, 8, 8, 8, 8, 8);

    let error = declare_query_subscription(selection, no_allocation_budget).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionDeclarationDenialKind::AllocationBudgetExceeded
    );
    assert_eq!(error.counters().declaration_count(), 0);
    assert_eq!(error.counters().declaration_denial_count(), 1);
    assert_eq!(error.counters().declaration_digest_part_count(), 0);
    assert_eq!(error.counters().forbidden_heap_allocation_denial_count(), 1);
}

#[test]
fn admitted_declaration_exposes_exact_structural_counters() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();

    let declaration = declare_query_subscription(selection, roomy_slice_budget()).unwrap();

    assert_eq!(declaration.counters().declaration_count(), 1);
    assert_eq!(declaration.counters().declared_slice_count(), 4);
    assert_eq!(declaration.counters().deduplicated_slice_count(), 4);
    assert_eq!(declaration.counters().slice_deduplication_input_count(), 4);
    assert_eq!(declaration.counters().slice_sort_comparison_count(), 3);
    assert_eq!(declaration.counters().declaration_digest_part_count(), 18);
    assert_eq!(declaration.counters().masked_slice_denial_count(), 0);
    assert_eq!(declaration.counters().delivery_intent_denial_count(), 0);
}

#[test]
fn unsupported_grouping_slice_denies_before_declaration_digest() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::KanbanGrouped),
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
    let budget = roomy_slice_budget().without_grouping_slice_support();

    let error = declare_query_subscription(selection, budget).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionDeclarationDenialKind::UnsupportedGroupingSlice
    );
    assert_eq!(error.counters().declaration_count(), 0);
    assert_eq!(error.counters().declaration_denial_count(), 1);
    assert_eq!(error.counters().declaration_digest_part_count(), 0);
}

#[test]
fn masked_slice_request_denies_before_declaration_digest() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
    let budget = roomy_slice_budget().with_masked_slice_request_detected();

    let error = declare_query_subscription(selection, budget).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionDeclarationDenialKind::UnsupportedMaskedSlice
    );
    assert_eq!(error.counters().declaration_count(), 0);
    assert_eq!(error.counters().declaration_denial_count(), 1);
    assert_eq!(error.counters().masked_slice_denial_count(), 1);
    assert_eq!(error.counters().declaration_digest_part_count(), 0);
}

#[test]
fn unsupported_bounded_materialization_slice_denies_before_declaration_digest() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::BoundedMaterialization,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(input, roomy_budget()).unwrap();
    let budget = roomy_slice_budget().without_bounded_materialization_slice_support();

    let error = declare_query_subscription(selection, budget).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionDeclarationDenialKind::UnsupportedBoundedMaterializationSlice
    );
    assert_eq!(error.counters().declaration_count(), 0);
    assert_eq!(error.counters().declaration_denial_count(), 1);
    assert_eq!(error.counters().declaration_digest_part_count(), 0);
}
