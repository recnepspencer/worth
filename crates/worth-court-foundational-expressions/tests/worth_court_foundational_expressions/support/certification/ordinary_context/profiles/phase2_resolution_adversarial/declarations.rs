pub(crate) use {worth_foundational::admit_requested_foundational_profile_with_resolutions, worth_foundational::foundational_profile_progression_authority, worth_foundational::materialize_admitted_foundational_profile_with_resolutions, worth_foundational::request_foundational_profile_set, worth_foundational::AdmissionReadinessProfile, worth_foundational::CertificationPostureProfile, worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::ExecutionObjectiveProfile, worth_foundational::FoundationalProfileProgressionDenial, worth_foundational::FoundationalProfileResolutionFamily, worth_foundational::FoundationalProfileResolutionLedger, worth_foundational::FoundationalProfileResolutionLedgerDenial, worth_foundational::FoundationalProfileResolutionRecord, worth_foundational::FoundationalProfileResolutionRelation, worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput, worth_foundational::ObservationActivationProfile, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};
pub(crate) use {worth_proof::TransitionOutcome};

pub(crate) fn base_profile(
    objective: ExecutionObjectiveProfile,
    activation: ObservationActivationProfile,
) -> FoundationalProfileSet {
    worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Standard,
        support_posture: worth_foundational::SupportPostureProfile::SupportReady,
        compatibility_posture: worth_foundational::CompatibilityPostureProfile::NativeOnly,
        admission_readiness: worth_foundational::AdmissionReadinessProfile::Admitted,
        retention_delivery: worth_foundational::RetentionDeliveryProfile::Retained,
        certification_posture: worth_foundational::CertificationPostureProfile::Uncertified,
        execution_objective: objective,
        observation_activation: activation,
    })
    .expect("phase 2 profile axes compose")
}

pub(crate) fn profile_with_axes(
    profile: FoundationalProfileSet,
    objective: ExecutionObjectiveProfile,
    activation: ObservationActivationProfile,
) -> FoundationalProfileSet {
    worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: profile.diagnostic_richness(),
        support_posture: profile.support_posture(),
        compatibility_posture: profile.compatibility_posture(),
        admission_readiness: profile.admission_readiness(),
        retention_delivery: profile.retention_delivery(),
        certification_posture: profile.certification_posture(),
        execution_objective: objective,
        observation_activation: activation,
    })
    .expect("phase 2 profile axes remain coherent")
}

pub(crate) fn resolution_ledger(objective: bool, activation: bool) -> FoundationalProfileResolutionLedger {
    let mut ledger = worth_foundational::FoundationalProfileResolutionLedger::empty();
    if objective {
        ledger
            .insert(worth_foundational::FoundationalProfileResolutionRecord::new(
                worth_foundational::FoundationalProfileResolutionFamily::ExecutionObjective,
                worth_foundational::FoundationalProfileResolutionRelation::ObjectiveSelection,
                "objective selected",
            ))
            .expect("objective family is unique");
    }
    if activation {
        ledger
            .insert(worth_foundational::FoundationalProfileResolutionRecord::new(
                worth_foundational::FoundationalProfileResolutionFamily::ObservationActivation,
                worth_foundational::FoundationalProfileResolutionRelation::ActivationSelection,
                "activation selected",
            ))
            .expect("activation family is unique");
    }
    ledger
}

pub(crate) fn admit_with_resolutions(
    requested: FoundationalProfileSet,
    admitted: FoundationalProfileSet,
    resolutions: FoundationalProfileResolutionLedger,
) -> worth_foundational::AdmittedFoundationalProfileArtifact {
    match admit_requested_foundational_profile_with_resolutions(
        request_foundational_profile_set(requested),
        admitted,
        resolutions,
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(artifact) => artifact,
        other => panic!("expected admitted profile, got {other:?}"),
    }
}