use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn equivalent_future_live_meaning_selects_same_family_regardless_of_construction_source() {
    let temporal_selections = [
        QuerySubscriptionConstructionSource::Direct,
        QuerySubscriptionConstructionSource::ScopeExpanded,
        QuerySubscriptionConstructionSource::TemplateInstantiated,
        QuerySubscriptionConstructionSource::SavedExactReuse,
        QuerySubscriptionConstructionSource::FacadeLive,
    ]
    .into_iter()
    .map(|source| {
        select_query_subscription_family(
            LiveQueryAdmissionArtifact::for_test_with_future_selection(
                LiveQueryFamily::OrderedCollection,
                Some(LiveViewShapeFamily::Table),
                source,
                QuerySubscriptionFutureSelection::temporal().expect("fixture digest must fit"),
            )
            .expect("fixture digest must fit"),
            roomy_budget(),
        )
        .unwrap()
    })
    .collect::<Vec<_>>();
    let temporal_first = &temporal_selections[0];

    for selection in &temporal_selections[1..] {
        assert_eq!(temporal_first.family(), selection.family());
        assert_eq!(
            temporal_first
                .future_selection()
                .future_selection_projection()
                .label(),
            selection
                .future_selection()
                .future_selection_projection()
                .label()
        );
        assert_eq!(
            temporal_first
                .equivalence_basis()
                .equivalence_projection()
                .label(),
            selection
                .equivalence_basis()
                .equivalence_projection()
                .label()
        );
    }

    let async_selections = [
        QuerySubscriptionConstructionSource::Direct,
        QuerySubscriptionConstructionSource::ScopeExpanded,
        QuerySubscriptionConstructionSource::TemplateInstantiated,
        QuerySubscriptionConstructionSource::SavedExactReuse,
        QuerySubscriptionConstructionSource::FacadeLive,
    ]
    .into_iter()
    .map(|source| {
        select_query_subscription_family(
            LiveQueryAdmissionArtifact::for_test_with_future_selection(
                LiveQueryFamily::Detail,
                None,
                source,
                QuerySubscriptionFutureSelection::async_resource(true)
                    .expect("fixture digest must fit"),
            )
            .expect("fixture digest must fit"),
            roomy_budget(),
        )
        .unwrap()
    })
    .collect::<Vec<_>>();
    let async_first = &async_selections[0];

    for selection in &async_selections[1..] {
        assert_eq!(async_first.family(), selection.family());
        assert_eq!(
            async_first
                .future_selection()
                .future_selection_projection()
                .label(),
            selection
                .future_selection()
                .future_selection_projection()
                .label()
        );
        assert_eq!(
            async_first
                .equivalence_basis()
                .equivalence_projection()
                .label(),
            selection
                .equivalence_basis()
                .equivalence_projection()
                .label()
        );
    }
}

#[test]
fn future_live_meaning_changes_selection_and_support_story_without_forking_family_vocabulary() {
    let ordinary = select_query_subscription_family(
        LiveQueryAdmissionArtifact::for_test(
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            QuerySubscriptionConstructionSource::Direct,
        )
        .expect("fixture digest must fit"),
        roomy_budget(),
    )
    .unwrap();
    let temporal = select_query_subscription_family(
        LiveQueryAdmissionArtifact::for_test_with_future_selection(
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            QuerySubscriptionConstructionSource::Direct,
            QuerySubscriptionFutureSelection::temporal().expect("fixture digest must fit"),
        )
        .expect("fixture digest must fit"),
        roomy_budget(),
    )
    .unwrap();
    let async_live = select_query_subscription_family(
        LiveQueryAdmissionArtifact::for_test_with_future_selection(
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            QuerySubscriptionConstructionSource::Direct,
            QuerySubscriptionFutureSelection::async_resource(true)
                .expect("fixture digest must fit"),
        )
        .expect("fixture digest must fit"),
        roomy_budget(),
    )
    .unwrap();

    assert_eq!(
        ordinary.family(),
        &QuerySubscriptionFamily::CollectionMembership
    );
    assert_eq!(
        temporal.family(),
        &QuerySubscriptionFamily::CollectionMembership
    );
    assert_eq!(
        async_live.family(),
        &QuerySubscriptionFamily::CollectionMembership
    );
    assert_eq!(ordinary.future_selection().class().as_str(), "ordinary");
    assert_eq!(temporal.future_selection().class().as_str(), "temporal");
    assert_eq!(
        async_live.future_selection().class().as_str(),
        "async_resource"
    );
    assert_ne!(
        ordinary
            .equivalence_basis()
            .equivalence_projection()
            .label(),
        temporal
            .equivalence_basis()
            .equivalence_projection()
            .label()
    );
    assert_ne!(
        temporal
            .equivalence_basis()
            .equivalence_projection()
            .label(),
        async_live
            .equivalence_basis()
            .equivalence_projection()
            .label()
    );

    let temporal_declaration =
        declare_query_subscription(temporal.clone(), roomy_slice_budget()).unwrap();
    let temporal_support = report_query_subscription_support(
        QuerySubscriptionSupportSubject::declaration(&temporal_declaration)
            .expect("fixture digest must fit"),
        QuerySubscriptionSupportEvidence::declaration(&temporal_declaration),
    )
    .unwrap()
    .0;
    let async_declaration = declare_query_subscription(async_live, roomy_slice_budget()).unwrap();
    let async_support = report_query_subscription_support(
        QuerySubscriptionSupportSubject::declaration(&async_declaration)
            .expect("fixture digest must fit"),
        QuerySubscriptionSupportEvidence::declaration(&async_declaration),
    )
    .unwrap()
    .0;

    assert_eq!(
        temporal_declaration.future_selection().class().as_str(),
        temporal_support
            .support_subject()
            .future_selection()
            .class()
            .as_str()
    );
    assert_eq!(
        async_declaration.future_selection().class().as_str(),
        async_support
            .support_subject()
            .future_selection()
            .class()
            .as_str()
    );
    assert_ne!(
        temporal_declaration.declaration_projection().label(),
        async_declaration.declaration_projection().label()
    );
}

