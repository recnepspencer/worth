use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

use super::world::table_declaration;
use super::*;

#[test]
fn delivery_intent_denial_has_specific_diagnostic_stage() {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionConstructionSource::Direct,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(live, roomy_budget()).unwrap();

    let error = declare_query_subscription(
        selection,
        roomy_slice_budget().without_delivery_intent_support(),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &QuerySubscriptionDeclarationDenialKind::DeliveryIntentUnsupported
    );
    assert_eq!(
        error.diagnostic().expect("fixture digest must fit").stage(),
        &QuerySubscriptionDiagnosticStage::DeliveryIntent
    );
    assert_eq!(error.counters().delivery_intent_denial_count(), 1);
    assert_eq!(error.counters().declaration_denial_count(), 1);
    assert_eq!(error.counters().declaration_count(), 0);
    assert_eq!(error.counters().bridge_lowering_count(), 0);
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
fn bridge_family_slice_and_basis_denials_have_distinct_diagnostic_stages() {
    let table_declaration = table_declaration();
    let family_error = lower_query_subscription_to_bridge(
        table_declaration.clone(),
        roomy_lowering_budget().without_bridge_family_support(),
    )
    .unwrap_err();
    assert_eq!(
        family_error
            .diagnostic()
            .expect("fixture digest must fit")
            .stage(),
        &QuerySubscriptionDiagnosticStage::BridgeFamilyLowering
    );
    assert_eq!(
        family_error.denial_kind(),
        &QuerySubscriptionBridgeLoweringDenialKind::BridgeFamilyUnsupported
    );
    assert_eq!(family_error.counters().bridge_lowering_count(), 0);
    assert_eq!(family_error.counters().bridge_family_denial_count(), 1);
    assert_eq!(
        family_error
            .diagnostic()
            .expect("fixture digest must fit")
            .counter_projection()
            .label()
            .as_str(),
        family_error.counters().counter_projection().label()
    );

    let slice_error = lower_query_subscription_to_bridge(
        table_declaration.clone(),
        roomy_lowering_budget().without_bridge_slice_support(BridgeSubscriptionSliceKind::Ordering),
    )
    .unwrap_err();
    assert_eq!(
        slice_error
            .diagnostic()
            .expect("fixture digest must fit")
            .stage(),
        &QuerySubscriptionDiagnosticStage::BridgeSliceLowering
    );
    assert_eq!(
        slice_error.denial_kind(),
        &QuerySubscriptionBridgeLoweringDenialKind::BridgeSliceUnsupported
    );
    assert_eq!(slice_error.counters().bridge_lowering_count(), 0);
    assert_eq!(slice_error.counters().bridge_slice_denial_count(), 1);
    assert_eq!(
        slice_error
            .diagnostic()
            .expect("fixture digest must fit")
            .counter_projection()
            .label()
            .as_str(),
        slice_error.counters().counter_projection().label()
    );

    let live = LiveQueryAdmissionArtifact::for_test_with_basis(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionConstructionSource::Direct,
        QuerySubscriptionBasisPosture::RuntimeHistoricalSnapshot,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(live, roomy_budget()).unwrap();
    let declaration = declare_query_subscription(selection, roomy_slice_budget()).unwrap();
    let basis_error = lower_query_subscription_to_bridge(
        declaration,
        roomy_lowering_budget().without_historical_basis_support(),
    )
    .unwrap_err();
    assert_eq!(
        basis_error
            .diagnostic()
            .expect("fixture digest must fit")
            .stage(),
        &QuerySubscriptionDiagnosticStage::BasisBinding
    );
    assert_eq!(
        basis_error.denial_kind(),
        &QuerySubscriptionBridgeLoweringDenialKind::BasisBindingUnsupported
    );
    assert_eq!(basis_error.counters().bridge_lowering_count(), 0);
    assert_eq!(basis_error.counters().basis_binding_denial_count(), 1);
    assert_eq!(
        basis_error
            .diagnostic()
            .expect("fixture digest must fit")
            .counter_projection()
            .label()
            .as_str(),
        basis_error.counters().counter_projection().label()
    );
}
