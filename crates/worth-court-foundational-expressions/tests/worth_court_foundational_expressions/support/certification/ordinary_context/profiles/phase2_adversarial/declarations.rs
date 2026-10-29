pub(crate) use {worth_foundational::admit_requested_foundational_profile_with_resolutions, worth_foundational::compare_foundational_profiles, worth_foundational::foundational_profile_progression_authority, worth_foundational::profiles, worth_foundational::request_foundational_profile_set, worth_foundational::AdmissionReadinessProfile, worth_foundational::CertificationPostureProfile, worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::ExecutionObjectiveProfile, worth_foundational::FoundationalProfileCompatibilityClass, worth_foundational::FoundationalProfileFrontDoorConstructionDenial, worth_foundational::FoundationalProfileFrontDoorFamily, worth_foundational::FoundationalProfileResolutionLedger, worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput, worth_foundational::ObservationActivationProfile, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};
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

pub(crate) fn profile_identity(
    profile: FoundationalProfileSet,
) -> worth_foundational::FoundationalProfileIdentity {
    let admitted = match admit_requested_foundational_profile_with_resolutions(
        request_foundational_profile_set(profile),
        profile,
        worth_foundational::FoundationalProfileResolutionLedger::empty(),
        foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(artifact) => artifact,
        other => panic!("expected admitted profile, got {other:?}"),
    };
    match worth_foundational::derive_foundational_profile_identity(
        worth_foundational::CanonicalizationRuleVersion::new("m10.phase2.identity")
            .expect("identity rule"),
        &admitted,
    ) {
        worth_proof::TransitionOutcome::Success(identity) => identity,
        other => panic!("expected identity, got {other:?}"),
    }
}

pub(crate) fn canonical_value_debug(profile: FoundationalProfileSet, locus: &str) -> String {
    let identity = profile_identity(profile);
    let entry = identity
        .basis()
        .entries()
        .iter()
        .find(|entry| {
            entry.locus()
                == &worth_foundational::CanonicalBasisLocus::Named(locus.to_string().into())
        })
        .expect("canonical profile locus");
    format!("{:?}", entry.value())
}