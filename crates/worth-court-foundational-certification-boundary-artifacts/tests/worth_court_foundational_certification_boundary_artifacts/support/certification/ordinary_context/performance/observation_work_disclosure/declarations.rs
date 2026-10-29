pub(crate) use {worth_foundational::admit_requested_foundational_profile, worth_foundational::derive_foundational_profile_identity, worth_foundational::foundational_profile_progression_authority, worth_foundational::performance, worth_foundational::performance_api, worth_foundational::profiles, worth_foundational::request_foundational_profile_set, worth_foundational::AdmissionReadinessProfile, worth_foundational::CanonicalizationRuleVersion, worth_foundational::CertificationPostureProfile, worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::FoundationalObservationDisposition, worth_foundational::FoundationalPerformanceAccessPatternPosture, worth_foundational::FoundationalPerformanceAttachmentTargetKind, worth_foundational::FoundationalPerformanceBoundary, worth_foundational::FoundationalPerformanceBreadthLocalityPosture, worth_foundational::FoundationalPerformanceClaimConstructionDenial, worth_foundational::FoundationalPerformanceEvidenceStrength, worth_foundational::FoundationalPerformanceExecutionTemperature, worth_foundational::FoundationalPerformanceFallbackDebtPosture, worth_foundational::FoundationalPerformanceFreshnessRetentionPosture, worth_foundational::FoundationalPerformanceMismatch, worth_foundational::FoundationalPerformanceObservationContext, worth_foundational::FoundationalPerformanceReportRequest, worth_foundational::FoundationalPerformanceWorkClass, worth_foundational::ObservationActivationProfile, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) fn claim_with_work(
    work_class: FoundationalPerformanceWorkClass,
    context: Option<FoundationalPerformanceObservationContext>,
) -> Result<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
    FoundationalPerformanceClaimConstructionDenial,
> {
    let mut builder = performance()
        .claim()
        .authoritative_execution()
        .boundary(worth_foundational::FoundationalPerformanceBoundary::AuthoritativeExecution)
        .evidence_strength(worth_foundational::FoundationalPerformanceEvidenceStrength::CounterBackedExecutionReceipt)
        .breadth_locality(worth_foundational::FoundationalPerformanceBreadthLocalityPosture::PointLocal)
        .access_pattern(worth_foundational::FoundationalPerformanceAccessPatternPosture::PointLookup)
        .execution_temperature(worth_foundational::FoundationalPerformanceExecutionTemperature::HotPath)
        .freshness_retention(worth_foundational::FoundationalPerformanceFreshnessRetentionPosture::ExactBasisCurrent)
        .fallback_debt(worth_foundational::FoundationalPerformanceFallbackDebtPosture::Verified)
        .include_work(work_class)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ForensicParity);
    if let Some(context) = context {
        builder = builder.observation_context(context);
    }
    builder.finish()
}

pub(crate) fn claim_with_excluded_work(
    work_class: FoundationalPerformanceWorkClass,
) -> Result<
    worth_foundational::FoundationalAuthoritativePerformanceClaim,
    FoundationalPerformanceClaimConstructionDenial,
> {
    performance()
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
        .exclude_work(work_class)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ReplayReconstruction)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::SupportReportAssembly)
        .exclude_work(worth_foundational::FoundationalPerformanceWorkClass::ForensicParity)
        .finish()
}

pub(crate) fn active_context(
    disposition: FoundationalObservationDisposition,
) -> FoundationalPerformanceObservationContext {
    let profile = profiles()
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
        .expect("context profile");
    let admitted = match admit_requested_foundational_profile(
        request_foundational_profile_set(profile),
        profile,
        None,
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        other => panic!("expected admitted profile, got {other:?}"),
    };
    let identity = match derive_foundational_profile_identity(
        worth_foundational::CanonicalizationRuleVersion::new("m10.observation").expect("version"),
        &admitted,
    ) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        other => panic!("expected identity, got {other:?}"),
    };
    worth_foundational::FoundationalPerformanceObservationContext::new(identity, disposition)
}

pub(crate) fn report_profile() -> worth_foundational::FoundationalProfileSet {
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
        .expect("report profile")
}