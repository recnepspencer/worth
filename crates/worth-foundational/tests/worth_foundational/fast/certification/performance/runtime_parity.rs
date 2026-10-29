use worth_foundational::{
    performance_api::{
        common_path,
        lower_lane::{basis, receipts, reports},
        stronger_lane::certified,
    },
    profiles, CertificationPostureProfile, DiagnosticRichnessProfile,
    FoundationalCertifiedPerformanceClass, FoundationalCertifiedPerformanceSourceKind,
    FoundationalPerformanceAccessPatternPosture, FoundationalPerformanceAttachmentTargetKind,
    FoundationalPerformanceBoundary, FoundationalPerformanceBreadthLocalityPosture,
    FoundationalPerformanceCounterName, FoundationalPerformanceCounterRow,
    FoundationalPerformanceCounterSpec, FoundationalPerformanceEvidenceStrength,
    FoundationalPerformanceExecutionTemperature, FoundationalPerformanceFallbackDebtPosture,
    FoundationalPerformanceFreshnessRetentionPosture, FoundationalPerformanceLayoutIntent,
    FoundationalPerformanceWorkClass, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

use super::basis_support::{derive_digest, exact_compare};

#[derive(Clone, Copy)]
enum SyntheticRuntimeFamily {
    QueryLike,
    StoreLike,
    RelationalLike,
}

fn support_ready_profile() -> worth_foundational::FoundationalProfileSet {
    profiles()
        .set()
        .diagnostic_richness(DiagnosticRichnessProfile::Standard)
        .support_posture(SupportPostureProfile::SupportReady)
        .compatibility_posture(worth_foundational::CompatibilityPostureProfile::NativeOnly)
        .admission_readiness(worth_foundational::AdmissionReadinessProfile::Admitted)
        .retention_delivery(RetentionDeliveryProfile::Retained)
        .certification_posture(CertificationPostureProfile::Uncertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Balanced)
        .observation_activation(worth_foundational::ObservationActivationProfile::Continuous)
        .compose()
        .expect("support-ready profile should compose")
}

fn canonical_hot_receipt(
    family: SyntheticRuntimeFamily,
    contract_name: &str,
) -> worth_foundational::FoundationalCounterBackedPerformanceReceipt<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
> {
    let common = common_path::performance();
    let mut claim_builder = common
        .claim()
        .authoritative_execution()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::AuthoritativeMutation);
    claim_builder = match family {
        SyntheticRuntimeFamily::QueryLike => claim_builder
            .exclude_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
            .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
            .exclude_work(FoundationalPerformanceWorkClass::ForensicParity),
        SyntheticRuntimeFamily::StoreLike => claim_builder
            .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
            .exclude_work(FoundationalPerformanceWorkClass::ForensicParity)
            .exclude_work(FoundationalPerformanceWorkClass::ReplayReconstruction),
        SyntheticRuntimeFamily::RelationalLike => claim_builder
            .exclude_work(FoundationalPerformanceWorkClass::ForensicParity)
            .exclude_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
            .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly),
    };
    let claim = claim_builder.finish().expect("claim should build");

    let layout = common.define_layout_intent(
        FoundationalPerformanceLayoutIntent::AoS,
        FoundationalPerformanceAccessPatternPosture::PointLookup,
        worth_foundational::FoundationalPerformanceAllocationPosture::ActionLocal,
    );
    let counter_name =
        FoundationalPerformanceCounterName::new("authoritative.rows").expect("counter name");
    let support_code =
        worth_foundational::FoundationalPerformanceSupportingEvidenceCode::new("support.trace")
            .expect("support code");

    let bundle_builder = basis::performance_bundle(claim.clone())
        .attach_layout_intent_claim(layout.clone())
        .attach_counter_spec(FoundationalPerformanceCounterSpec::new(
            counter_name.clone(),
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ));
    let bundle = match family {
        SyntheticRuntimeFamily::QueryLike => bundle_builder
            .attach_contract_name(
                worth_foundational::FoundationalPerformanceContractName::new(contract_name)
                    .expect("contract"),
            )
            .attach_supporting_evidence_row(
                worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                    support_code,
                    FoundationalPerformanceWorkClass::SupportReportAssembly,
                ),
            )
            .finish()
            .expect("query-like bundle should build"),
        SyntheticRuntimeFamily::StoreLike => bundle_builder
            .attach_supporting_evidence_row(
                worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                    support_code,
                    FoundationalPerformanceWorkClass::SupportReportAssembly,
                ),
            )
            .attach_contract_name(
                worth_foundational::FoundationalPerformanceContractName::new(contract_name)
                    .expect("contract"),
            )
            .finish()
            .expect("store-like bundle should build"),
        SyntheticRuntimeFamily::RelationalLike => basis::performance_bundle(claim)
            .attach_contract_name(
                worth_foundational::FoundationalPerformanceContractName::new(contract_name)
                    .expect("contract"),
            )
            .attach_layout_intent_claim(layout)
            .attach_supporting_evidence_row(
                worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                    support_code,
                    FoundationalPerformanceWorkClass::SupportReportAssembly,
                ),
            )
            .attach_counter_spec(FoundationalPerformanceCounterSpec::new(
                counter_name.clone(),
                FoundationalPerformanceWorkClass::AuthoritativeMutation,
                3,
            ))
            .finish()
            .expect("relational-like bundle should build"),
    };

    receipts::counter_backed_performance_receipt(bundle)
        .attach_counter_row(FoundationalPerformanceCounterRow::new(counter_name, 3))
        .finish()
        .expect("receipt should build")
}

