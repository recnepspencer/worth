pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::admit_planned_work_boundary_artifact,
    worth_foundational::admit_same_family_boundary_artifact,
    worth_foundational::claim_derived_projection_boundary_surface,
    worth_foundational::claim_planned_work_boundary_surface,
    worth_foundational::derive_same_family_boundary_identity,
    worth_foundational::evaluate_planned_work_reserved_authority_transition_legality,
    worth_foundational::evaluate_same_family_reserved_authority_transition_legality,
    worth_foundational::materialize_admitted_foundational_profile,
    worth_foundational::materialize_descriptive_boundary_surface,
    worth_foundational::request_foundational_profile_set,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::CanonicalizationRuleVersion,
    worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalBoundaryArtifactSurface,
    worth_foundational::FoundationalBoundaryMaterializationSeam,
    worth_foundational::FoundationalBoundaryMaterializationSource,
    worth_foundational::FoundationalPlannedWorkBoundaryArtifactDenial,
    worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput,
    worth_foundational::FoundationalReservedAuthorityTransitionDenial,
    worth_foundational::FoundationalReservedAuthorityTransitionKind,
    worth_foundational::FoundationalSameFamilyBoundaryArtifactDenial,
    worth_foundational::FoundationalSameFamilyBoundaryFamily,
    worth_foundational::FoundationalSameFamilyBoundaryFamilyDenial,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

pub(crate) fn version(name: &str) -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new(name)
        .expect("valid canonicalization version")
}

pub(crate) fn materialized_profile() -> worth_foundational::MaterializedFoundationalProfileSet {
    let profile = worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: worth_foundational::DiagnosticRichnessProfile::Standard,
        support_posture: worth_foundational::SupportPostureProfile::CertificationReady,
        compatibility_posture:
            worth_foundational::CompatibilityPostureProfile::CompatibilityLowered,
        admission_readiness: worth_foundational::AdmissionReadinessProfile::Admitted,
        retention_delivery: worth_foundational::RetentionDeliveryProfile::Durable,
        certification_posture: worth_foundational::CertificationPostureProfile::EvidenceBacked,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("coherent profile");
    let requested = request_foundational_profile_set(profile);
    let admitted = match worth_foundational::admit_requested_foundational_profile(
        requested,
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(admitted) => admitted,
        _ => panic!("expected admitted profile"),
    };

    match materialize_admitted_foundational_profile(
        admitted,
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(materialized) => *materialized.payload(),
        _ => panic!("expected materialized profile"),
    }
}
