pub(crate) use {worth_foundational::performance, worth_foundational::performance_api, worth_foundational::profiles, worth_foundational::CertificationPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::FoundationalPerformanceAccessPatternPosture, worth_foundational::FoundationalPerformanceAttachmentDenial, worth_foundational::FoundationalPerformanceAttachmentTargetKind, worth_foundational::FoundationalPerformanceBoundary, worth_foundational::FoundationalPerformanceBreadthLocalityPosture, worth_foundational::FoundationalPerformanceCounterRow, worth_foundational::FoundationalPerformanceEvidenceStrength, worth_foundational::FoundationalPerformanceExecutionTemperature, worth_foundational::FoundationalPerformanceFallbackDebtPosture, worth_foundational::FoundationalPerformanceFreshnessRetentionPosture, worth_foundational::FoundationalPerformanceLayoutIntent, worth_foundational::FoundationalPerformanceReportMaterializationBoundary, worth_foundational::FoundationalPerformanceReportSection, worth_foundational::FoundationalPerformanceReportSectionDecisionCause, worth_foundational::FoundationalPerformanceWorkClass, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};

pub(crate) fn standard_profile() -> worth_foundational::FoundationalProfileSet {
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
        .expect("standard profile should compose")
}

pub(crate) fn operational_minimal_profile() -> worth_foundational::FoundationalProfileSet {
    profiles()
        .set()
        .diagnostic_richness(worth_foundational::DiagnosticRichnessProfile::OperationalMinimal)
        .support_posture(worth_foundational::SupportPostureProfile::InternalOnly)
        .compatibility_posture(worth_foundational::CompatibilityPostureProfile::NativeOnly)
        .admission_readiness(worth_foundational::AdmissionReadinessProfile::Admitted)
        .retention_delivery(worth_foundational::RetentionDeliveryProfile::Retained)
        .certification_posture(worth_foundational::CertificationPostureProfile::Uncertified)
        .execution_objective(worth_foundational::ExecutionObjectiveProfile::Throughput)
        .observation_activation(worth_foundational::ObservationActivationProfile::OnDemand)
        .compose()
        .expect("minimal profile should compose")
}

pub(crate) fn authoritative_bundle() -> worth_foundational::FoundationalPerformanceBundle<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
> {
    let claim = performance()
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
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("claim should build");

    let layout = performance().define_layout_intent(
        worth_foundational::FoundationalPerformanceLayoutIntent::AoS,
        worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup,
        worth_foundational::FoundationalPerformanceAllocationPosture::ActionLocal,
    );
    let contract_name =
        worth_foundational::FoundationalPerformanceContractName::new("query.snapshot.targets")
            .expect("valid contract");
    let counter_name =
        worth_foundational::FoundationalPerformanceCounterName::new("authoritative.rows")
            .expect("valid counter");
    let support_code = worth_foundational::FoundationalPerformanceSupportingEvidenceCode::new(
        "support.snapshot.audit",
    )
    .expect("valid support code");

    worth_foundational::performance_api::lower_lane::basis::performance_bundle(claim)
        .attach_layout_intent_claim(layout)
        .attach_contract_name(contract_name)
        .attach_counter_spec(worth_foundational::FoundationalPerformanceCounterSpec::new(
            counter_name,
            worth_foundational::FoundationalPerformanceWorkClass::AuthoritativeMutation,
            3,
        ))
        .attach_supporting_evidence_row(
            worth_foundational::FoundationalPerformanceSupportingEvidenceRow::new(
                support_code,
                worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly,
            ),
        )
        .finish()
        .expect("bundle should build")
}

pub(crate) fn policy_receipt() -> worth_foundational::FoundationalPolicyAdmissionReceipt {
    let claim = performance()
        .claim()
        .policy_admission()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::RuntimePolicyAdmission)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::DeltaBound)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(worth_foundational::FoundationalPerformanceWorkClass::ValidationPlanning)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .finish()
        .expect("policy claim should build");

    performance()
        .policy_admission_receipt(claim)
        .budget_decision(
            worth_foundational::FoundationalPerformanceBudgetKind::Breadth,
            4,
            4,
        )
        .finish()
        .expect("policy receipt should build")
}