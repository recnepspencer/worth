use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn explicit_work_budget_denies_before_subscription_selection_artifact_exists() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::KanbanGrouped),
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let budget = QuerySubscriptionWorkBudget::no_allocation(3, 8, 8, 32, 1);

    let error = select_query_subscription_family(input, budget).unwrap_err();

    assert_eq!(
        error.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::WorkBudgetExceeded
    );
    assert_eq!(error.counters().family_selection_count(), 0);
    assert_eq!(error.counters().family_denial_count(), 1);
    assert_eq!(error.counters().work_budget_denial_count(), 1);
}

#[test]
fn exhausted_lookup_budget_denies_without_claiming_a_registry_lookup() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let budget = QuerySubscriptionWorkBudget::scratch_buffer_only(8, 8, 8, 32, 0);

    let error = select_query_subscription_family(input, budget).unwrap_err();

    assert_eq!(
        error.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::WorkBudgetExceeded
    );
    assert_eq!(error.counters().family_registry_lookup_count(), 0);
    assert_eq!(error.counters().view_family_registry_lookup_count(), 0);
    assert_eq!(error.counters().family_selection_count(), 0);
    assert_eq!(error.counters().family_denial_count(), 1);
    assert_eq!(error.counters().work_budget_denial_count(), 1);
    assert_eq!(
        error
            .diagnostic()
            .expect("fixture digest must fit")
            .counter_projection()
            .label()
            .as_str(),
        error.counters().counter_projection().label()
    );
}

#[test]
fn no_allocation_budget_denies_before_digest_scratch_is_allocated() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");
    let budget = QuerySubscriptionWorkBudget::no_allocation(8, 8, 8, 32, 1);

    let error = select_query_subscription_family(input, budget).unwrap_err();

    assert_eq!(
        error.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::AllocationBudgetExceeded
    );
    assert_eq!(error.counters().family_selection_count(), 0);
    assert_eq!(error.counters().family_denial_count(), 1);
    assert_eq!(error.counters().work_budget_denial_count(), 1);
    assert_eq!(error.counters().equivalence_digest_part_count(), 0);
    assert_eq!(error.counters().scratch_allocation_count(), 0);
    assert_eq!(error.counters().forbidden_heap_allocation_denial_count(), 1);
}
