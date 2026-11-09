use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn manual_bridge_witness_and_parity_explanation_bind_canonical_bridge_artifacts() {
    let artifacts = parity_artifacts_for(LiveQueryFamily::Detail, None);

    let witness = build_query_subscription_manual_bridge_witness(
        &artifacts.declaration,
        &artifacts.lowering,
        &artifacts.activation,
    )
    .unwrap();
    let (explanation, receipt) = explain_query_subscription_bridge_parity(
        &artifacts.declaration,
        &artifacts.lowering,
        &artifacts.activation,
        witness.clone(),
    )
    .unwrap();

    assert_eq!(witness.query_family_label(), "detail_exact");
    assert_eq!(witness.bridge_family_label(), "detail_exact");
    assert_eq!(
        receipt.witness_assembly_posture(),
        &BridgeWitnessAssemblyPosture::PreLoweredWitness
    );
    assert_eq!(
        receipt.parity_class(),
        &QuerySubscriptionBridgeParityClass::ExactParity
    );
    assert_eq!(receipt.semantic_rebuild_count(), 0);
    assert_eq!(
        receipt.comparison_width().compared_slice_dimension_count(),
        2
    );
    assert_eq!(explanation.query_family_label(), "detail_exact");
    assert_eq!(explanation.bridge_family_label(), "detail_exact");
    assert_eq!(
        explanation.signal_strategy_class_label(),
        "exact_detail_signals"
    );
    assert_eq!(
        explanation
            .comparison()
            .query_declaration_projection()
            .label(),
        artifacts.declaration.declaration_projection().label()
    );
    assert_eq!(
        explanation
            .comparison()
            .bridge_declaration_projection()
            .label(),
        artifacts.lowering.bridge_declaration_projection().label()
    );
    assert_eq!(
        witness.query_declaration_identity(),
        artifacts.declaration.declaration_identity()
    );
    assert_eq!(
        witness.bridge_declaration_identity(),
        artifacts.lowering.bridge_declaration_identity()
    );
    assert_eq!(
        witness.basis_binding_identity(),
        artifacts.lowering.basis_request().evidence_identity()
    );
    assert_eq!(
        witness.signal_strategy_identity(),
        artifacts
            .lowering
            .signal_strategy_request()
            .evidence_identity()
    );
    assert_eq!(
        witness.activation_identity(),
        artifacts.activation.evidence_identity()
    );
    assert_eq!(
        explanation
            .counters()
            .subscription_bridge_parity_comparison_count(),
        1
    );
    assert_eq!(
        explanation
            .counters()
            .subscription_bridge_parity_admitted_count(),
        1
    );
    assert_eq!(
        explanation
            .counters()
            .subscription_bridge_parity_denial_count(),
        0
    );
    assert!(!explanation.explanation_projection().label().is_empty());
}

#[test]
fn grouped_and_inspector_families_preserve_query_distinction_even_with_shared_bridge_family() {
    let grouped = parity_artifacts_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::KanbanGrouped),
    );
    let inspector = parity_artifacts_for(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::InspectorDetailFocused),
    );

    let grouped_explanation = explain_query_subscription_bridge_parity(
        &grouped.declaration,
        &grouped.lowering,
        &grouped.activation,
        build_query_subscription_manual_bridge_witness(
            &grouped.declaration,
            &grouped.lowering,
            &grouped.activation,
        )
        .unwrap(),
    )
    .unwrap()
    .0;
    let inspector_explanation = explain_query_subscription_bridge_parity(
        &inspector.declaration,
        &inspector.lowering,
        &inspector.activation,
        build_query_subscription_manual_bridge_witness(
            &inspector.declaration,
            &inspector.lowering,
            &inspector.activation,
        )
        .unwrap(),
    )
    .unwrap()
    .0;

    assert_eq!(
        grouped_explanation.comparison().parity_class(),
        &QuerySubscriptionBridgeParityClass::FamilyDistinctBridgeShared
    );
    assert_eq!(
        inspector_explanation.comparison().parity_class(),
        &QuerySubscriptionBridgeParityClass::FamilyDistinctBridgeShared
    );
    assert_eq!(
        grouped_explanation.query_family_label(),
        "grouped_collection_membership"
    );
    assert_eq!(
        inspector_explanation.query_family_label(),
        "inspector_detail_exact"
    );
    assert_eq!(
        grouped_explanation
            .counters()
            .subscription_bridge_family_distinction_preservation_count(),
        1
    );
    assert_eq!(
        inspector_explanation
            .counters()
            .subscription_bridge_family_distinction_preservation_count(),
        1
    );
}

