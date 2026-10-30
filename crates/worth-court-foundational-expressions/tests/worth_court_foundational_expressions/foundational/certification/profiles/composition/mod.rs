#[allow(unused_imports)]
use crate::certification::ordinary_context::profiles::composition::*;
use {
    worth_foundational::foundational_responsibilities,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalProfileCompositionDenial,
    worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

#[test]
fn profile_responsibility_home_is_named_in_the_facade_topology() {
    let names: Vec<_> = foundational_responsibilities()
        .iter()
        .map(|area| area.name())
        .collect();

    assert_eq!(
        names,
        vec![
            "canonical_values",
            "aspect_state_and_patches",
            "identity_categories",
            "locators",
            "compatibility_bridges",
            "canonical_ordering_and_equality",
            "profiles",
            "boundary_artifacts",
            "transitions",
            "diagnostics",
            "boundary_evidence",
            "performance",
            "budget_limits",
            "expressions",
        ]
    );
}

#[test]
fn coherent_profile_sets_require_one_explicit_value_per_family() {
    let set = coherent_profile_set();

    assert_eq!(
        set.diagnostic_richness(),
        worth_foundational::DiagnosticRichnessProfile::Standard
    );
    assert_eq!(
        set.support_posture(),
        worth_foundational::SupportPostureProfile::CertificationReady
    );
    assert_eq!(
        set.compatibility_posture(),
        worth_foundational::CompatibilityPostureProfile::CompatibilityLowered
    );
    assert_eq!(
        set.admission_readiness(),
        worth_foundational::AdmissionReadinessProfile::ProductionGateReady
    );
    assert_eq!(
        set.retention_delivery(),
        worth_foundational::RetentionDeliveryProfile::Retained
    );
    assert_eq!(
        set.certification_posture(),
        worth_foundational::CertificationPostureProfile::ProductionCertified
    );
}

#[test]
fn profile_composition_denies_certified_posture_without_required_support_or_retention() {
    let internal_only =
        worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
            diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Forensic,
            support_posture: worth_foundational::SupportPostureProfile::InternalOnly,
            compatibility_posture: worth_foundational::CompatibilityPostureProfile::NativeOnly,
            admission_readiness: worth_foundational::AdmissionReadinessProfile::Admitted,
            retention_delivery: worth_foundational::RetentionDeliveryProfile::Retained,
            certification_posture: worth_foundational::CertificationPostureProfile::EvidenceBacked,
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
            observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
        });
    assert_eq!(
        internal_only,
        Err(worth_foundational::FoundationalProfileCompositionDenial::InternalSupportCannotClaimCertifiedPosture)
    );

    let readiness_too_weak =
        worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
            diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Forensic,
            support_posture: worth_foundational::SupportPostureProfile::SupportReady,
            compatibility_posture: worth_foundational::CompatibilityPostureProfile::NativeOnly,
            admission_readiness: worth_foundational::AdmissionReadinessProfile::CandidateOnly,
            retention_delivery: worth_foundational::RetentionDeliveryProfile::Retained,
            certification_posture: worth_foundational::CertificationPostureProfile::EvidenceBacked,
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
            observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
        });
    assert_eq!(
        readiness_too_weak,
        Err(worth_foundational::FoundationalProfileCompositionDenial::EvidenceBackedRequiresAdmittedReadiness)
    );

    let retention_too_weak_for_evidence =
        worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
            diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Standard,
            support_posture: worth_foundational::SupportPostureProfile::SupportReady,
            compatibility_posture:
                worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
            admission_readiness: worth_foundational::AdmissionReadinessProfile::Admitted,
            retention_delivery: worth_foundational::RetentionDeliveryProfile::Ephemeral,
            certification_posture: worth_foundational::CertificationPostureProfile::EvidenceBacked,
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
            observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
        });
    assert_eq!(
        retention_too_weak_for_evidence,
        Err(worth_foundational::FoundationalProfileCompositionDenial::EvidenceBackedRequiresRetainedDelivery)
    );

    let support_too_weak =
        worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
            diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Standard,
            support_posture: worth_foundational::SupportPostureProfile::SupportReady,
            compatibility_posture:
                worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
            admission_readiness: worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
            retention_delivery: worth_foundational::RetentionDeliveryProfile::Durable,
            certification_posture:
                worth_foundational::CertificationPostureProfile::ProductionCertified,
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
            observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
        });
    assert_eq!(
        support_too_weak,
        Err(
            worth_foundational::FoundationalProfileCompositionDenial::ProductionCertifiedRequiresCertificationReadySupport
        )
    );

    let readiness_too_weak_for_production =
        worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
            diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Standard,
            support_posture: worth_foundational::SupportPostureProfile::CertificationReady,
            compatibility_posture:
                worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
            admission_readiness: worth_foundational::AdmissionReadinessProfile::Admitted,
            retention_delivery: worth_foundational::RetentionDeliveryProfile::Retained,
            certification_posture:
                worth_foundational::CertificationPostureProfile::ProductionCertified,
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
            observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
        });
    assert_eq!(
        readiness_too_weak_for_production,
        Err(worth_foundational::FoundationalProfileCompositionDenial::ProductionCertifiedRequiresProductionGateReadiness)
    );

    let retention_too_weak =
        worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
            diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Standard,
            support_posture: worth_foundational::SupportPostureProfile::CertificationReady,
            compatibility_posture:
                worth_foundational::CompatibilityPostureProfile::CompatibilityRequired,
            admission_readiness: worth_foundational::AdmissionReadinessProfile::ProductionGateReady,
            retention_delivery: worth_foundational::RetentionDeliveryProfile::Ephemeral,
            certification_posture:
                worth_foundational::CertificationPostureProfile::ProductionCertified,
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
            observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
        });
    assert_eq!(
        retention_too_weak,
        Err(worth_foundational::FoundationalProfileCompositionDenial::ProductionCertifiedRequiresRetainedDelivery)
    );
}

#[test]
fn independently_constructed_profile_sets_compare_equal_when_meaning_matches() {
    let direct = coherent_profile_set();
    let reconstructed = worth_foundational::FoundationalProfileSet::new(coherent_profile_input())
        .expect("same meaning");

    assert_eq!(direct, reconstructed);
}
