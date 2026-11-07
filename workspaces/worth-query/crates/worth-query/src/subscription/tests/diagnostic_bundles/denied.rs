use super::world::runtime_artifacts_for;
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn selection_denied_diagnostic_bundle_localizes_family_selection_failure_and_omits_later_stages() {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::KanbanGrouped),
        QuerySubscriptionConstructionSource::Direct,
    )
    .expect("fixture digest must fit");
    let selection_error =
        select_query_subscription_family(live.clone(), roomy_budget()).unwrap_err();
    let failure = QuerySubscriptionDiagnosticFailure::from_family_selection_error(&selection_error)
        .expect("fixture digest must fit");
    let selection_context =
        QuerySubscriptionDiagnosticSelectionContext::from_selection_denial(&live, &selection_error)
            .expect("fixture digest must fit");
    let trace = trace_denied_query_subscription_diagnostics(
        &selection_context,
        None,
        None,
        None,
        None,
        failure.clone(),
    )
    .unwrap();

    let (bundle, receipt) = bundle_denied_query_subscription_diagnostics(
        trace,
        &selection_context,
        None,
        None,
        None,
        None,
        failure,
    )
    .unwrap();

    assert_eq!(
        bundle.failure().stage(),
        &QuerySubscriptionDiagnosticStage::ViewMismatch
    );
    assert_eq!(
        bundle.semantic_labels().query_family_label(),
        "selection_unresolved:detail:kanban_grouped"
    );
    assert_eq!(
        bundle.semantic_labels().declaration_family_label(),
        "not_declared:selection_unresolved:detail:kanban_grouped"
    );
    assert_eq!(
        bundle.semantic_labels().basis_posture_label(),
        "current_head"
    );
    assert_eq!(
        bundle.semantic_labels().live_graph_access_posture_label(),
        "selection_denied"
    );
    assert_eq!(bundle.support_report_projection(), None);
    assert_eq!(
        bundle.omitted_stages(),
        &[
            QuerySubscriptionDiagnosticStage::Declaration,
            QuerySubscriptionDiagnosticStage::BridgeFamilyLowering,
            QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission,
            QuerySubscriptionDiagnosticStage::SupportReporting,
            QuerySubscriptionDiagnosticStage::Certification,
        ]
    );
    assert_eq!(receipt.stage_rederivation_count(), 0);
    assert_eq!(bundle.counters().denied_bundle_emission_count(), 1);
}

#[test]
fn denied_bundle_rejects_trace_that_claims_runtime_admission_without_admission_artifact() {
    let artifacts = runtime_artifacts_for(LiveQueryFamily::Detail, None, 0);
    let failure = QuerySubscriptionDiagnosticFailure::from_support_report_error(
        &report_query_subscription_support(
            QuerySubscriptionSupportSubject::activation(
                &artifacts.declaration,
                &prepare_subscription_activation(artifacts.admission.clone()),
            )
            .expect("fixture digest must fit"),
            QuerySubscriptionSupportEvidence::declaration(&artifacts.declaration),
        )
        .unwrap_err(),
    )
    .expect("fixture digest must fit");
    let selection_context =
        QuerySubscriptionDiagnosticSelectionContext::from_selection(&artifacts.selection);
    let trace = trace_denied_query_subscription_diagnostics(
        &selection_context,
        Some(&artifacts.declaration),
        Some(&artifacts.lowering),
        Some(&artifacts.admission),
        None,
        failure.clone(),
    )
    .unwrap();

    let error = bundle_denied_query_subscription_diagnostics(
        trace,
        &selection_context,
        Some(&artifacts.declaration),
        Some(&artifacts.lowering),
        None,
        None,
        failure,
    )
    .unwrap_err();

    assert_eq!(
        error.error_kind(),
        &QuerySubscriptionDiagnosticBundleErrorKind::MissingRequiredStage
    );
    assert!(error
        .message()
        .contains("trace to carry every stage that the assembled artifacts claim"));
}

#[test]
fn selection_denied_trace_rejects_mismatched_failure_source() {
    let first_live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::KanbanGrouped),
        QuerySubscriptionConstructionSource::Direct,
    )
    .expect("fixture digest must fit");
    let second_live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::BoundedMaterialization,
        Some(LiveViewShapeFamily::InspectorDetailFocused),
        QuerySubscriptionConstructionSource::Direct,
    );
    let first_error =
        select_query_subscription_family(first_live.clone(), roomy_budget()).unwrap_err();
    let second_error = select_query_subscription_family(
        second_live.clone().expect("fixture digest must fit"),
        roomy_budget(),
    )
    .unwrap_err();
    let selection_context = QuerySubscriptionDiagnosticSelectionContext::from_selection_denial(
        &first_live,
        &first_error,
    )
    .expect("fixture digest must fit");
    let failure = QuerySubscriptionDiagnosticFailure::from_family_selection_error(&second_error)
        .expect("fixture digest must fit");

    let error = trace_denied_query_subscription_diagnostics(
        &selection_context,
        None,
        None,
        None,
        None,
        failure,
    )
    .unwrap_err();

    assert_eq!(
        error.error_kind(),
        &QuerySubscriptionDiagnosticBundleErrorKind::SelectionContextMismatch
    );
    assert_eq!(error.counters().diagnostic_missing_stage_denial_count(), 1);
}

#[test]
fn denied_trace_preserves_runtime_backed_admission_stage_before_support_failure() {
    let artifacts = runtime_artifacts_for(LiveQueryFamily::Detail, None, 0);
    let failure = QuerySubscriptionDiagnosticFailure::from_support_report_error(
        &report_query_subscription_support(
            QuerySubscriptionSupportSubject::activation(
                &artifacts.declaration,
                &prepare_subscription_activation(artifacts.admission.clone()),
            )
            .expect("fixture digest must fit"),
            QuerySubscriptionSupportEvidence::declaration(&artifacts.declaration),
        )
        .unwrap_err(),
    )
    .expect("fixture digest must fit");
    let selection_context =
        QuerySubscriptionDiagnosticSelectionContext::from_selection(&artifacts.selection);

    let trace = trace_denied_query_subscription_diagnostics(
        &selection_context,
        Some(&artifacts.declaration),
        Some(&artifacts.lowering),
        Some(&artifacts.admission),
        None,
        failure,
    )
    .unwrap();

    assert_eq!(
        trace
            .stage_traces()
            .iter()
            .map(|trace| (trace.stage(), trace.outcome()))
            .collect::<Vec<_>>(),
        vec![
            (
                &QuerySubscriptionDiagnosticStage::FamilySelection,
                &QuerySubscriptionDiagnosticOutcome::Admitted,
            ),
            (
                &QuerySubscriptionDiagnosticStage::Declaration,
                &QuerySubscriptionDiagnosticOutcome::Admitted,
            ),
            (
                &QuerySubscriptionDiagnosticStage::BridgeFamilyLowering,
                &QuerySubscriptionDiagnosticOutcome::Admitted,
            ),
            (
                &QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission,
                &QuerySubscriptionDiagnosticOutcome::Admitted,
            ),
            (
                &QuerySubscriptionDiagnosticStage::SupportReporting,
                &QuerySubscriptionDiagnosticOutcome::Denied,
            ),
        ]
    );
}