#[test]
fn bridge_parity_denies_typed_on_mismatched_declaration_sources() {
    let detail = parity_artifacts_for(LiveQueryFamily::Detail, None);
    let collection = parity_artifacts_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let witness = build_query_subscription_manual_bridge_witness(
        &detail.declaration,
        &detail.lowering,
        &detail.activation,
    )
    .unwrap();

    let error = explain_query_subscription_bridge_parity(
        &collection.declaration,
        &collection.lowering,
        &collection.activation,
        witness,
    )
    .unwrap_err();

    assert_eq!(
        error.failure().failure_kind(),
        &QuerySubscriptionBridgeParityFailureKind::DeclarationMismatch
    );
    assert_eq!(
        error.failure().parity_class(),
        &QuerySubscriptionBridgeParityClass::DeniedSourceMismatch
    );
    assert_eq!(
        error.counters().subscription_bridge_parity_denial_count(),
        1
    );
    assert_eq!(
        error.counters().subscription_bridge_parity_admitted_count(),
        0
    );
}

#[test]
fn bridge_parity_denies_when_witness_and_activation_do_not_share_runtime_activation_identity() {
    let first = parity_artifacts_for_with_budget(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionAdmissionBudget::admitted(1, 8, 1, 1, 1),
    );
    let second = parity_artifacts_for_with_budget(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionAdmissionBudget::admitted(2, 8, 1, 1, 1),
    );
    let witness = build_query_subscription_manual_bridge_witness(
        &first.declaration,
        &first.lowering,
        &first.activation,
    )
    .unwrap();

    let error = explain_query_subscription_bridge_parity(
        &second.declaration,
        &second.lowering,
        &second.activation,
        witness,
    )
    .unwrap_err();

    assert_eq!(
        error.failure().failure_kind(),
        &QuerySubscriptionBridgeParityFailureKind::ActivationMismatch
    );
    assert_eq!(
        error.failure().parity_class(),
        &QuerySubscriptionBridgeParityClass::DeniedSourceMismatch
    );
    assert_eq!(
        error.counters().subscription_bridge_parity_denial_count(),
        1
    );
}

struct BridgeParityArtifacts {
    declaration: QuerySubscriptionDeclarationArtifact,
    lowering: BridgeSubscriptionLoweringPlan,
    activation: SubscriptionActivationInput,
}

fn parity_artifacts_for(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
) -> BridgeParityArtifacts {
    parity_artifacts_for_with_budget(
        live_family,
        view_family,
        QuerySubscriptionAdmissionBudget::admitted(1, 8, 1, 1, 1),
    )
}

fn parity_artifacts_for_with_budget(
    live_family: LiveQueryFamily,
    view_family: Option<LiveViewShapeFamily>,
    admission_budget: QuerySubscriptionAdmissionBudget,
) -> BridgeParityArtifacts {
    let live = LiveQueryAdmissionArtifact::for_test(
        live_family,
        view_family,
        QuerySubscriptionConstructionSource::Direct,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(live, roomy_budget()).unwrap();
    let declaration = declare_query_subscription(selection, roomy_slice_budget()).unwrap();
    let lowering =
        lower_query_subscription_to_bridge(declaration.clone(), roomy_lowering_budget()).unwrap();
    let admission = admit_query_subscription(lowering.clone(), admission_budget).unwrap();
    let activation = prepare_subscription_activation(admission);

    BridgeParityArtifacts {
        declaration,
        lowering,
        activation,
    }
}
