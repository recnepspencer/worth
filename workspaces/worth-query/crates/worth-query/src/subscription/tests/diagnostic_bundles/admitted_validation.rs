use super::world::runtime_artifacts_for;
use super::*;
use crate::live::LiveQueryFamily;

#[test]
fn admitted_bundle_rejects_trace_with_missing_terminal_certification_stage() {
    let artifacts = runtime_artifacts_for(LiveQueryFamily::Detail, None, 0);
    let support_subject = QuerySubscriptionSupportSubject::active_lifecycle(
        &artifacts.declaration,
        &artifacts.admission,
        &artifacts.active_admission,
    )
    .expect("fixture digest must fit");
    let support_evidence =
        QuerySubscriptionSupportEvidence::admission(&artifacts.declaration, &artifacts.admission)
            .unwrap();
    let (support_report, _) =
        report_query_subscription_support(support_subject, support_evidence).unwrap();
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
    let denied_trace = trace_denied_query_subscription_diagnostics(
        &selection_context,
        Some(&artifacts.declaration),
        Some(&artifacts.lowering),
        Some(&artifacts.admission),
        Some(&support_report),
        failure,
    )
    .unwrap();

    let error = bundle_admitted_query_subscription_diagnostics(
        denied_trace,
        &artifacts.selection,
        &artifacts.declaration,
        &artifacts.lowering,
        &artifacts.admission,
        &support_report,
        &artifacts.lifecycle_bundle,
        None,
        None,
        Some(&artifacts.closeout),
    )
    .unwrap_err();

    assert_eq!(
        error.error_kind(),
        &QuerySubscriptionDiagnosticBundleErrorKind::MissingRequiredStage
    );
    assert_eq!(error.counters().diagnostic_missing_stage_denial_count(), 1);
}

#[test]
fn admitted_bundle_rejects_trace_with_unclaimed_optional_continuation_stage() {
    let artifacts = runtime_artifacts_for(LiveQueryFamily::Detail, None, 1);
    let support_subject = QuerySubscriptionSupportSubject::continuation(
        &artifacts.declaration,
        &artifacts.admission,
        artifacts.continuation_report.as_ref().unwrap(),
    )
    .expect("fixture digest must fit");
    let support_evidence =
        QuerySubscriptionSupportEvidence::admission(&artifacts.declaration, &artifacts.admission)
            .unwrap();
    let (support_report, _) =
        report_query_subscription_support(support_subject, support_evidence).unwrap();
    let trace = trace_admitted_query_subscription_diagnostics(
        &artifacts.selection,
        &artifacts.declaration,
        &artifacts.lowering,
        &artifacts.admission,
        &support_report,
        &artifacts.lifecycle_bundle,
        artifacts.continuation_report.as_ref(),
        None,
        Some(&artifacts.closeout),
    )
    .unwrap();

    let error = bundle_admitted_query_subscription_diagnostics(
        trace,
        &artifacts.selection,
        &artifacts.declaration,
        &artifacts.lowering,
        &artifacts.admission,
        &support_report,
        &artifacts.lifecycle_bundle,
        None,
        None,
        Some(&artifacts.closeout),
    )
    .unwrap_err();

    assert_eq!(
        error.error_kind(),
        &QuerySubscriptionDiagnosticBundleErrorKind::MissingRequiredStage
    );
    assert!(error.message().contains("continuation trace evidence"));
}

#[test]
fn lifecycle_instance_churn_changes_trace_without_changing_family_semantic_labels() {
    let base = runtime_artifacts_for(LiveQueryFamily::Detail, None, 0);
    let continued = runtime_artifacts_for(LiveQueryFamily::Detail, None, 1);

    let base_support = report_query_subscription_support(
        QuerySubscriptionSupportSubject::active_lifecycle(
            &base.declaration,
            &base.admission,
            &base.active_admission,
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionSupportEvidence::admission(&base.declaration, &base.admission).unwrap(),
    )
    .unwrap()
    .0;
    let continued_support = report_query_subscription_support(
        QuerySubscriptionSupportSubject::continuation(
            &continued.declaration,
            &continued.admission,
            continued.continuation_report.as_ref().unwrap(),
        )
        .expect("fixture digest must fit"),
        QuerySubscriptionSupportEvidence::admission(&continued.declaration, &continued.admission)
            .unwrap(),
    )
    .unwrap()
    .0;

    let base_bundle = bundle_admitted_query_subscription_diagnostics(
        trace_admitted_query_subscription_diagnostics(
            &base.selection,
            &base.declaration,
            &base.lowering,
            &base.admission,
            &base_support,
            &base.lifecycle_bundle,
            None,
            None,
            Some(&base.closeout),
        )
        .unwrap(),
        &base.selection,
        &base.declaration,
        &base.lowering,
        &base.admission,
        &base_support,
        &base.lifecycle_bundle,
        None,
        None,
        Some(&base.closeout),
    )
    .unwrap()
    .0;
    let continued_bundle = bundle_admitted_query_subscription_diagnostics(
        trace_admitted_query_subscription_diagnostics(
            &continued.selection,
            &continued.declaration,
            &continued.lowering,
            &continued.admission,
            &continued_support,
            &continued.lifecycle_bundle,
            continued.continuation_report.as_ref(),
            None,
            Some(&continued.closeout),
        )
        .unwrap(),
        &continued.selection,
        &continued.declaration,
        &continued.lowering,
        &continued.admission,
        &continued_support,
        &continued.lifecycle_bundle,
        continued.continuation_report.as_ref(),
        None,
        Some(&continued.closeout),
    )
    .unwrap()
    .0;

    assert_eq!(
        base_bundle.semantic_labels().query_family_label(),
        continued_bundle.semantic_labels().query_family_label()
    );
    assert_eq!(
        base_bundle.semantic_labels().declaration_family_label(),
        continued_bundle
            .semantic_labels()
            .declaration_family_label()
    );
    assert_ne!(
        base_bundle.trace().trace_projection().label(),
        continued_bundle.trace().trace_projection().label()
    );
    assert_ne!(
        base_bundle.bundle_projection().label(),
        continued_bundle.bundle_projection().label()
    );
}
