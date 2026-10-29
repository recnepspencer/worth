#[allow(unused_imports)] use crate::certification::ordinary_context::profiles::milestone10_vocabulary::*;
use {worth_foundational::derive_foundational_profile_identity, worth_foundational::foundational_profile_progression_authority, worth_foundational::request_foundational_profile_set, worth_foundational::FoundationalProfileResolutionFamily, worth_foundational::FoundationalProfileResolutionLedger, worth_foundational::FoundationalProfileResolutionRecord, worth_foundational::FoundationalProfileResolutionRelation};
use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::profiles::support::admit_same_profile, crate::certification::ordinary_context::profiles::support::profile};

#[test]
fn objective_and_activation_have_independent_resolution_records() {
    let requested = profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::NativeOnly,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::Uncertified,
    );
    let admitted = worth_foundational::FoundationalProfileSet::new(
        worth_foundational::FoundationalProfileSetInput {
            diagnostic_richness: requested.diagnostic_richness(),
            support_posture: requested.support_posture(),
            compatibility_posture: requested.compatibility_posture(),
            admission_readiness: requested.admission_readiness(),
            retention_delivery: requested.retention_delivery(),
            certification_posture: requested.certification_posture(),
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Throughput,
            observation_activation: worth_foundational::ObservationActivationProfile::OnDemand,
        },
    )
    .expect("objective and activation profile should compose");

    let mut resolutions = worth_foundational::FoundationalProfileResolutionLedger::empty();
    resolutions
        .insert(worth_foundational::FoundationalProfileResolutionRecord::new(
            worth_foundational::FoundationalProfileResolutionFamily::ExecutionObjective,
            worth_foundational::FoundationalProfileResolutionRelation::ObjectiveSelection,
            "select throughput objective",
        ))
        .expect("objective family is unique");
    resolutions
        .insert(worth_foundational::FoundationalProfileResolutionRecord::new(
            worth_foundational::FoundationalProfileResolutionFamily::ObservationActivation,
            worth_foundational::FoundationalProfileResolutionRelation::ActivationSelection,
            "select on-demand activation",
        ))
        .expect("activation family is unique");

    let outcome = worth_foundational::admit_requested_foundational_profile_with_resolutions(
        request_foundational_profile_set(requested),
        admitted,
        resolutions,
        foundational_profile_progression_authority(),
    );
    let admitted = match outcome {
        worth_proof::TransitionOutcome::Success(artifact) => artifact,
        other => panic!("expected objective/activation admission, got {other:?}"),
    };
    assert_eq!(
        admitted.payload().requested_to_admitted_resolutions().len(),
        2
    );
    assert_eq!(
        admitted
            .payload()
            .requested_to_admitted_resolutions()
            .get(worth_foundational::FoundationalProfileResolutionFamily::ExecutionObjective)
            .expect("objective record")
            .relation(),
        worth_foundational::FoundationalProfileResolutionRelation::ObjectiveSelection
    );
}

#[test]
fn resolution_admission_reuses_monotonic_profile_progression_rules() {
    let requested = profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::NativeOnly,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::Uncertified,
    );
    let illegally_widened = profile(
        worth_foundational::DiagnosticRichnessProfile::Forensic,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::NativeOnly,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::Uncertified,
    );
    let mut resolutions = worth_foundational::FoundationalProfileResolutionLedger::empty();
    resolutions
        .insert(worth_foundational::FoundationalProfileResolutionRecord::new(
            worth_foundational::FoundationalProfileResolutionFamily::DiagnosticRichness,
            worth_foundational::FoundationalProfileResolutionRelation::Narrowing,
            "forged widening must not become an admitted resolution",
        ))
        .expect("diagnostic family is unique");

    let outcome = worth_foundational::admit_requested_foundational_profile_with_resolutions(
        request_foundational_profile_set(requested),
        illegally_widened,
        resolutions,
        foundational_profile_progression_authority(),
    );
    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::FoundationalProfileProgressionDenial::
                RequestedAndAdmittedProfilesMayOnlyNarrow,
        )
    );
}

#[test]
fn objective_and_activation_change_profile_identity_independently() {
    let balanced_continuous = profile(
        worth_foundational::DiagnosticRichnessProfile::Standard,
        worth_foundational::SupportPostureProfile::SupportReady,
        worth_foundational::CompatibilityPostureProfile::NativeOnly,
        worth_foundational::AdmissionReadinessProfile::Admitted,
        worth_foundational::RetentionDeliveryProfile::Retained,
        worth_foundational::CertificationPostureProfile::Uncertified,
    );
    let throughput_on_demand =
        worth_foundational::FoundationalProfileSet::new(worth_foundational_profile_input(
            balanced_continuous,
            worth_foundational::ExecutionObjectiveProfile::Throughput,
            worth_foundational::ObservationActivationProfile::OnDemand,
        ))
        .expect("objective and activation profile should compose");
    let first = admit_same_profile(balanced_continuous);
    let second = admit_same_profile(throughput_on_demand);
    assert_ne!(profile_identity(&first), profile_identity(&second));
}
