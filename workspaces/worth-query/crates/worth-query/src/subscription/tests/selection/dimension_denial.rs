use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn malformed_family_dimensions_deny_before_slice_count_is_repaired() {
    let input = LiveQueryAdmissionArtifact::for_test_grouped_with_missing_grouping_width()
        .expect("fixture digest must fit");

    let error = select_query_subscription_family(input, roomy_budget()).unwrap_err();

    assert_eq!(
        error.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::InvalidAdmissionDimensions
    );
    assert_eq!(error.counters().family_selection_count(), 0);
    assert_eq!(error.counters().family_denial_count(), 1);
    assert_eq!(error.counters().admission_dimension_denial_count(), 1);
    assert_eq!(error.counters().equivalence_digest_part_count(), 0);
    assert_eq!(error.counters().scratch_allocation_count(), 0);
}

#[test]
fn view_family_mismatch_denies_before_bridge_lowering() {
    let input = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::KanbanGrouped),
        QuerySubscriptionConstructionSource::FacadeLive,
    )
    .expect("fixture digest must fit");

    let error = select_query_subscription_family(input, roomy_budget()).unwrap_err();

    assert_eq!(
        error.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::ViewFamilyLiveFamilyMismatch
    );
    assert_eq!(error.counters().family_selection_count(), 0);
    assert_eq!(error.counters().family_denial_count(), 1);
    assert_eq!(error.counters().view_family_registry_lookup_count(), 1);
}
