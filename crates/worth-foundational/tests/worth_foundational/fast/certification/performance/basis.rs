use worth_foundational::{
    performance_api, FoundationalPerformanceAttachmentConstructionDenial,
    FoundationalPerformanceBundleConstructionDenial, FoundationalPerformanceCounterRow,
    FoundationalPerformanceMismatch, FoundationalPerformanceWorkClass,
};

use super::basis_support::authoritative_claim;

#[test]
fn lower_lane_builds_canonical_bundle_and_explains_mismatch_precisely() {
    let contract_name =
        performance_api::lower_lane::basis::FoundationalPerformanceContractName::new(
            "query.snapshot_explicit_targets",
        )
        .expect("valid contract name");
    let counter_name = performance_api::lower_lane::basis::FoundationalPerformanceCounterName::new(
        "authoritative_mutation.rows",
    )
    .expect("valid counter name");
    let support_code =
        performance_api::lower_lane::basis::FoundationalPerformanceSupportingEvidenceCode::new(
            "support.snapshot.audit",
        )
        .expect("valid support code");

    let left = performance_api::lower_lane::basis::performance_bundle(authoritative_claim())
        .attach_contract_name(contract_name.clone())
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            counter_name.clone(),
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .attach_supporting_evidence_row(
            worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                support_code.clone(),
                FoundationalPerformanceWorkClass::SupportReportAssembly,
            ),
        )
        .finish()
        .expect("left bundle should build");

    let right = performance_api::lower_lane::basis::performance_bundle(authoritative_claim())
        .attach_contract_name(contract_name)
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            counter_name,
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            5,
        ))
        .attach_supporting_evidence_row(
            worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                support_code,
                FoundationalPerformanceWorkClass::SupportReportAssembly,
            ),
        )
        .finish()
        .expect("right bundle should build");

    let comparison = performance_api::lower_lane::basis::compare_performance_bundles(&left, &right);
    assert!(!comparison.is_equivalent());
    assert_eq!(comparison.mismatches().len(), 1);
    assert!(matches!(
        comparison.mismatches()[0],
        FoundationalPerformanceMismatch::CounterSpecs { .. }
    ));
}

#[test]
fn bundle_comparison_covers_full_claim_surface_not_just_boundary_and_freshness() {
    let common = performance_api::common_path::performance();
    let left_claim = common
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(
            worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
        )
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
        )
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("left claim should build");
    let right_claim = common
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(
            worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt,
        )
        .breadth_locality(
            worth_foundational::FoundationalPerformanceBreadthLocalityPosture::BasisLocalBatch,
        )
        .access_pattern(
            worth_foundational::FoundationalPerformanceAccessPatternPosture::TraversalLocal,
        )
        .execution_temperature(
            worth_foundational::FoundationalPerformanceExecutionTemperature::WarmPath,
        )
        .freshness_retention(
            worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent,
        )
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Deferred)
        .include_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("right claim should build");

    let left = performance_api::lower_lane::basis::performance_bundle(left_claim)
        .finish()
        .expect("left bundle should build");
    let right = performance_api::lower_lane::basis::performance_bundle(right_claim)
        .finish()
        .expect("right bundle should build");

    let comparison = performance_api::lower_lane::basis::compare_performance_bundles(&left, &right);
    assert!(comparison.mismatches().iter().any(|mismatch| {
        matches!(
            mismatch,
            FoundationalPerformanceMismatch::BreadthLocality { .. }
        )
    }));
    assert!(comparison.mismatches().iter().any(|mismatch| {
        matches!(
            mismatch,
            FoundationalPerformanceMismatch::AccessPattern { .. }
        )
    }));
    assert!(comparison.mismatches().iter().any(|mismatch| {
        matches!(
            mismatch,
            FoundationalPerformanceMismatch::ExecutionTemperature { .. }
        )
    }));
    assert!(comparison.mismatches().iter().any(|mismatch| {
        matches!(
            mismatch,
            FoundationalPerformanceMismatch::FallbackDebt { .. }
        )
    }));
}

#[test]
fn attachment_names_and_bundle_members_fail_closed() {
    let invalid_name =
        performance_api::lower_lane::basis::FoundationalPerformanceContractName::new("Bad Name");
    assert_eq!(
        invalid_name,
        Err(FoundationalPerformanceAttachmentConstructionDenial::InvalidNameCharacter)
    );

    let counter_name = performance_api::lower_lane::basis::FoundationalPerformanceCounterName::new(
        "authoritative_mutation.rows",
    )
    .expect("valid counter name");
    let contract_name =
        performance_api::lower_lane::basis::FoundationalPerformanceContractName::new(
            "query.snapshot_explicit_targets",
        )
        .expect("valid contract name");

    let denial = performance_api::lower_lane::basis::performance_bundle(authoritative_claim())
        .attach_contract_name(contract_name.clone())
        .attach_contract_name(contract_name)
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            counter_name.clone(),
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            counter_name,
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .finish();
    assert_eq!(
        denial,
        Err(FoundationalPerformanceBundleConstructionDenial::DuplicateContractName)
    );
}

#[test]
fn counter_backed_receipts_require_exact_rows_and_counter_backed_claims() {
    let counter_name = performance_api::lower_lane::basis::FoundationalPerformanceCounterName::new(
        "authoritative_mutation.rows",
    )
    .expect("valid counter name");
    let bundle = performance_api::lower_lane::basis::performance_bundle(authoritative_claim())
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            counter_name.clone(),
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .finish()
        .expect("bundle should build");

    let receipt = performance_api::lower_lane::receipts::counter_backed_performance_receipt(bundle)
        .attach_counter_row(FoundationalPerformanceCounterRow::new(
            counter_name.clone(),
            3,
        ))
        .finish()
        .expect("counter-backed receipt should build");
    assert_eq!(receipt.counter_rows()[0].observed_count(), 3);

    let mismatch_bundle =
        performance_api::lower_lane::basis::performance_bundle(authoritative_claim())
            .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
                counter_name.clone(),
                FoundationalPerformanceWorkClass::AuthoritativeMutation,
                3,
            ))
            .finish()
            .expect("bundle should build");
    let mismatch =
        performance_api::lower_lane::receipts::counter_backed_performance_receipt(mismatch_bundle)
            .attach_counter_row(FoundationalPerformanceCounterRow::new(counter_name, 2))
            .finish();
    assert_eq!(
        mismatch,
        Err(
            worth_foundational::FoundationalCounterBackedPerformanceReceiptConstructionDenial::CounterValueMismatch
        )
    );
}
