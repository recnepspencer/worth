#[allow(unused_imports)]
use crate::certification::ordinary_context::performance::runtime_parity::*;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::performance_api::common_path,
    worth_foundational::performance_api::lower_lane::basis,
    worth_foundational::performance_api::lower_lane::receipts,
    worth_foundational::performance_api::lower_lane::reports,
    worth_foundational::performance_api::stronger_lane::certified, worth_foundational::profiles,
    worth_foundational::CertificationPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalCertifiedPerformanceClass,
    worth_foundational::FoundationalCertifiedPerformanceSourceKind,
    worth_foundational::FoundationalPerformanceAccessPatternPosture,
    worth_foundational::FoundationalPerformanceAttachmentTargetKind,
    worth_foundational::FoundationalPerformanceBoundary,
    worth_foundational::FoundationalPerformanceBreadthLocalityPosture,
    worth_foundational::FoundationalPerformanceCounterName,
    worth_foundational::FoundationalPerformanceCounterRow,
    worth_foundational::FoundationalPerformanceCounterSpec,
    worth_foundational::FoundationalPerformanceEvidenceStrength,
    worth_foundational::FoundationalPerformanceExecutionTemperature,
    worth_foundational::FoundationalPerformanceFallbackDebtPosture,
    worth_foundational::FoundationalPerformanceFreshnessRetentionPosture,
    worth_foundational::FoundationalPerformanceLayoutIntent,
    worth_foundational::FoundationalPerformanceWorkClass,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

use {
    crate::certification::ordinary_context::performance::basis_support::derive_digest,
    crate::certification::ordinary_context::performance::basis_support::exact_compare,
};

#[test]
fn adopting_runtime_families_share_one_canonical_hot_path_meaning() {
    let query_like = canonical_hot_receipt(
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::QueryLike,
        "query.snapshot.explicit_targets",
    );
    let store_like = canonical_hot_receipt(
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::StoreLike,
        "query.snapshot.explicit_targets",
    );
    let relational_like = canonical_hot_receipt(
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::RelationalLike,
        "query.snapshot.explicit_targets",
    );

    let query_ready = match worth_foundational::performance_api::lower_lane::basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
        &query_like,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("query-like basis should prepare"),
    };
    let store_ready = match worth_foundational::performance_api::lower_lane::basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
        &store_like,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("store-like basis should prepare"),
    };
    let _relational_ready =
        match worth_foundational::performance_api::lower_lane::basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
            worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
            &relational_like,
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("relational-like basis should prepare"),
        };

    assert!(matches!(
        exact_compare(query_ready, store_ready),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));
    let query_digest = derive_digest(
        match worth_foundational::performance_api::lower_lane::basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
            worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
            &query_like,
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("query-like digest basis should prepare"),
        },
    );
    let relational_digest = derive_digest(
        match worth_foundational::performance_api::lower_lane::basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
            worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
            &relational_like,
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("relational-like digest basis should prepare"),
        },
    );
    assert_eq!(query_digest.value(), relational_digest.value());
}

#[test]
fn adopting_runtime_drift_stays_blind_consumer_visible() {
    let canonical = canonical_hot_receipt(
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::QueryLike,
        "query.snapshot.explicit_targets",
    );
    let drifted = canonical_hot_receipt(
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::StoreLike,
        "query.snapshot.branch_targets",
    );

    let canonical_ready =
        match worth_foundational::performance_api::lower_lane::basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
            worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
            &canonical,
        ) {
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("canonical basis should prepare"),
        };
    let drifted_ready = match worth_foundational::performance_api::lower_lane::basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        worth_foundational::performance_api::lower_lane::basis::performance_basis_rule_version(),
        &drifted,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("drifted basis should prepare"),
    };

    assert!(matches!(
        exact_compare(canonical_ready, drifted_ready),
        worth_foundational::CanonicalComparisonOutcome::Mismatched(_)
    ));
}

#[test]
fn certified_source_compatibility_matrix_is_runtime_adoption_proven_locally() {
    let hot_receipt = canonical_hot_receipt(
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::QueryLike,
        "query.snapshot.explicit_targets",
    );
    let hot_certified = match worth_foundational::performance_api::stronger_lane::certified::certify_hot_path_counter_backed_performance_receipt(
        hot_receipt,
        worth_foundational::performance_api::stronger_lane::certified::foundational_performance_certified_attachment_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("hot certified bundle should build"),
    };
    let support_report = support_expansion_report_from(canonical_hot_receipt(
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::RelationalLike,
        "query.snapshot.explicit_targets",
    ));
    let support_certified = match worth_foundational::performance_api::stronger_lane::certified::certify_support_expansion_performance_report(
        support_report,
        worth_foundational::performance_api::stronger_lane::certified::foundational_performance_certified_attachment_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("support certified bundle should build"),
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
        hot_certified.source_digest().domain(),
        worth_foundational::CanonicalBasisDomain::Performance
    );
    assert!(hot_certified.source_digest().entry_count() > 0);

    assert_eq!(
        support_certified.source_kind(),
        worth_foundational::FoundationalCertifiedPerformanceSourceKind::MaterializedSupportExpansionReport
    );
    assert_eq!(
        support_certified.certified_class(),
        worth_foundational::FoundationalCertifiedPerformanceClass::SupportExpansionCompatibility
    );
    assert_eq!(
        support_certified.source_digest().domain(),
        worth_foundational::CanonicalBasisDomain::Performance
    );
    assert!(support_certified.source_digest().entry_count() > 0);
}
