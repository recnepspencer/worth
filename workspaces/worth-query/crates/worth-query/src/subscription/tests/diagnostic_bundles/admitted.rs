use super::world::{runtime_artifacts_for, runtime_artifacts_for_with_basis};
use super::*;
use crate::live::LiveQueryFamily;
use crate::subscription::posture::QuerySubscriptionBasisPosture;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn admitted_diagnostic_bundle_carries_offline_semantic_labels_and_canonical_digests() {
    let artifacts = runtime_artifacts_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        0,
    );
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

    let (bundle, receipt) = bundle_admitted_query_subscription_diagnostics(
        trace,
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

    assert_eq!(
        bundle.semantic_labels().query_family_label(),
        artifacts.selection.family().as_str()
    );
    assert_eq!(
        bundle.semantic_labels().declaration_family_label(),
        artifacts.declaration.family().as_str()
    );
    assert_eq!(
        bundle.semantic_labels().bridge_family_label(),
        artifacts.lowering.bridge_family().as_str()
    );
    assert_eq!(
        bundle.semantic_labels().basis_posture_label(),
        artifacts.lowering.basis_request().request_kind().as_str()
    );
    assert_eq!(
        bundle.semantic_labels().signal_strategy_class_label(),
        artifacts
            .lowering
            .signal_strategy_request()
            .request_kind()
            .as_str()
    );
    assert_eq!(
        bundle.semantic_labels().live_graph_access_posture_label(),
        artifacts.declaration.live_graph_access_posture().as_str()
    );
    assert_eq!(
        bundle.semantic_labels().support_posture_label(),
        support_report.support_posture().as_str()
    );
    assert_eq!(
        bundle.semantic_labels().denial_or_coverage_class_label(),
        "runtime_lifecycle_certified"
    );
    assert_eq!(
        bundle.support_report_projection().label(),
        support_report.report_projection().label()
    );
    assert_eq!(
        bundle.lifecycle_certification_projection().label(),
        artifacts
            .lifecycle_bundle
            .certification_bundle_projection()
            .label()
    );
    assert_eq!(
        bundle
            .lifecycle_closeout_projection()
            .map(|projection| projection.label().to_string()),
        Some(artifacts.closeout.closeout_projection().label().to_string())
    );
    assert_eq!(
        receipt.bundle_assembly_posture(),
        &BundleAssemblyPosture::ComposedFromCanonicalArtifacts
    );
    assert_eq!(receipt.stage_rederivation_count(), 0);
    assert_eq!(bundle.counters().diagnostic_bundle_emission_count(), 1);
    assert_eq!(bundle.counters().denied_bundle_emission_count(), 0);
    assert!(!bundle.bundle_projection().label().is_empty());
}

#[test]
fn admitted_diagnostic_bundle_preserves_canonical_basis_posture_labels() {
    let artifacts = runtime_artifacts_for_with_basis(
        LiveQueryFamily::Detail,
        None,
        0,
        QuerySubscriptionBasisPosture::RuntimeHistoricalSnapshot,
    );
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

    let (bundle, _) = bundle_admitted_query_subscription_diagnostics(
        trace,
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

    assert_eq!(
        bundle.semantic_labels().basis_posture_label(),
        QuerySubscriptionBasisPosture::RuntimeHistoricalSnapshot.as_str()
    );
}
