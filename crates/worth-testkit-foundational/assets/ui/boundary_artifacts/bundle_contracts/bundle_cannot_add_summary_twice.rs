use worth_foundational::{
    claim_derived_projection_boundary_surface, plan_artifact_boundary_bundle,
    plan_descriptive_boundary_materialization, request_foundational_profile_set,
    AdmissionReadinessProfile, CertificationPostureProfile, CompatibilityPostureProfile,
    DiagnosticRichnessProfile, FoundationalBoundaryArtifactSurface,
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
        other => panic!("expected admitted profile, got {other:?}"),
    };
    let materialized = match worth_foundational::materialize_admitted_foundational_profile(
        admitted,
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(materialized) => *materialized.payload(),
        other => panic!("expected materialized profile, got {other:?}"),
    };
    let primary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![1_u8],
            1,
        )),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        materialized.clone(),
    )
    .expect("primary");
    let summary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            FoundationalBoundarySummarySurface::new("summary", 1).expect("summary"),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        materialized,
    )
    .expect("summary");

    let _ = plan_artifact_boundary_bundle(primary)
        .with_summary(summary.clone())
        .expect("first summary")
        .with_summary(summary)
        .expect("second summary");
}
