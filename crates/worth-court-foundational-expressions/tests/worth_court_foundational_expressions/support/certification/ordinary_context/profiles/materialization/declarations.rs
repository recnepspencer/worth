pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::boundary_artifact_surface_inventory,
    worth_foundational::foundational_profile_applicability,
    worth_foundational::plan_foundational_profile_materialization,
    worth_foundational::plan_foundational_profile_materialization_with_elision,
    worth_foundational::plan_selected_foundational_profile_materialization,
    worth_foundational::proof_bearing_artifact_surface_inventory,
    worth_foundational::support_artifact_surface_inventory,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::BoundaryArtifactTarget,
    worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalDescriptiveElisionProfile,
    worth_foundational::FoundationalDescriptiveSurface,
    worth_foundational::FoundationalMaterializationPlanningDenial,
    worth_foundational::FoundationalProfileDecisionKind,
    worth_foundational::FoundationalProfileFamily,
    worth_foundational::FoundationalSurfaceAbsenceCause,
    worth_foundational::MaterializedFoundationalProfileSet,
    worth_foundational::ProofBearingArtifactTarget, worth_foundational::RetentionDeliveryProfile,
    worth_foundational::SupportArtifactTarget, worth_foundational::SupportPostureProfile,
};

pub(crate) use {
    crate::certification::ordinary_context::profiles::support::admit_same_profile,
    crate::certification::ordinary_context::profiles::support::profile,
};

pub(crate) fn materialized_profile(
    materialized: worth_foundational::FoundationalProfileSet,
) -> MaterializedFoundationalProfileSet {
    let admitted = admit_same_profile(*materialized.admitted_or_self());

    match worth_foundational::materialize_admitted_foundational_profile(
        admitted,
        materialized,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(materialized) => *materialized.payload(),
        outcome => panic!("expected materialized profile, got {outcome:?}"),
    }
}

pub(crate) trait MaterializedSeed {
    fn admitted_or_self(&self) -> &worth_foundational::FoundationalProfileSet;
}

impl MaterializedSeed for worth_foundational::FoundationalProfileSet {
    fn admitted_or_self(&self) -> &worth_foundational::FoundationalProfileSet {
        self
    }
}
