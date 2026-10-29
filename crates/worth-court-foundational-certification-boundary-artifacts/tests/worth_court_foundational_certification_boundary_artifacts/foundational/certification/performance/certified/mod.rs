#[allow(unused_imports)] use crate::certification::ordinary_context::performance::certified::*;
use {worth_foundational::performance_api::common_path, worth_foundational::performance_api::lower_lane::basis, worth_foundational::performance_api::lower_lane::receipts, worth_foundational::performance_api::lower_lane::reports, worth_foundational::performance_api::stronger_lane::certified, worth_foundational::profiles, worth_foundational::CertificationPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::FoundationalCertifiedPerformanceAttachmentDenial, worth_foundational::FoundationalCertifiedPerformanceClass, worth_foundational::FoundationalCertifiedPerformanceSourceKind, worth_foundational::FoundationalPerformanceAccessPatternPosture, worth_foundational::FoundationalPerformanceAttachmentTargetKind, worth_foundational::FoundationalPerformanceBoundary, worth_foundational::FoundationalPerformanceBreadthLocalityPosture, worth_foundational::FoundationalPerformanceCounterName, worth_foundational::FoundationalPerformanceCounterRow, worth_foundational::FoundationalPerformanceCounterSpec, worth_foundational::FoundationalPerformanceEvidenceStrength, worth_foundational::FoundationalPerformanceExecutionTemperature, worth_foundational::FoundationalPerformanceFallbackDebtPosture, worth_foundational::FoundationalPerformanceFreshnessRetentionPosture, worth_foundational::FoundationalPerformanceLayoutIntent, worth_foundational::FoundationalPerformanceSupportingEvidenceCode, worth_foundational::FoundationalPerformanceSupportingEvidenceRow, worth_foundational::FoundationalPerformanceWorkClass, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};
use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::performance::basis_support::exact_compare};

#[test]
fn certified_performance_bundle_reuses_proof_lane_for_hot_path_receipts_and_support_reports() {
    let hot_source = hot_path_receipt(true, true);
    let hot_basis = match worth_foundational::performance_api::lower_lane::basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
        &hot_source,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected hot-path canonical basis"),
    };
    let hot_certified = match worth_foundational::performance_api::stronger_lane::certified::certify_hot_path_counter_backed_performance_receipt(
        hot_source,
        worth_foundational::performance_api::stronger_lane::certified::foundational_performance_certified_attachment_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(bundle) => bundle,
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("expected certified hot-path bundle, denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(_) => panic!("expected certified hot-path bundle, deferred"),
        worth_proof::TransitionOutcome::Stale(_) => panic!("expected certified hot-path bundle, stale"),
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("expected certified hot-path bundle, rebind required")
        }
        worth_proof::TransitionOutcome::Failed(_) => panic!("expected certified hot-path bundle, failed"),
    };

    assert_eq!(
        hot_certified.source_kind(),
        worth_foundational::FoundationalCertifiedPerformanceSourceKind::CurrentBasisCounterBackedExecutionReceipt
    );
    assert_eq!(
        hot_certified.certified_class(),
        worth_foundational::FoundationalCertifiedPerformanceClass::HotPathOperational
    );
    assert_eq!(
        hot_certified.readmission_basis().payload().domain(),
        worth_foundational::CanonicalBasisDomain::Performance
    );

    let hot_readmitted = worth_foundational::performance_api::stronger_lane::certified::readmit_certified_performance_bundle_after_boundary(
        worth_foundational::performance_api::stronger_lane::certified::bridge_certified_performance_bundle_trust_boundary(hot_certified),
        hot_basis,
        worth_foundational::performance_api::stronger_lane::certified::foundational_performance_certified_readmission_authority(),
    );
    assert_eq!(
        hot_readmitted.certified_class(),
        worth_foundational::FoundationalCertifiedPerformanceClass::HotPathOperational
    );

    let support_source = support_expansion_report(true);
    let support_basis = match worth_foundational::performance_api::lower_lane::basis::prepare_materialized_performance_report_for_canonical_basis(
        worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
        &support_source,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected support-report canonical basis"),
    };
    let support_certified = match worth_foundational::performance_api::stronger_lane::certified::certify_support_expansion_performance_report(
        support_source,
        worth_foundational::performance_api::stronger_lane::certified::foundational_performance_certified_attachment_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(bundle) => bundle,
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("expected certified support-expansion bundle, denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(_) => {
            panic!("expected certified support-expansion bundle, deferred")
        }
        worth_proof::TransitionOutcome::Stale(_) => {
            panic!("expected certified support-expansion bundle, stale")
        }
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("expected certified support-expansion bundle, rebind required")
        }
        worth_proof::TransitionOutcome::Failed(_) => {
            panic!("expected certified support-expansion bundle, failed")
        }
    };

    assert_eq!(
        support_certified.source_kind(),
        worth_foundational::FoundationalCertifiedPerformanceSourceKind::MaterializedSupportExpansionReport
    );
    assert_eq!(
        support_certified.certified_class(),
        worth_foundational::FoundationalCertifiedPerformanceClass::SupportExpansionCompatibility
    );
    assert_eq!(
        support_certified.readmission_basis().payload().domain(),
        worth_foundational::CanonicalBasisDomain::Performance
    );

    let support_readmitted = worth_foundational::performance_api::stronger_lane::certified::readmit_certified_performance_bundle_after_boundary(
        worth_foundational::performance_api::stronger_lane::certified::bridge_certified_performance_bundle_trust_boundary(support_certified),
        support_basis,
        worth_foundational::performance_api::stronger_lane::certified::foundational_performance_certified_readmission_authority(),
    );
    assert_eq!(
        support_readmitted.source().supporting_evidence_rows().len(),
        1
    );
}

