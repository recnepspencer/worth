use worth_foundational::{
    foundational_responsibilities, AdmissionReadinessProfile, CertificationPostureProfile,
    CompatibilityPostureProfile, DiagnosticRichnessProfile, FoundationalProfileCompositionDenial,
    FoundationalProfileSet, FoundationalProfileSetInput, RetentionDeliveryProfile,
    SupportPostureProfile,
};

fn coherent_profile_input() -> FoundationalProfileSetInput {
    FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::CertificationReady,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityLowered,
        admission_readiness: AdmissionReadinessProfile::ProductionGateReady,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::ProductionCertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    }
}

fn coherent_profile_set() -> FoundationalProfileSet {
    FoundationalProfileSet::new(coherent_profile_input()).expect("coherent profile set")
}

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
        ]
    );
}

#[test]
fn coherent_profile_sets_require_one_explicit_value_per_family() {
    let set = coherent_profile_set();

    assert_eq!(
        set.diagnostic_richness(),
        DiagnosticRichnessProfile::Standard
    );
    assert_eq!(
        set.support_posture(),
        SupportPostureProfile::CertificationReady
    );
    assert_eq!(
        set.compatibility_posture(),
        CompatibilityPostureProfile::CompatibilityLowered
    );
    assert_eq!(
        set.admission_readiness(),
        AdmissionReadinessProfile::ProductionGateReady
    );
    assert_eq!(set.retention_delivery(), RetentionDeliveryProfile::Retained);
    assert_eq!(
        set.certification_posture(),
        CertificationPostureProfile::ProductionCertified
    );
}

#[test]
fn profile_composition_denies_certified_posture_without_required_support_or_retention() {
    let internal_only = FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Forensic,
        support_posture: SupportPostureProfile::InternalOnly,
        compatibility_posture: CompatibilityPostureProfile::NativeOnly,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::EvidenceBacked,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    });
    assert_eq!(
        internal_only,
        Err(FoundationalProfileCompositionDenial::InternalSupportCannotClaimCertifiedPosture)
    );

    let readiness_too_weak = FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Forensic,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::NativeOnly,
        admission_readiness: AdmissionReadinessProfile::CandidateOnly,
        retention_delivery: RetentionDeliveryProfile::Retained,
        certification_posture: CertificationPostureProfile::EvidenceBacked,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    });
    assert_eq!(
        readiness_too_weak,
        Err(FoundationalProfileCompositionDenial::EvidenceBackedRequiresAdmittedReadiness)
    );

    let retention_too_weak_for_evidence =
        FoundationalProfileSet::new(FoundationalProfileSetInput {
            diagnostic_richness: DiagnosticRichnessProfile::Standard,
            support_posture: SupportPostureProfile::SupportReady,
            compatibility_posture: CompatibilityPostureProfile::CompatibilityLowered,
            admission_readiness: AdmissionReadinessProfile::Admitted,
            retention_delivery: RetentionDeliveryProfile::Ephemeral,
            certification_posture: CertificationPostureProfile::EvidenceBacked,
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
            observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
        });
    assert_eq!(
        retention_too_weak_for_evidence,
        Err(FoundationalProfileCompositionDenial::EvidenceBackedRequiresRetainedDelivery)
    );

    let support_too_weak = FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::SupportReady,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityRequired,
        admission_readiness: AdmissionReadinessProfile::ProductionGateReady,
        retention_delivery: RetentionDeliveryProfile::Durable,
        certification_posture: CertificationPostureProfile::ProductionCertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    });
    assert_eq!(
        support_too_weak,
        Err(
            FoundationalProfileCompositionDenial::ProductionCertifiedRequiresCertificationReadySupport
        )
    );

    let readiness_too_weak_for_production =
        FoundationalProfileSet::new(FoundationalProfileSetInput {
            diagnostic_richness: DiagnosticRichnessProfile::Standard,
            support_posture: SupportPostureProfile::CertificationReady,
            compatibility_posture: CompatibilityPostureProfile::CompatibilityRequired,
            admission_readiness: AdmissionReadinessProfile::Admitted,
            retention_delivery: RetentionDeliveryProfile::Retained,
            certification_posture: CertificationPostureProfile::ProductionCertified,
            execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
            observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
        });
    assert_eq!(
        readiness_too_weak_for_production,
        Err(FoundationalProfileCompositionDenial::ProductionCertifiedRequiresProductionGateReadiness)
    );

    let retention_too_weak = FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::CertificationReady,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityRequired,
        admission_readiness: AdmissionReadinessProfile::ProductionGateReady,
        retention_delivery: RetentionDeliveryProfile::Ephemeral,
        certification_posture: CertificationPostureProfile::ProductionCertified,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    });
    assert_eq!(
        retention_too_weak,
        Err(FoundationalProfileCompositionDenial::ProductionCertifiedRequiresRetainedDelivery)
    );
}

#[test]
fn independently_constructed_profile_sets_compare_equal_when_meaning_matches() {
    let direct = coherent_profile_set();
    let reconstructed =
        FoundationalProfileSet::new(coherent_profile_input()).expect("same meaning");

    assert_eq!(direct, reconstructed);
}
