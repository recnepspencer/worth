pub use worth_foundational::facade::{
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, ExecutionObjectiveProfile, FoundationalProfileNarrowingKind,
    FoundationalProfileNarrowingRecord, FoundationalProfileProgressionDenial,
    FoundationalProfileSet, FoundationalProfileSetInput, ObservationActivationProfile,
    RetentionDeliveryProfile, SupportPostureProfile,
};
pub use worth_query_publication::facade::domain_computation::{
    publish_application_authorization_denial, WorthQueryApplicationAuthorizationProfileStage,
    WorthQueryApplicationAuthorizationPublicationDenial,
    WorthQueryApplicationAuthorizationPublicationProfile,
};

pub use super::installed_composition::{real_denial, CompositionScenario};

pub fn stage(
    profile: FoundationalProfileSet,
    reason: &'static str,
) -> WorthQueryApplicationAuthorizationProfileStage {
    WorthQueryApplicationAuthorizationProfileStage::new(
        profile,
        Some(FoundationalProfileNarrowingRecord::new(
            FoundationalProfileNarrowingKind::RichnessReduced,
            reason,
        )),
    )
}

pub fn profile(richness: DiagnosticRichnessProfile) -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: richness,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityLowered,
        execution_objective: ExecutionObjectiveProfile::Balanced,
        observation_activation: ObservationActivationProfile::Continuous,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::EvidenceBacked,
    })
    .unwrap()
}