#[test]
fn certified_performance_readmission_basis_tracks_canonical_meaning_not_shallow_shape_counts() {
    let left = match worth_foundational::performance_api::stronger_lane::certified::certify_hot_path_counter_backed_performance_receipt(
        hot_path_receipt_with_contract("query.snapshot_explicit_targets"),
        worth_foundational::performance_api::stronger_lane::certified::foundational_performance_certified_attachment_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("expected certified left bundle"),
    };
    let right = match worth_foundational::performance_api::stronger_lane::certified::certify_hot_path_counter_backed_performance_receipt(
        hot_path_receipt_with_contract("query.snapshot_branch_targets"),
        worth_foundational::performance_api::stronger_lane::certified::foundational_performance_certified_attachment_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("expected certified right bundle"),
    };

    assert_eq!(
        left.source_digest().entry_count(),
        right.source_digest().entry_count()
    );
    assert_eq!(
        left.source_digest().domain(),
        right.source_digest().domain()
    );
    let left_basis = match worth_foundational::performance_api::lower_lane::basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
        left.source(),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("left performance basis should prepare"),
    };
    let right_basis = match worth_foundational::performance_api::lower_lane::basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
        right.source(),
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("right performance basis should prepare"),
    };
    assert!(matches!(
        exact_compare(left_basis, right_basis),
        worth_foundational::CanonicalComparisonOutcome::Mismatched(_)
    ));
}

#[test]
fn certified_performance_bundle_denies_hot_path_overclaim_and_support_free_support_certification() {
    assert!(matches!(
        worth_foundational::performance_api::stronger_lane::certified::certify_hot_path_counter_backed_performance_receipt(
            hot_path_receipt(false, true),
            worth_foundational::performance_api::stronger_lane::certified::foundational_performance_certified_attachment_authority(),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalCertifiedPerformanceAttachmentDenial::HotPathCertificationRequiresExplicitOperationalExclusions
        )
    ));

    assert!(matches!(
        worth_foundational::performance_api::stronger_lane::certified::certify_support_expansion_performance_report(
            support_expansion_report(false),
            worth_foundational::performance_api::stronger_lane::certified::foundational_performance_certified_attachment_authority(),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalCertifiedPerformanceAttachmentDenial::SupportCertificationRequiresSupportExpansionBoundary
        )
    ));
}
