pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::derive_foundational_profile_identity,
    worth_foundational::foundational_profile_progression_authority,
    worth_foundational::request_foundational_profile_set,
    worth_foundational::FoundationalProfileResolutionFamily,
    worth_foundational::FoundationalProfileResolutionLedger,
    worth_foundational::FoundationalProfileResolutionRecord,
    worth_foundational::FoundationalProfileResolutionRelation,
};

pub(crate) use {
    crate::certification::ordinary_context::profiles::support::admit_same_profile,
    crate::certification::ordinary_context::profiles::support::profile,
};

pub(crate) fn profile_identity(
    admitted: &worth_foundational::AdmittedFoundationalProfileArtifact,
) -> worth_foundational::FoundationalProfileIdentity {
    match derive_foundational_profile_identity(
        worth_foundational::CanonicalizationRuleVersion::new("m10.identity")
            .expect("identity rule"),
        admitted,
    ) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        other => panic!("expected profile identity, got {other:?}"),
    }
}

pub(crate) fn worth_foundational_profile_input(
    profile: worth_foundational::FoundationalProfileSet,
    objective: worth_foundational::ExecutionObjectiveProfile,
    activation: worth_foundational::ObservationActivationProfile,
) -> worth_foundational::FoundationalProfileSetInput {
    worth_foundational::FoundationalProfileSetInput {
        diagnostic_richness: profile.diagnostic_richness(),
        support_posture: profile.support_posture(),
        compatibility_posture: profile.compatibility_posture(),
        admission_readiness: profile.admission_readiness(),
        retention_delivery: profile.retention_delivery(),
        certification_posture: profile.certification_posture(),
        execution_objective: objective,
        observation_activation: activation,
    }
}
