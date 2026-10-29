use worth_foundational::{
    performance_api::{
        common_path,
        lower_lane::{basis, receipts, reports},
        stronger_lane::certified,
    },
    profiles, CertificationPostureProfile, DiagnosticRichnessProfile,
    FoundationalCertifiedPerformanceAttachmentDenial, FoundationalCertifiedPerformanceClass,
    FoundationalCertifiedPerformanceSourceKind, FoundationalPerformanceAccessPatternPosture,
    FoundationalPerformanceAttachmentTargetKind, FoundationalPerformanceBoundary,
    FoundationalPerformanceBreadthLocalityPosture, FoundationalPerformanceCounterName,
    FoundationalPerformanceCounterRow, FoundationalPerformanceCounterSpec,
    FoundationalPerformanceEvidenceStrength, FoundationalPerformanceExecutionTemperature,
    FoundationalPerformanceFallbackDebtPosture, FoundationalPerformanceFreshnessRetentionPosture,
    FoundationalPerformanceLayoutIntent, FoundationalPerformanceSupportingEvidenceCode,
    FoundationalPerformanceSupportingEvidenceRow, FoundationalPerformanceWorkClass,
    RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

use super::basis_support::exact_compare;

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

fn hot_path_receipt(
    include_required_exclusions: bool,
    include_support_row: bool,
) -> worth_foundational::FoundationalCounterBackedPerformanceReceipt<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
> {
    let common = common_path::performance();
    let mut builder = common
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

    if include_required_exclusions {
        builder = builder
            .exclude_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
            .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
            .exclude_work(FoundationalPerformanceWorkClass::ForensicParity);
    } else {
        builder = builder.exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly);
    }

    let claim = builder
        .finish()
        .expect("authoritative hot-path claim should build");
    let layout = common.define_layout_intent(
        FoundationalPerformanceLayoutIntent::AoS,
        FoundationalPerformanceAccessPatternPosture::PointLookup,
        worth_foundational::FoundationalPerformanceAllocationPosture::ActionLocal,
    );

    let mut bundle = basis::performance_bundle(claim)
        .attach_layout_intent_claim(layout)
        .attach_counter_spec(FoundationalPerformanceCounterSpec::new(
            FoundationalPerformanceCounterName::new("authoritative.rows").expect("counter name"),
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ));
    if include_support_row {
        bundle = bundle.attach_supporting_evidence_row(
            FoundationalPerformanceSupportingEvidenceRow::new(
                FoundationalPerformanceSupportingEvidenceCode::new("support.trace")
                    .expect("support code"),
                FoundationalPerformanceWorkClass::SupportReportAssembly,
            ),
        );
    }
    let bundle = bundle.finish().expect("bundle should build");

    receipts::counter_backed_performance_receipt(bundle)
        .attach_counter_row(FoundationalPerformanceCounterRow::new(
            FoundationalPerformanceCounterName::new("authoritative.rows").expect("counter name"),
            3,
        ))
        .finish()
        .expect("counter-backed receipt should build")
}

fn support_expansion_report(
    include_support_rows: bool,
) -> worth_foundational::FoundationalMaterializedPerformanceReport<
    worth_foundational::FoundationalAttachedCounterBackedPerformanceReceipt<
        worth_foundational::FoundationalAuthoritativePerformanceClaim,
    >,
> {
    let attached = reports::attach_counter_backed_performance_receipt(
        FoundationalPerformanceAttachmentTargetKind::BoundaryReport,
        hot_path_receipt(true, include_support_rows),
    )
    .expect("receipt attachment should build");

    reports::plan_performance_report(worth_foundational::FoundationalPerformanceReportRequest {
        source: attached,
        profile: support_ready_profile(),
        include_layout_intent: true,
        include_contract_names: false,
        include_counter_specs: true,
        include_counter_rows: true,
        include_supporting_evidence_rows: true,
        include_budget_decisions: false,
        include_denied_work: false,
        include_widened_work: false,
    })
    .materialize()
}

