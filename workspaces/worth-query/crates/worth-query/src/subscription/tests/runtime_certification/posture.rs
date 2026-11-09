use super::world::{
    hostile_support_failure_for, runtime_certification_artifacts_for,
    runtime_certification_artifacts_for_source,
};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn runtime_family_certification_preserves_matrix_scan_debt_posture() {
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
        CoverageResolutionPosture::MatrixScanDebtExplicit,
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

    let (bundle, receipt) = certify_query_subscription_runtime_family(scope).unwrap();

    assert_eq!(
        receipt.coverage_resolution_posture(),
        &CoverageResolutionPosture::MatrixScanDebtExplicit
    );
    assert_eq!(receipt.family_coverage_index_lookup_count(), 0);
    assert_eq!(bundle.counters().family_coverage_index_lookup_count(), 0);
    assert_eq!(
        bundle.counters().family_coverage_matrix_scan_debt_count(),
        1
    );
}

#[test]
fn hostile_family_coverage_rows_reject_denied_bundles_from_foreign_proof_chains() {
    let artifacts = runtime_certification_artifacts_for(LiveQueryFamily::Detail, None);
    let foreign = runtime_certification_artifacts_for_source(
        LiveQueryFamily::Detail,
        Some(LiveViewShapeFamily::Detail),
        QuerySubscriptionConstructionSource::Direct,
    );
    let foreign_hostile = hostile_support_failure_for(&foreign);

    let error = QuerySubscriptionFamilyCoverageRow::hostile(
        artifacts.declaration.family(),
        &artifacts.support_report,
        &artifacts.parity_explanation,
        &artifacts.lifecycle_bundle,
        &foreign_hostile.denied_bundle,
        &foreign_hostile.failure,
        QuerySubscriptionLifecycleCoverageClass::ActiveLifecycle,
    )
    .unwrap_err();

    assert_eq!(
        error.error_kind(),
        &QuerySubscriptionRuntimeCertificationErrorKind::ScopeSourceMismatch
    );
}
