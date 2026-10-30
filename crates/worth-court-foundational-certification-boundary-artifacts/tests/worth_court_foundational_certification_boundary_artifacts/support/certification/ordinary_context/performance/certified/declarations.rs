pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::performance_api::common_path,
    worth_foundational::performance_api::lower_lane::basis,
    worth_foundational::performance_api::lower_lane::receipts,
    worth_foundational::performance_api::lower_lane::reports,
    worth_foundational::performance_api::stronger_lane::certified, worth_foundational::profiles,
    worth_foundational::CertificationPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalCertifiedPerformanceAttachmentDenial,
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
    worth_foundational::FoundationalPerformanceSupportingEvidenceCode,
    worth_foundational::FoundationalPerformanceSupportingEvidenceRow,
    worth_foundational::FoundationalPerformanceWorkClass,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

pub(crate) use crate::certification::ordinary_context::performance::basis_support::exact_compare;

pub(crate) fn support_ready_profile() -> worth_foundational::FoundationalProfileSet {
    profiles()
        .set()
        .diagnostic_richness(worth_foundational::DiagnosticRichnessProfile::Standard)
        .support_posture(worth_foundational::SupportPostureProfile::SupportReady)
        .compatibility_posture(worth_foundational::CompatibilityPostureProfile::NativeOnly)
        .admission_readiness(worth_foundational::AdmissionReadinessProfile::Admitted)
        .retention_delivery(worth_foundational::RetentionDeliveryProfile::Retained)
        .certification_posture(worth_foundational::CertificationPostureProfile::Uncertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Balanced)
        .observation_activation(worth_foundational::ObservationActivationProfile::Continuous)
        .compose()
        .expect("support-ready profile should compose")
}

pub(crate) fn hot_path_receipt(
    include_required_exclusions: bool,
    include_support_row: bool,
) -> worth_foundational::FoundationalCounterBackedPerformanceReceipt<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
> {
    let common = worth_foundational::performance_api::common_path::performance();
    let mut builder = common
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation);

    if include_required_exclusions {
        builder = builder
            .exclude_work(
                worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction,
            )
            .exclude_work(
                worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly,
            )
            .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ForensicParity);
    } else {
        builder = builder.exclude_work(
            worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly,
        );
    }

    let claim = builder
        .finish()
        .expect("authoritative hot-path claim should build");
    let layout = common.define_layout_intent(
        worth_foundational::FoundationalPerformanceLayoutIntent::AoS,
        worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        worth_foundational::FoundationalPerformanceAllocationPosture::ActionLocal,
    );

    let mut bundle =
        worth_foundational::performance_api::lower_lane::basis::performance_bundle(claim)
            .attach_layout_intent_claim(layout)
            .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
                worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
                    .expect("counter name"),
                worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation,
                3,
            ));
    if include_support_row {
        bundle = bundle.attach_supporting_evidence_row(
            worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                worth_foundational::FoundationalPerformanceSupportingEvidenceCode::new(
                    "support.trace",
                )
                .expect("support code"),
                worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly,
            ),
        );
    }
    let bundle = bundle.finish().expect("bundle should build");

    worth_foundational::performance_api::lower_lane::receipts::counter_backed_performance_receipt(
        bundle,
    )
    .attach_counter_row(worth_foundational::FoundationalPerformanceCounterRow::new(
        worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
            .expect("counter name"),
        3,
    ))
    .finish()
    .expect("counter-backed receipt should build")
}

pub(crate) fn support_expansion_report(
    include_support_rows: bool,
) -> worth_foundational::FoundationalMaterializedPerformanceReport<
    worth_foundational::FoundationalAttachedCounterBackedPerformanceReceipt<
        worth_foundational::FoundationalAuthoritativePerformanceClaim,
    >,
> {
    let attached = worth_foundational::performance_api::lower_lane::reports::attach_counter_backed_performance_receipt(
        worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryReport,
        hot_path_receipt(true, include_support_rows),
    )
    .expect("receipt attachment should build");

    worth_foundational::performance_api::lower_lane::reports::plan_performance_report(
        worth_foundational::FoundationalPerformanceReportRequest {
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
        },
    )
    .materialize()
}

pub(crate) fn hot_path_receipt_with_contract(
    contract_name: &str,
) -> worth_foundational::FoundationalCounterBackedPerformanceReceipt<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
> {
    let common = worth_foundational::performance_api::common_path::performance();
    let claim = common
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ForensicParity)
        .finish()
        .expect("authoritative claim should build");

    let bundle = worth_foundational::performance_api::lower_lane::basis::performance_bundle(claim)
        .attach_contract_name(
            worth_foundational::FoundationalPerformanceContractName::new(contract_name)
                .expect("contract name"),
        )
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
                .expect("counter name"),
            worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .finish()
        .expect("bundle should build");

    worth_foundational::performance_api::lower_lane::receipts::counter_backed_performance_receipt(
        bundle,
    )
    .attach_counter_row(worth_foundational::FoundationalPerformanceCounterRow::new(
        worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
            .expect("counter name"),
        3,
    ))
    .finish()
    .expect("receipt should build")
}
