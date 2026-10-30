pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {
    worth_foundational::claim_derived_projection_boundary_surface,
    worth_foundational::compare_canonical_basis,
    worth_foundational::materialize_admitted_foundational_profile,
    worth_foundational::materialize_descriptive_boundary_surface,
    worth_foundational::prepare_canonical_comparison,
    worth_foundational::request_foundational_profile_set,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::CanonicalComparisonOutcome,
    worth_foundational::CanonicalEquivalenceBasis, worth_foundational::CanonicalizationRuleVersion,
    worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalBoundaryArtifactSurface,
    worth_foundational::FoundationalMaterializedBoundaryArtifact,
    worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

pub(crate) fn version(name: &str) -> CanonicalizationRuleVersion {
    worth_foundational::CanonicalizationRuleVersion::new(name)
        .expect("valid canonicalization version")
}

pub(crate) fn materialized_profile(
    richness: DiagnosticRichnessProfile,
    support: SupportPostureProfile,
    compatibility: CompatibilityPostureProfile,
    readiness: AdmissionReadinessProfile,
    retention: RetentionDeliveryProfile,
    certification: CertificationPostureProfile,
) -> worth_foundational::MaterializedFoundationalProfileSet {
    let profile = worth_foundational::FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: richness,
        support_posture: support,
        compatibility_posture: compatibility,
        admission_readiness: readiness,
        retention_delivery: retention,
        certification_posture: certification,
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
        outcome => panic!("expected admitted profile, got {outcome:?}"),
    };

    match materialize_admitted_foundational_profile(
        admitted,
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(materialized) => *materialized.payload(),
        outcome => panic!("expected materialized profile, got {outcome:?}"),
    }
}

pub(crate) fn exact_compare(
    left: worth_foundational::CanonicalBasisReadyArtifact,
    right: worth_foundational::CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(
        worth_foundational::CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };

    compare_canonical_basis(&ready)
}

pub(crate) fn materialize_projection_artifact(
    payload: Vec<u8>,
    attachment_slots: usize,
    profile: worth_foundational::MaterializedFoundationalProfileSet,
) -> FoundationalMaterializedBoundaryArtifact<FoundationalBoundaryArtifactSurface<Vec<u8>>> {
    materialize_descriptive_boundary_surface(
        claim_derived_projection_boundary_surface(
            worth_foundational::FoundationalBoundaryArtifactSurface::new(payload, attachment_slots),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("projection artifact materialized")
}