fn hot_path_receipt_with_contract(
    contract_name: &str,
) -> worth_foundational::FoundationalCounterBackedPerformanceReceipt<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
> {
    let common = common_path::performance();
    let claim = common
        .claim()
        .authoritative_execution()
        .boundary(FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(FoundationalPerformanceWorkClass::ForensicParity)
        .finish()
        .expect("authoritative claim should build");

    let bundle = basis::performance_bundle(claim)
        .attach_contract_name(
            worth_foundational::FoundationalPerformanceContractName::new(contract_name)
                .expect("contract name"),
        )
        .attach_counter_spec(FoundationalPerformanceCounterSpec::new(
            FoundationalPerformanceCounterName::new("authoritative.rows").expect("counter name"),
            FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .finish()
        .expect("bundle should build");

    receipts::counter_backed_performance_receipt(bundle)
        .attach_counter_row(FoundationalPerformanceCounterRow::new(
            FoundationalPerformanceCounterName::new("authoritative.rows").expect("counter name"),
            3,
        ))
        .finish()
        .expect("receipt should build")
}

#[test]
fn certified_performance_bundle_reuses_proof_lane_for_hot_path_receipts_and_support_reports() {
    let hot_source = hot_path_receipt(true, true);
    let hot_basis = match basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        basis::performance_basis_rule_version(),
        &hot_source,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected hot-path canonical basis"),
    };
    let hot_certified = match certified::certify_hot_path_counter_backed_performance_receipt(
        hot_source,
        certified::foundational_performance_certified_attachment_authority(),
    ) {
        TransitionOutcome::Success(bundle) => bundle,
        TransitionOutcome::Denied(denial) => {
            panic!("expected certified hot-path bundle, denied: {denial:?}")
        }
        TransitionOutcome::Deferred(_) => panic!("expected certified hot-path bundle, deferred"),
        TransitionOutcome::Stale(_) => panic!("expected certified hot-path bundle, stale"),
        TransitionOutcome::RebindRequired(_) => {
            panic!("expected certified hot-path bundle, rebind required")
        }
        TransitionOutcome::Failed(_) => panic!("expected certified hot-path bundle, failed"),
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
        hot_certified.readmission_basis().payload().domain(),
        worth_foundational::CanonicalBasisDomain::Performance
    );

    let hot_readmitted = certified::readmit_certified_performance_bundle_after_boundary(
        certified::bridge_certified_performance_bundle_trust_boundary(hot_certified),
        hot_basis,
        certified::foundational_performance_certified_readmission_authority(),
    );
    assert_eq!(
        hot_readmitted.certified_class(),
        FoundationalCertifiedPerformanceClass::HotPathOperational
    );

    let support_source = support_expansion_report(true);
    let support_basis = match basis::prepare_materialized_performance_report_for_canonical_basis(
        basis::performance_basis_rule_version(),
        &support_source,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected support-report canonical basis"),
    };
    let support_certified = match certified::certify_support_expansion_performance_report(
        support_source,
        certified::foundational_performance_certified_attachment_authority(),
    ) {
        TransitionOutcome::Success(bundle) => bundle,
        TransitionOutcome::Denied(denial) => {
            panic!("expected certified support-expansion bundle, denied: {denial:?}")
        }
        TransitionOutcome::Deferred(_) => {
            panic!("expected certified support-expansion bundle, deferred")
        }
        TransitionOutcome::Stale(_) => {
            panic!("expected certified support-expansion bundle, stale")
        }
        TransitionOutcome::RebindRequired(_) => {
            panic!("expected certified support-expansion bundle, rebind required")
        }
        TransitionOutcome::Failed(_) => {
            panic!("expected certified support-expansion bundle, failed")
        }
    };

    assert_eq!(
        support_certified.source_kind(),
        FoundationalCertifiedPerformanceSourceKind::MaterializedSupportExpansionReport
    );
    assert_eq!(
        support_certified.certified_class(),
        FoundationalCertifiedPerformanceClass::SupportExpansionCompatibility
    );
    assert_eq!(
        support_certified.readmission_basis().payload().domain(),
        worth_foundational::CanonicalBasisDomain::Performance
    );

    let support_readmitted = certified::readmit_certified_performance_bundle_after_boundary(
        certified::bridge_certified_performance_bundle_trust_boundary(support_certified),
        support_basis,
        certified::foundational_performance_certified_readmission_authority(),
    );
    assert_eq!(
        support_readmitted.source().supporting_evidence_rows().len(),
        1
    );
}

#[test]
fn certified_performance_readmission_basis_tracks_canonical_meaning_not_shallow_shape_counts() {
    let left = match certified::certify_hot_path_counter_backed_performance_receipt(
        hot_path_receipt_with_contract("query.snapshot_explicit_targets"),
        certified::foundational_performance_certified_attachment_authority(),
    ) {
        TransitionOutcome::Success(bundle) => bundle,
        _ => panic!("expected certified left bundle"),
    };
    let right = match certified::certify_hot_path_counter_backed_performance_receipt(
        hot_path_receipt_with_contract("query.snapshot_branch_targets"),
        certified::foundational_performance_certified_attachment_authority(),
    ) {
        TransitionOutcome::Success(bundle) => bundle,
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
    let left_basis = match basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        basis::performance_basis_rule_version(),
        left.source(),
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("left performance basis should prepare"),
    };
    let right_basis = match basis::prepare_counter_backed_performance_receipt_for_canonical_basis(
        basis::performance_basis_rule_version(),
        right.source(),
    ) {
        TransitionOutcome::Success(ready) => ready,
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
        certified::certify_hot_path_counter_backed_performance_receipt(
            hot_path_receipt(false, true),
            certified::foundational_performance_certified_attachment_authority(),
        ),
        TransitionOutcome::Denied(
            FoundationalCertifiedPerformanceAttachmentDenial::HotPathCertificationRequiresExplicitOperationalExclusions
        )
    ));

    assert!(matches!(
        certified::certify_support_expansion_performance_report(
            support_expansion_report(false),
            certified::foundational_performance_certified_attachment_authority(),
        ),
        TransitionOutcome::Denied(
            FoundationalCertifiedPerformanceAttachmentDenial::SupportCertificationRequiresSupportExpansionBoundary
        )
    ));
}
