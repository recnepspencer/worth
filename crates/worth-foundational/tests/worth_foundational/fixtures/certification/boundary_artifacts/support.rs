use worth_foundational::{
    claim_derived_projection_boundary_surface, compare_canonical_basis,
    materialize_admitted_foundational_profile, materialize_descriptive_boundary_surface,
    prepare_canonical_comparison, request_foundational_profile_set, AdmissionReadinessProfile,
    CanonicalComparisonOutcome, CanonicalEquivalenceBasis, CanonicalizationRuleVersion,
    CertificationPostureProfile, CompatibilityPostureProfile, DiagnosticRichnessProfile,
    FoundationalBoundaryArtifactSurface, FoundationalMaterializedBoundaryArtifact,
    FoundationalProfileSet, FoundationalProfileSetInput, RetentionDeliveryProfile,
    SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

pub fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("valid canonicalization version")
}

pub fn materialized_profile(
    richness: DiagnosticRichnessProfile,
    support: SupportPostureProfile,
    compatibility: CompatibilityPostureProfile,
    readiness: AdmissionReadinessProfile,
    retention: RetentionDeliveryProfile,
    certification: CertificationPostureProfile,
) -> worth_foundational::MaterializedFoundationalProfileSet {
    let profile = FoundationalProfileSet::new(FoundationalProfileSetInput {
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
        TransitionOutcome::Success(admitted) => admitted,
        outcome => panic!("expected admitted profile, got {outcome:?}"),
    };

    match materialize_admitted_foundational_profile(
        admitted,
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(materialized) => *materialized.payload(),
        outcome => panic!("expected materialized profile, got {outcome:?}"),
    }
}

pub fn exact_compare(
    left: worth_foundational::CanonicalBasisReadyArtifact,
    right: worth_foundational::CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = match prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left,
        right,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected comparison readiness"),
    };

    compare_canonical_basis(&ready)
}

pub fn materialize_projection_artifact(
    payload: Vec<u8>,
    attachment_slots: usize,
    profile: worth_foundational::MaterializedFoundationalProfileSet,
) -> FoundationalMaterializedBoundaryArtifact<FoundationalBoundaryArtifactSurface<Vec<u8>>> {
    materialize_descriptive_boundary_surface(
        claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            payload,
            attachment_slots,
        )),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("projection artifact materialized")
}
