pub use worth_proof::TransitionOutcome;
pub use {
    worth_foundational::admit_requested_foundational_profile,
    worth_foundational::request_foundational_profile_set,
    worth_foundational::AdmissionReadinessProfile,
    worth_foundational::AdmittedFoundationalProfileArtifact,
    worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

pub fn profile(
    diagnostic_richness: DiagnosticRichnessProfile,
    support_posture: SupportPostureProfile,
    compatibility_posture: CompatibilityPostureProfile,
    admission_readiness: AdmissionReadinessProfile,
    retention_delivery: RetentionDeliveryProfile,
    certification_posture: CertificationPostureProfile,
) -> FoundationalProfileSet {
    worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness,
        support_posture,
        compatibility_posture,
        admission_readiness,
        retention_delivery,
        certification_posture,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("coherent profile")
}

pub fn admit_same_profile(profile: FoundationalProfileSet) -> AdmittedFoundationalProfileArtifact {
    match admit_requested_foundational_profile(
        request_foundational_profile_set(profile),
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("expected admitted profile, got {outcome:?}"),
    }
}
