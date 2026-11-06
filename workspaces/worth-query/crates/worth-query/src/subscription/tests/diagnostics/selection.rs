use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

use super::*;

#[test]
fn view_family_mismatch_carries_family_selection_diagnostic_evidence() {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionConstructionSource::Direct,
    );

    let error = select_query_subscription_family(live, roomy_budget()).unwrap_err();

    assert_eq!(
        error.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::ViewFamilyLiveFamilyMismatch
    );
    assert_eq!(
        error.diagnostic().stage(),
        &QuerySubscriptionDiagnosticStage::ViewMismatch
    );
    assert_eq!(
        error.diagnostic().outcome(),
        &QuerySubscriptionDiagnosticOutcome::Denied
    );
    assert_eq!(
        error.diagnostic().counter_projection().label().as_str(),
        error.counters().counter_projection().label()
    );
    assert_eq!(error.counters().family_registry_lookup_count(), 1);
    assert_eq!(error.counters().view_family_registry_lookup_count(), 1);
    assert_eq!(error.counters().family_selection_count(), 0);
    assert_eq!(error.counters().family_denial_count(), 1);
    assert_eq!(error.counters().declaration_count(), 0);
    assert_eq!(error.counters().bridge_lowering_count(), 0);
    assert!(!error.diagnostic().source_projection().label().is_empty());
    assert!(!error.diagnostic().evidence_projection().label().is_empty());
}

#[test]
fn masked_detail_table_and_grouped_requests_deny_before_bridge_lowering() {
    for (
        live_family,
        view_family,
        expected_family,
        expected_ordering_width,
        expected_grouping_width,
    ) in [
        (
            LiveQueryFamily::Detail,
            None,
            QuerySubscriptionFamily::DetailExact,
            0,
            0,
        ),
        (
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            QuerySubscriptionFamily::CollectionMembership,
            1,
            0,
        ),
        (
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::KanbanGrouped),
            QuerySubscriptionFamily::GroupedCollectionMembership,
            1,
            1,
        ),
    ] {
        let live = LiveQueryAdmissionArtifact::for_test(
            live_family,
            view_family,
            QuerySubscriptionConstructionSource::Direct,
        );
        let selection = select_query_subscription_family(live, roomy_budget()).unwrap();

        assert_eq!(selection.family(), &expected_family);
        assert_eq!(selection.ordering_width(), expected_ordering_width);
        assert_eq!(selection.grouping_width(), expected_grouping_width);

        let error = declare_query_subscription(
            selection,
            roomy_slice_budget().with_masked_slice_request_detected(),
        )
        .unwrap_err();

        assert_eq!(
            error.denial_kind(),
            &QuerySubscriptionDeclarationDenialKind::UnsupportedMaskedSlice
        );
        assert_eq!(
            error.diagnostic().stage(),
            &QuerySubscriptionDiagnosticStage::Declaration
        );
        assert_eq!(error.counters().declaration_count(), 0);
        assert_eq!(error.counters().declaration_denial_count(), 1);
        assert_eq!(error.counters().bridge_lowering_count(), 0);
        assert_eq!(error.counters().masked_slice_denial_count(), 1);
        assert_eq!(
            error.diagnostic().counter_projection().label().as_str(),
            error.counters().counter_projection().label()
        );
    }
}

#[test]
fn oversized_diagnostic_preserves_the_selection_failure() {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionConstructionSource::Direct,
    );
    let original = select_query_subscription_family(live, roomy_budget()).unwrap_err();
    let source = original.diagnostic().unwrap().source_identity();
    let counters = original.counters().clone();
    let maximum =
        worth_foundational::facade::CanonicalDigestWorkBudget::standard().maximum_encoded_bytes();
    let reason = "x".repeat(maximum + 1);
    let failure = QuerySubscriptionFamilySelectionError::new(
        QuerySubscriptionFamilySelectionFailureClass::ViewFamilyLiveFamilyMismatch,
        reason.clone(),
        QuerySubscriptionDiagnosticStage::ViewMismatch,
        source,
        counters.clone(),
    );
    assert_eq!(
        failure.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::ViewFamilyLiveFamilyMismatch
    );
    assert_eq!(failure.message(), reason);
    assert_eq!(failure.counters(), &counters);
    let refusal = failure.diagnostic().unwrap_err();
    assert!(matches!(
        refusal.cause,
        worth_foundational::facade::CanonicalDigestDerivationDenial::EncodedByteLimitExceeded { .. }
    ));
}
