use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn denied_diagnostic_bundle_localizes_declaration_failure_and_omits_later_stages() {
    let live = LiveQueryAdmissionArtifact::for_test(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::KanbanGrouped),
        QuerySubscriptionConstructionSource::Direct,
    )
    .expect("fixture digest must fit");
    let selection = select_query_subscription_family(live, roomy_budget()).unwrap();
    let declaration_error = declare_query_subscription(
        selection.clone(),
        roomy_slice_budget().with_masked_slice_request_detected(),
    )
    .unwrap_err();
    let failure = QuerySubscriptionDiagnosticFailure::from_declaration_denial(&declaration_error)
        .expect("fixture digest must fit");
    let selection_context = QuerySubscriptionDiagnosticSelectionContext::from_selection(&selection);
    let trace = trace_denied_query_subscription_diagnostics(
        &selection_context,
        None,
        None,
        None,
        None,
        failure.clone().expect("fixture digest must fit"),
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
        &QuerySubscriptionDiagnosticStage::Declaration
    );
    assert_eq!(
        bundle.semantic_labels().query_family_label(),
        selection.family().as_str()
    );
    assert_eq!(
        bundle.semantic_labels().live_graph_access_posture_label(),
        selection.live_graph_access_posture().as_str()
    );
    assert_eq!(bundle.support_report_projection(), None);
    assert_eq!(
        bundle.omitted_stages(),
        &[
            QuerySubscriptionDiagnosticStage::BridgeFamilyLowering,
            QuerySubscriptionDiagnosticStage::RuntimeBackedAdmission,
            QuerySubscriptionDiagnosticStage::SupportReporting,
            QuerySubscriptionDiagnosticStage::Certification,
        ]
    );
    assert_eq!(receipt.bundle_width().failure_evidence_count(), 1);
    assert_eq!(bundle.counters().denied_bundle_emission_count(), 1);
    assert!(!bundle.bundle_projection().label().is_empty());
}