#[test]
fn future_live_shape_and_basis_mismatches_deny_during_family_selection() {
    let temporal_inspector = select_query_subscription_family(
        LiveQueryAdmissionArtifact::for_test_with_future_selection(
            LiveQueryFamily::Detail,
            Some(LiveViewShapeFamily::InspectorDetailFocused),
            QuerySubscriptionConstructionSource::Direct,
            QuerySubscriptionFutureSelection::temporal().expect("fixture digest must fit"),
        )
        .expect("fixture digest must fit"),
        roomy_budget(),
    )
    .unwrap_err();
    assert_eq!(
        temporal_inspector.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::UnsupportedTemporalLiveShape
    );
    assert_eq!(
        temporal_inspector
            .diagnostic()
            .expect("fixture digest must fit")
            .stage(),
        &QuerySubscriptionDiagnosticStage::ViewMismatch
    );

    let async_grouped = select_query_subscription_family(
        LiveQueryAdmissionArtifact::for_test_with_future_selection(
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::KanbanGrouped),
            QuerySubscriptionConstructionSource::Direct,
            QuerySubscriptionFutureSelection::async_resource(true)
                .expect("fixture digest must fit"),
        )
        .expect("fixture digest must fit"),
        roomy_budget(),
    )
    .unwrap_err();
    assert_eq!(
        async_grouped.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::UnsupportedAsyncLiveShape
    );
    assert_eq!(
        async_grouped
            .diagnostic()
            .expect("fixture digest must fit")
            .stage(),
        &QuerySubscriptionDiagnosticStage::ViewMismatch
    );

    let temporal_historical = select_query_subscription_family(
        LiveQueryAdmissionArtifact::for_test_with_context(
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            QuerySubscriptionConstructionSource::Direct,
            QuerySubscriptionBasisPosture::RuntimeHistoricalSnapshot,
            QuerySubscriptionFutureSelection::temporal().expect("fixture digest must fit"),
            Some("policy".to_string()),
            Some("tenant".to_string()),
            Some("relationship-proof".to_string()),
            QuerySubscriptionRelationshipProofPosture::Admitted,
        )
        .expect("fixture digest must fit"),
        roomy_budget(),
    )
    .unwrap_err();
    assert_eq!(
        temporal_historical.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::FutureLiveBasisUnsupported
    );
    assert_eq!(
        temporal_historical
            .diagnostic()
            .expect("fixture digest must fit")
            .stage(),
        &QuerySubscriptionDiagnosticStage::FamilySelection
    );
}

#[test]
fn future_live_meaning_still_honors_explicit_budget_denials() {
    let error = select_query_subscription_family(
        LiveQueryAdmissionArtifact::for_test_with_future_selection(
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            QuerySubscriptionConstructionSource::Direct,
            QuerySubscriptionFutureSelection::temporal_async(true)
                .expect("fixture digest must fit"),
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionWorkBudget::scratch_buffer_only(8, 8, 8, 32, 0),
    )
    .unwrap_err();

    assert_eq!(
        error.failure_class(),
        &QuerySubscriptionFamilySelectionFailureClass::WorkBudgetExceeded
    );
    assert_eq!(
        error.diagnostic().expect("fixture digest must fit").stage(),
        &QuerySubscriptionDiagnosticStage::FamilySelection
    );
    assert_eq!(error.counters().family_selection_count(), 0);
    assert_eq!(error.counters().family_denial_count(), 1);
    assert_eq!(error.counters().work_budget_denial_count(), 1);
}
