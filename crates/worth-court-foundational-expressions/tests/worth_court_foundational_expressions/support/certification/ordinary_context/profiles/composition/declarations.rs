pub(crate) use {worth_foundational::foundational_responsibilities, worth_foundational::AdmissionReadinessProfile, worth_foundational::CertificationPostureProfile, worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::FoundationalProfileCompositionDenial, worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};

pub(crate) fn coherent_profile_input() -> FoundationalProfileSetInput {
    FoundationalProfileSetInput {
        diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Standard,
        support_posture: worth_foundational::SupportPostureProfile::CertificationReady,
        compatibility_posture: worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        admission_readiness: worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
        retention_delivery: worth_foundational::RetentionDeliveryProfile::Retained,
        certification_posture: worth_foundational::CertificationPostureProfile::ProductionCertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    }
}

pub(crate) fn coherent_profile_set() -> FoundationalProfileSet {
    worth_foundational::FoundationalProfileSet::new(coherent_profile_input()).expect("coherent profile set")
}