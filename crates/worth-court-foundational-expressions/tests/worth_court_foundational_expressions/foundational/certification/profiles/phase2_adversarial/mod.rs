#[allow(unused_imports)]
use crate::certification::ordinary_context::profiles::phase2_adversarial::*;
use {worth_foundational::compare_foundational_profiles, worth_foundational::profiles};

#[test]
fn objective_and_activation_form_an_orthogonal_three_by_two_matrix() {
    for objective in [
        worth_foundational::ExecutionObjectiveProfile::LatencyBounded,
        worth_foundational::ExecutionObjectiveProfile::Balanced,
        worth_foundational::ExecutionObjectiveProfile::Throughput,
    ] {
        for activation in [
            worth_foundational::ObservationActivationProfile::Continuous,
            worth_foundational::ObservationActivationProfile::OnDemand,
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
        worth_foundational::ExecutionObjectiveProfile::Balanced,
        worth_foundational::ObservationActivationProfile::Continuous,
    );
    let objective = profile_with_axes(
        base,
        worth_foundational::ExecutionObjectiveProfile::Throughput,
        worth_foundational::ObservationActivationProfile::Continuous,
    );
    let activation = profile_with_axes(
        base,
        worth_foundational::ExecutionObjectiveProfile::Balanced,
        worth_foundational::ObservationActivationProfile::OnDemand,
    );
    let both = profile_with_axes(
        base,
        worth_foundational::ExecutionObjectiveProfile::Throughput,
        worth_foundational::ObservationActivationProfile::OnDemand,
    );

    assert_eq!(
        compare_foundational_profiles(base, objective).compatibility_class(),
        worth_foundational::FoundationalProfileCompatibilityClass::ExecutionObjectiveChange
    );
    assert_eq!(
        compare_foundational_profiles(base, activation).compatibility_class(),
        worth_foundational::FoundationalProfileCompatibilityClass::ObservationActivationChange
    );
    assert_eq!(
        compare_foundational_profiles(base, both).compatibility_class(),
        worth_foundational::FoundationalProfileCompatibilityClass::Incompatible
    );
    assert_ne!(profile_identity(base), profile_identity(objective));
    assert_ne!(profile_identity(base), profile_identity(activation));

    for (objective, token) in [
        (
            worth_foundational::ExecutionObjectiveProfile::LatencyBounded,
            "latency-bounded",
        ),
        (
            worth_foundational::ExecutionObjectiveProfile::Balanced,
            "balanced",
        ),
        (
            worth_foundational::ExecutionObjectiveProfile::Throughput,
            "throughput",
        ),
    ] {
        assert_eq!(
            canonical_value_debug(
                profile_with_axes(
                    base,
                    objective,
                    worth_foundational::ObservationActivationProfile::Continuous,
                ),
                "execution_objective",
            ),
            format!("ExactText(Raw(\"{token}\"))")
        );
    }
    for (activation, token) in [
        (
            worth_foundational::ObservationActivationProfile::Continuous,
            "continuous",
        ),
        (
            worth_foundational::ObservationActivationProfile::OnDemand,
            "on-demand",
        ),
    ] {
        assert_eq!(
            canonical_value_debug(
                profile_with_axes(
                    base,
                    worth_foundational::ExecutionObjectiveProfile::Balanced,
                    activation
                ),
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
        .support_posture(worth_foundational::SupportPostureProfile::SupportReady)
        .support_posture(worth_foundational::SupportPostureProfile::SupportReady)
        .compose();
    assert_eq!(
        result,
        Err(
            worth_foundational::FoundationalProfileFrontDoorConstructionDenial::DuplicateFamilyAssignment(
                worth_foundational::FoundationalProfileFrontDoorFamily::SupportPosture,
            )
        )
    );
}
