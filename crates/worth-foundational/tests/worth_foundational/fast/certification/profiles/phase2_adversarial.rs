use worth_foundational::{
    admit_requested_foundational_profile_with_resolutions, compare_foundational_profiles,
    foundational_profile_progression_authority, profiles, request_foundational_profile_set,
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, ExecutionObjectiveProfile, FoundationalProfileCompatibilityClass,
    FoundationalProfileFrontDoorConstructionDenial, FoundationalProfileFrontDoorFamily,
    FoundationalProfileResolutionLedger, FoundationalProfileSet, FoundationalProfileSetInput,
    ObservationActivationProfile, RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

fn base_profile(
    objective: ExecutionObjectiveProfile,
    activation: ObservationActivationProfile,
) -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::NativeOnly,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::Uncertified,
        execution_objective: objective,
        observation_activation: activation,
    })
    .expect("phase 2 profile axes compose")
}

fn profile_with_axes(
    profile: FoundationalProfileSet,
    objective: ExecutionObjectiveProfile,
    activation: ObservationActivationProfile,
) -> FoundationalProfileSet {
    FoundationalProfileSet::new(FoundationalProfileSetInput {
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

fn profile_identity(
    profile: FoundationalProfileSet,
) -> worth_foundational::FoundationalProfileIdentity {
    let admitted = match admit_requested_foundational_profile_with_resolutions(
        request_foundational_profile_set(profile),
        profile,
        FoundationalProfileResolutionLedger::empty(),
        foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(artifact) => artifact,
        other => panic!("expected admitted profile, got {other:?}"),
    };
    match worth_foundational::derive_foundational_profile_identity(
        worth_foundational::CanonicalizationRuleVersion::new("m10.phase2.identity")
            .expect("identity rule"),
        &admitted,
    ) {
        TransitionOutcome::Success(identity) => identity,
        other => panic!("expected identity, got {other:?}"),
    }
}

fn canonical_value_debug(profile: FoundationalProfileSet, locus: &str) -> String {
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

#[test]
fn objective_and_activation_form_an_orthogonal_three_by_two_matrix() {
    for objective in [
        ExecutionObjectiveProfile::LatencyBounded,
        ExecutionObjectiveProfile::Balanced,
        ExecutionObjectiveProfile::Throughput,
    ] {
        for activation in [
            ObservationActivationProfile::Continuous,
            ObservationActivationProfile::OnDemand,
        ] {
            let profile = base_profile(objective, activation);
            assert_eq!(profile.execution_objective(), objective);
            assert_eq!(profile.observation_activation(), activation);
        }
    }
}

#[test]
fn each_new_axis_has_its_own_difference_class_and_canonical_token() {
    let base = base_profile(
        ExecutionObjectiveProfile::Balanced,
        ObservationActivationProfile::Continuous,
    );
    let objective = profile_with_axes(
        base,
        ExecutionObjectiveProfile::Throughput,
        ObservationActivationProfile::Continuous,
    );
    let activation = profile_with_axes(
        base,
        ExecutionObjectiveProfile::Balanced,
        ObservationActivationProfile::OnDemand,
    );
    let both = profile_with_axes(
        base,
        ExecutionObjectiveProfile::Throughput,
        ObservationActivationProfile::OnDemand,
    );

    assert_eq!(
        compare_foundational_profiles(base, objective).compatibility_class(),
        FoundationalProfileCompatibilityClass::ExecutionObjectiveChange
    );
    assert_eq!(
        compare_foundational_profiles(base, activation).compatibility_class(),
        FoundationalProfileCompatibilityClass::ObservationActivationChange
    );
    assert_eq!(
        compare_foundational_profiles(base, both).compatibility_class(),
        FoundationalProfileCompatibilityClass::Incompatible
    );
    assert_ne!(profile_identity(base), profile_identity(objective));
    assert_ne!(profile_identity(base), profile_identity(activation));

    for (objective, token) in [
        (ExecutionObjectiveProfile::LatencyBounded, "latency-bounded"),
        (ExecutionObjectiveProfile::Balanced, "balanced"),
        (ExecutionObjectiveProfile::Throughput, "throughput"),
    ] {
        assert_eq!(
            canonical_value_debug(
                profile_with_axes(base, objective, ObservationActivationProfile::Continuous,),
                "execution_objective",
            ),
            format!("ExactText(Raw(\"{token}\"))")
        );
    }
    for (activation, token) in [
        (ObservationActivationProfile::Continuous, "continuous"),
        (ObservationActivationProfile::OnDemand, "on-demand"),
    ] {
        assert_eq!(
            canonical_value_debug(
                profile_with_axes(base, ExecutionObjectiveProfile::Balanced, activation),
                "observation_activation",
            ),
            format!("ExactText(Raw(\"{token}\"))")
        );
    }
}

#[test]
fn common_profile_front_door_rejects_duplicate_family_assignment() {
    let result = profiles()
        .set()
        .support_posture(SupportPostureProfile::SupportReady)
        .support_posture(SupportPostureProfile::SupportReady)
        .compose();
    assert_eq!(
        result,
        Err(
            FoundationalProfileFrontDoorConstructionDenial::DuplicateFamilyAssignment(
                FoundationalProfileFrontDoorFamily::SupportPosture,
            )
        )
    );
}
