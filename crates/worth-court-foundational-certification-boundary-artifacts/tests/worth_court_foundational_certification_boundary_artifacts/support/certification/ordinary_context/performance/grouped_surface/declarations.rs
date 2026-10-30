pub(crate) use {
    worth_foundational::performance_api::common_path,
    worth_foundational::performance_api::lower_lane::basis,
    worth_foundational::performance_api::lower_lane::receipts,
    worth_foundational::performance_api::lower_lane::reports,
    worth_foundational::performance_api::stronger_lane::certified,
    worth_foundational::performance_api::stronger_lane::readiness as stronger_readiness,
    worth_foundational::profiles, worth_foundational::CertificationPostureProfile,
    worth_foundational::DiagnosticRichnessProfile,
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
