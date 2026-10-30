use worth_foundational::{
    claim_derived_projection_boundary_surface, foundational_boundary_canonical_basis_entries,
    materialize_admitted_foundational_profile, plan_artifact_boundary_bundle,
    plan_descriptive_boundary_materialization, request_foundational_profile_set,
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, FoundationalBoundaryArtifactSurface,
    FoundationalBoundaryMaterializationSeam, FoundationalBoundaryMaterializationSource,
    FoundationalBoundarySummarySurface, FoundationalProfileSet, FoundationalProfileSetInput,
    RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

fn main() {
    let profile = FoundationalProfileSet::new(FoundationalProfileSetInput {
        diagnostic_richness: DiagnosticRichnessProfile::Standard,
        support_posture: SupportPostureProfile::CertificationReady,
        compatibility_posture: CompatibilityPostureProfile::CompatibilityLowered,
        admission_readiness: AdmissionReadinessProfile::Admitted,
        retention_delivery: RetentionDeliveryProfile::Durable,
        certification_posture: CertificationPostureProfile::EvidenceBacked,
        execution_objective: worth_foundational::ExecutionObjectiveProfile::Balanced,
        observation_activation: worth_foundational::ObservationActivationProfile::Continuous,
    })
    .expect("profile");
    let requested = request_foundational_profile_set(profile);
    let admitted = match worth_foundational::admit_requested_foundational_profile(
        requested,
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(admitted) => admitted,
        _ => panic!("expected admitted profile"),
    };
    let materialized_profile = match materialize_admitted_foundational_profile(
        admitted,
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(materialized) => *materialized.payload(),
        _ => panic!("expected materialized profile"),
    };
    let primary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![1_u8, 2, 3],
            2,
        )),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        materialized_profile.clone(),
    )
    .expect("primary plan");
    let summary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            FoundationalBoundarySummarySurface::new("summary", 1).expect("summary"),
        ),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        materialized_profile,
    )
    .expect("summary plan");
    let bundle = plan_artifact_boundary_bundle(primary)
        .with_summary(summary)
        .expect("summary legality")
        .materialize()
        .expect("bundle");

    let _ = foundational_boundary_canonical_basis_entries(&bundle);
}