fn support_expansion_report_from(
    receipt: worth_foundational::FoundationalCounterBackedPerformanceReceipt<
        worth_foundational::FoundationalAuthoritativePerformanceClaim,
    >,
) -> worth_foundational::FoundationalMaterializedPerformanceReport<
    worth_foundational::FoundationalAttachedCounterBackedPerformanceReceipt<
        worth_foundational::FoundationalAuthoritativePerformanceClaim,
    >,
> {
    let attached = reports::attach_counter_backed_performance_receipt(
        FoundationalPerformanceAttachmentTargetKind::BoundaryReport,
        receipt,
    )
    .expect("receipt attachment should build");
    reports::plan_performance_report(worth_foundational::FoundationalPerformanceReportRequest {
        source: attached,
        profile: support_ready_profile(),
        include_layout_intent: true,
        include_contract_names: true,
        include_counter_specs: true,
        include_counter_rows: true,
        include_supporting_evidence_rows: true,
        include_budget_decisions: false,
        include_denied_work: false,
        include_widened_work: false,
    })
    .materialize()
}

#[test]
fn adopting_runtime_families_share_one_canonical_hot_path_meaning() {
    let query_like = canonical_hot_receipt(
        SyntheticRuntimeFamily::QueryLike,
        "query.snapshot.explicit_targets",
    );
    let store_like = canonical_hot_receipt(
        SyntheticRuntimeFamily::StoreLike,
        "query.snapshot.explicit_targets",
    );
    let relational_like = canonical_hot_receipt(
        SyntheticRuntimeFamily::RelationalLike,
        "query.snapshot.explicit_targets",
    );

    let query_ready = match basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        basis::performance_basis_rule_version(),
        &query_like,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("query-like basis should prepare"),
    };
    let store_ready = match basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        basis::performance_basis_rule_version(),
        &store_like,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("store-like basis should prepare"),
    };
    let _relational_ready =
        match basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
            basis::performance_basis_rule_version(),
            &relational_like,
        ) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("relational-like basis should prepare"),
        };

    assert!(matches!(
        exact_compare(query_ready, store_ready),
        worth_foundational::CanonicalComparisonOutcome::Equivalent(_)
    ));
    let query_digest = derive_digest(
        match basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
            basis::performance_basis_rule_version(),
            &query_like,
        ) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("query-like digest basis should prepare"),
        },
    );
    let relational_digest = derive_digest(
        match basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
            basis::performance_basis_rule_version(),
            &relational_like,
        ) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("relational-like digest basis should prepare"),
        },
    );
    assert_eq!(query_digest.value(), relational_digest.value());
}

#[test]
fn adopting_runtime_drift_stays_blind_consumer_visible() {
    let canonical = canonical_hot_receipt(
        SyntheticRuntimeFamily::QueryLike,
        "query.snapshot.explicit_targets",
    );
    let drifted = canonical_hot_receipt(
        SyntheticRuntimeFamily::StoreLike,
        "query.snapshot.branch_targets",
    );

    let canonical_ready =
        match basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
            basis::performance_basis_rule_version(),
            &canonical,
        ) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("canonical basis should prepare"),
        };
    let drifted_ready = match basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        basis::performance_basis_rule_version(),
        &drifted,
    ) {
        TransitionOutcome::Success(ready) => ready,
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
        SyntheticRuntimeFamily::QueryLike,
        "query.snapshot.explicit_targets",
    );
    let hot_certified = match certified::certify_hot_path_counter_backed_performance_receipt(
        hot_receipt,
        certified::foundational_performance_certified_attachment_authority(),
    ) {
        TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("hot certified bundle should build"),
    };
    let support_report = support_expansion_report_from(canonical_hot_receipt(
        SyntheticRuntimeFamily::RelationalLike,
        "query.snapshot.explicit_targets",
    ));
    let support_certified = match certified::certify_support_expansion_performance_report(
        support_report,
        certified::foundational_performance_certified_attachment_authority(),
    ) {
        TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("support certified bundle should build"),
    };

    assert_eq!(
        hot_certified.source_kind(),
        FoundationalCertifiedPerformanceSourceKind::CurrentBasisCounterBackedExecutionReceipt
    );
    assert_eq!(
        hot_certified.certified_class(),
        FoundationalCertifiedPerformanceClass::HotPathOperational
    );
    assert_eq!(
        hot_certified.source_digest().domain(),
        worth_foundational::CanonicalBasisDomain::Performance
    );
    assert!(hot_certified.source_digest().entry_count() > 0);

    assert_eq!(
        support_certified.source_kind(),
        FoundationalCertifiedPerformanceSourceKind::MaterializedSupportExpansionReport
    );
    assert_eq!(
        support_certified.certified_class(),
        FoundationalCertifiedPerformanceClass::SupportExpansionCompatibility
    );
    assert_eq!(
        support_certified.source_digest().domain(),
        worth_foundational::CanonicalBasisDomain::Performance
    );
    assert!(support_certified.source_digest().entry_count() > 0);
}
