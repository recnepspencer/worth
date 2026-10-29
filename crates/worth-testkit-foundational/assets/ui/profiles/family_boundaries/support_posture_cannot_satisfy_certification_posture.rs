use worth_foundational::{
    CertificationPostureProfile, FoundationalProfileSetInput, SupportPostureProfile,
};

fn requires_certification(_: CertificationPostureProfile) {}

fn main() {
    requires_certification(SupportPostureProfile::CertificationReady);

    let _ = FoundationalProfileSetInput {
        diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: worth_foundational::CompatibilityPostureProfile::NativeOnly,
        admission_readiness: worth_foundational::AdmissionReadinessProfile::Admitted,
        retention_delivery: worth_foundational::RetentionDeliveryProfile::Retained,
        certification_posture: SupportPostureProfile::CertificationReady,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    };
}
