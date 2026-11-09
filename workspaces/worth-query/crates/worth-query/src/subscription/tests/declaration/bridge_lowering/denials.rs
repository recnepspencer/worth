use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn unsupported_bridge_family_slice_and_basis_deny_before_lowering_plan_exists() {
    let declaration = declaration_for(LiveQueryFamily::BoundedMaterialization, None);
    let family_error = lower_query_subscription_to_bridge(
        declaration.clone(),
        roomy_lowering_budget().without_bridge_family_support(),
    )
    .unwrap_err();
    assert_eq!(
        family_error.denial_kind(),
        &QuerySubscriptionBridgeLoweringDenialKind::BridgeFamilyUnsupported
    );
    assert_eq!(family_error.counters().bridge_lowering_count(), 0);
    assert_eq!(family_error.counters().bridge_family_denial_count(), 1);

    let slice_error = lower_query_subscription_to_bridge(
        declaration.clone(),
        roomy_lowering_budget()
            .without_bridge_slice_support(BridgeSubscriptionSliceKind::RelationScope),
    )
    .unwrap_err();
    assert_eq!(
        slice_error.denial_kind(),
        &QuerySubscriptionBridgeLoweringDenialKind::BridgeSliceUnsupported
    );
    assert_eq!(slice_error.counters().bridge_lowering_count(), 0);
    assert_eq!(slice_error.counters().bridge_slice_denial_count(), 1);

    let historical_input = LiveQueryAdmissionArtifact::for_test_with_basis(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::FacadeLive,
        QuerySubscriptionBasisPosture::RuntimeHistoricalSnapshot,
    )
    .expect("fixture digest must fit");
    let historical_selection =
        select_query_subscription_family(historical_input, roomy_budget()).unwrap();
    let historical_declaration =
        declare_query_subscription(historical_selection, roomy_slice_budget()).unwrap();
    let basis_error = lower_query_subscription_to_bridge(
        historical_declaration,
        roomy_lowering_budget().without_historical_basis_support(),
    )
    .unwrap_err();
    assert_eq!(
        basis_error.denial_kind(),
        &QuerySubscriptionBridgeLoweringDenialKind::BasisBindingUnsupported
    );
    assert_eq!(basis_error.counters().basis_binding_denial_count(), 1);

    let preview_declaration = declaration_for_basis(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionBasisPosture::PreviewScoped,
    );
    let preview_error =
        lower_query_subscription_to_bridge(preview_declaration, roomy_lowering_budget())
            .unwrap_err();
    assert_eq!(
        preview_error.denial_kind(),
        &QuerySubscriptionBridgeLoweringDenialKind::BasisBindingUnsupported
    );
    assert_eq!(preview_error.counters().basis_binding_denial_count(), 1);

    let denied_basis_declaration = declaration_for_basis(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionBasisPosture::DeniedUnsupportedBasis,
    );
    let denied_basis_error = lower_query_subscription_to_bridge(
        denied_basis_declaration,
        roomy_lowering_budget().with_preview_basis_support(),
    )
    .unwrap_err();
    assert_eq!(
        denied_basis_error.denial_kind(),
        &QuerySubscriptionBridgeLoweringDenialKind::BasisBindingUnsupported
    );
    assert_eq!(
        denied_basis_error.counters().basis_binding_denial_count(),
        1
    );
}

#[test]
fn bridge_lowering_budget_exhaustion_denies_before_plan_exists() {
    let declaration = declaration_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let error = lower_query_subscription_to_bridge(
        declaration,
        QuerySubscriptionBridgeLoweringBudget::admitted(1, 1, 8, 1, 1),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionBridgeLoweringDenialKind::LoweringBudgetExceeded
    );
    assert_eq!(error.counters().bridge_lowering_count(), 0);
    assert_eq!(error.counters().work_budget_denial_count(), 1);
}

#[test]
fn bridge_fallback_lowering_is_explicitly_budget_gated() {
    let declaration = declaration_for(LiveQueryFamily::Detail, None)
        .with_bridge_posture(QuerySubscriptionBridgePosture::BridgeLoweringDeferred);

    let error = lower_query_subscription_to_bridge(declaration.clone(), roomy_lowering_budget())
        .unwrap_err();
    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionBridgeLoweringDenialKind::BridgeFallbackUnsupported
    );
    assert_eq!(error.counters().bridge_lowering_count(), 0);
    assert_eq!(error.counters().bridge_fallback_denial_count(), 1);

    let admitted = lower_query_subscription_to_bridge(
        declaration,
        roomy_lowering_budget().with_bridge_fallback_support(),
    )
    .unwrap();
    assert_eq!(admitted.counters().bridge_lowering_count(), 1);
}
