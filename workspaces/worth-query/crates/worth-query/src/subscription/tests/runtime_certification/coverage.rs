use super::world::{hostile_support_failure_for, runtime_certification_artifacts_for};
use super::*;
use crate::live::LiveQueryFamily;

#[test]
fn runtime_family_certification_closes_with_admitted_and_hostile_family_coverage() {
    let artifacts = runtime_certification_artifacts_for(LiveQueryFamily::Detail, None);
    let hostile = hostile_support_failure_for(&artifacts);

    let admitted_row = QuerySubscriptionFamilyCoverageRow::admitted(
        artifacts.declaration.family(),
        &artifacts.support_report,
        &artifacts.parity_explanation,
        &artifacts.lifecycle_bundle,
        &artifacts.admitted_bundle,
        QuerySubscriptionLifecycleCoverageClass::LifecycleCloseout,
    )
    .unwrap();
    let hostile_row = QuerySubscriptionFamilyCoverageRow::hostile(
        artifacts.declaration.family(),
        &artifacts.support_report,
        &artifacts.parity_explanation,
        &artifacts.lifecycle_bundle,
        &hostile.denied_bundle,
        &hostile.failure,
        QuerySubscriptionLifecycleCoverageClass::ActiveLifecycle,
    )
    .unwrap();
    let matrix = build_query_subscription_family_coverage_matrix(vec![admitted_row, hostile_row])
        .expect("fixture digest must fit");
    let handle = build_certified_family_coverage_handle(
        &matrix,
        artifacts.declaration.family(),
        CoverageResolutionPosture::IndexedCoverageSet,
    )
    .unwrap();
    let support_report_digest = artifacts
        .support_report
        .report_projection()
        .label()
        .to_string();
    let bridge_parity_digest = artifacts
        .parity_explanation
        .explanation_projection()
        .label()
        .to_string();
    let diagnostic_bundle_digest = artifacts
        .admitted_bundle
        .bundle_projection()
        .label()
        .to_string();
    let lifecycle_certification_digest = artifacts
        .lifecycle_bundle
        .certification_bundle_projection()
        .label()
        .to_string();
    let scope = build_query_subscription_runtime_certification_scope(
        artifacts.support_report,
        artifacts.parity_explanation,
        artifacts.admitted_bundle,
        artifacts.lifecycle_bundle,
        handle,
    )
    .unwrap();

    let (bundle, receipt) = certify_query_subscription_runtime_family(scope).unwrap();

    assert_eq!(
        receipt.coverage_resolution_posture(),
        &CoverageResolutionPosture::IndexedCoverageSet
    );
    assert_eq!(receipt.family_coverage_index_lookup_count(), 1);
    assert_eq!(receipt.covered_row_width().admitted_row_count(), 1);
    assert_eq!(receipt.covered_row_width().hostile_row_count(), 1);
    assert_eq!(receipt.uncovered_variation_width(), 0);
    assert_eq!(bundle.counters().certified_family_count(), 1);
    assert_eq!(bundle.counters().hostile_row_coverage_count(), 1);
    assert_eq!(
        bundle.support_report_projection().label().as_str(),
        support_report_digest.as_str()
    );
    assert_eq!(
        bundle.bridge_parity_projection().label().as_str(),
        bridge_parity_digest.as_str()
    );
    assert_eq!(
        bundle.diagnostic_bundle_projection().label().as_str(),
        diagnostic_bundle_digest.as_str()
    );
    assert_eq!(
        bundle.lifecycle_certification_projection().label().as_str(),
        lifecycle_certification_digest.as_str()
    );
}

#[test]
fn runtime_family_certification_denies_without_hostile_family_coverage() {
    let artifacts = runtime_certification_artifacts_for(LiveQueryFamily::Detail, None);
    let admitted_row = QuerySubscriptionFamilyCoverageRow::admitted(
        artifacts.declaration.family(),
        &artifacts.support_report,
        &artifacts.parity_explanation,
        &artifacts.lifecycle_bundle,
        &artifacts.admitted_bundle,
        QuerySubscriptionLifecycleCoverageClass::LifecycleCloseout,
    )
    .unwrap();
    let matrix = build_query_subscription_family_coverage_matrix(vec![admitted_row])
        .expect("fixture digest must fit");
    let handle = build_certified_family_coverage_handle(
        &matrix,
        artifacts.declaration.family(),
        CoverageResolutionPosture::IndexedCoverageSet,
    )
    .unwrap();
    let scope = build_query_subscription_runtime_certification_scope(
        artifacts.support_report,
        artifacts.parity_explanation,
        artifacts.admitted_bundle,
        artifacts.lifecycle_bundle,
        handle,
    )
    .unwrap();

    let error = certify_query_subscription_runtime_family(scope).unwrap_err();

    assert_eq!(
        error.error_kind(),
        &QuerySubscriptionRuntimeCertificationErrorKind::MissingHostileCoverage
    );
    assert_eq!(error.counters().uncovered_family_denial_count(), 1);
}

#[test]
fn runtime_family_coverage_handle_rejects_denied_matrix_scan_posture() {
    let artifacts = runtime_certification_artifacts_for(LiveQueryFamily::Detail, None);
    let admitted_row = QuerySubscriptionFamilyCoverageRow::admitted(
        artifacts.declaration.family(),
        &artifacts.support_report,
        &artifacts.parity_explanation,
        &artifacts.lifecycle_bundle,
        &artifacts.admitted_bundle,
        QuerySubscriptionLifecycleCoverageClass::LifecycleCloseout,
    )
    .unwrap();
    let matrix = build_query_subscription_family_coverage_matrix(vec![admitted_row])
        .expect("fixture digest must fit");

    let error = build_certified_family_coverage_handle(
        &matrix,
        artifacts.declaration.family(),
        CoverageResolutionPosture::MatrixScanDenied,
    )
    .unwrap_err();

    assert_eq!(
        error.error_kind(),
        &QuerySubscriptionRuntimeCertificationErrorKind::CoverageResolutionDenied
    );
}
