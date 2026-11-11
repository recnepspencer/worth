use super::world::{hostile_support_failure_for, runtime_certification_artifacts_for};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn runtime_certification_scope_rejects_cross_family_coverage_handles() {
    let detail = runtime_certification_artifacts_for(LiveQueryFamily::Detail, None);
    let collection = runtime_certification_artifacts_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let collection_hostile = hostile_support_failure_for(&collection);
    let collection_matrix = build_query_subscription_family_coverage_matrix(vec![
        QuerySubscriptionFamilyCoverageRow::admitted(
            collection.declaration.family(),
            &collection.support_report,
            &collection.parity_explanation,
            &collection.lifecycle_bundle,
            &collection.admitted_bundle,
            QuerySubscriptionLifecycleCoverageClass::LifecycleCloseout,
        )
        .unwrap(),
        QuerySubscriptionFamilyCoverageRow::hostile(
            collection.declaration.family(),
            &collection.support_report,
            &collection.parity_explanation,
            &collection.lifecycle_bundle,
            &collection_hostile.denied_bundle,
            &collection_hostile.failure,
            QuerySubscriptionLifecycleCoverageClass::ActiveLifecycle,
        )
        .unwrap(),
    ])
    .expect("fixture digest must fit");
    let collection_handle = build_certified_family_coverage_handle(
        &collection_matrix,
        collection.declaration.family(),
        CoverageResolutionPosture::IndexedCoverageSet,
    )
    .unwrap();

    let error = build_query_subscription_runtime_certification_scope(
        detail.support_report,
        detail.parity_explanation,
        detail.admitted_bundle,
        detail.lifecycle_bundle,
        collection_handle,
    )
    .unwrap_err();

    assert_eq!(
        error.error_kind(),
        &QuerySubscriptionRuntimeCertificationErrorKind::ScopeFamilyMismatch
    );
}

#[test]
fn runtime_family_certification_rejects_non_runtime_support_subjects() {
    let artifacts = runtime_certification_artifacts_for(LiveQueryFamily::Detail, None);
    let hostile = hostile_support_failure_for(&artifacts);
    let declaration_support = report_query_subscription_support(
        QuerySubscriptionSupportSubject::declaration(&artifacts.declaration)
            .expect("fixture digest must fit"),
        QuerySubscriptionSupportEvidence::declaration(&artifacts.declaration),
    )
    .unwrap()
    .0;

    let error = QuerySubscriptionFamilyCoverageRow::admitted(
        artifacts.declaration.family(),
        &declaration_support,
        &artifacts.parity_explanation,
        &artifacts.lifecycle_bundle,
        &artifacts.admitted_bundle,
        QuerySubscriptionLifecycleCoverageClass::LifecycleCloseout,
    )
    .unwrap_err();

    assert_eq!(
        error.error_kind(),
        &QuerySubscriptionRuntimeCertificationErrorKind::CertificationSupportClassDenied
    );

    let hostile_error = QuerySubscriptionFamilyCoverageRow::hostile(
        artifacts.declaration.family(),
        &declaration_support,
        &artifacts.parity_explanation,
        &artifacts.lifecycle_bundle,
        &hostile.denied_bundle,
        &hostile.failure,
        QuerySubscriptionLifecycleCoverageClass::ActiveLifecycle,
    )
    .unwrap_err();

    assert_eq!(
        hostile_error.error_kind(),
        &QuerySubscriptionRuntimeCertificationErrorKind::CertificationSupportClassDenied
    );
}
