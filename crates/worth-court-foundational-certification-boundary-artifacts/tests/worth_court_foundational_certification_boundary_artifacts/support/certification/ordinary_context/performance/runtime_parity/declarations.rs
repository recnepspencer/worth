pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
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

pub(crate) use {
    crate::certification::ordinary_context::performance::basis_support::derive_digest,
    crate::certification::ordinary_context::performance::basis_support::exact_compare,
};

#[derive(Clone, Copy)]
pub(crate) enum SyntheticRuntimeFamily {
    QueryLike,
    StoreLike,
    RelationalLike,
}

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

pub(crate) fn canonical_hot_receipt(
    family: SyntheticRuntimeFamily,
    contract_name: &str,
) -> worth_foundational::FoundationalCounterBackedPerformanceReceipt<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
> {
    let common = worth_foundational::performance_api::common_path::performance();
    let mut claim_builder = common
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
    claim_builder = match family {
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::QueryLike => claim_builder
            .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
            .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
            .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ForensicParity),
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::StoreLike => claim_builder
            .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
            .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ForensicParity)
            .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction),
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::RelationalLike => claim_builder
            .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ForensicParity)
            .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
            .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly),
    };
    let claim = claim_builder.finish().expect("claim should build");

    let layout = common.define_layout_intent(
        worth_foundational::FoundationalPerformanceLayoutIntent::AoS,
        worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        worth_foundational::FoundationalPerformanceAllocationPosture::ActionLocal,
    );
    let counter_name =
        worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
            .expect("counter name");
    let support_code =
        worth_foundational::FoundationalPerformanceSupportingEvidenceCode::new("support.trace")
            .expect("support code");

    let bundle_builder =
        worth_foundational::performance_api::lower_lane::basis::performance_bundle(claim.clone())
            .attach_layout_intent_claim(layout.clone())
            .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
                counter_name.clone(),
                worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation,
                3,
            ));
    let bundle = match family {
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::QueryLike => bundle_builder
            .attach_contract_name(
                worth_foundational::FoundationalPerformanceContractName::new(contract_name)
                    .expect("contract"),
            )
            .attach_supporting_evidence_row(
                worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                    support_code,
                    worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly,
                ),
            )
            .finish()
            .expect("query-like bundle should build"),
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::StoreLike => bundle_builder
            .attach_supporting_evidence_row(
                worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                    support_code,
                    worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly,
                ),
            )
            .attach_contract_name(
                worth_foundational::FoundationalPerformanceContractName::new(contract_name)
                    .expect("contract"),
            )
            .finish()
            .expect("store-like bundle should build"),
        crate::certification::ordinary_context::performance::runtime_parity::SyntheticRuntimeFamily::RelationalLike => worth_foundational::performance_api::lower_lane::basis::performance_bundle(claim)
            .attach_contract_name(
                worth_foundational::FoundationalPerformanceContractName::new(contract_name)
                    .expect("contract"),
            )
            .attach_layout_intent_claim(layout)
            .attach_supporting_evidence_row(
                worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                    support_code,
                    worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly,
                ),
            )
            .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
                counter_name.clone(),
                worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation,
                3,
            ))
            .finish()
            .expect("relational-like bundle should build"),
    };

    worth_foundational::performance_api::lower_lane::receipts::counter_backed_performance_receipt(
        bundle,
    )
    .attach_counter_row(worth_foundational::FoundationalPerformanceCounterRow::new(
        counter_name,
        3,
    ))
    .finish()
    .expect("receipt should build")
}

pub(crate) fn support_expansion_report_from(
    receipt: worth_foundational::FoundationalCounterBackedPerformanceReceipt<
        worth_foundational::FoundationalAuthoritativePerformanceClaim,
    >,
) -> worth_foundational::FoundationalMaterializedPerformanceReport<
    worth_foundational::FoundationalAttachedCounterBackedPerformanceReceipt<
        worth_foundational::FoundationalAuthoritativePerformanceClaim,
    >,
> {
    let attached = worth_foundational::performance_api::lower_lane::reports::attach_counter_backed_performance_receipt(
        worth_foundational::FoundationalPerformanceAttachmentTargetKind::BoundaryReport,
        receipt,
    )
    .expect("receipt attachment should build");
    worth_foundational::performance_api::lower_lane::reports::plan_performance_report(
        worth_foundational::FoundationalPerformanceReportRequest {
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
        },
    )
    .materialize()
}
