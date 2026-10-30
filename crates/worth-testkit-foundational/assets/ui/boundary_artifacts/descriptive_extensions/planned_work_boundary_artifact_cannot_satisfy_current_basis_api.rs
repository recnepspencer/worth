use worth_foundational::{
    admit_planned_work_boundary_artifact, claim_planned_work_boundary_surface,
    materialize_admitted_foundational_profile, materialize_descriptive_boundary_surface,
    request_foundational_profile_set, AdmissionReadinessProfile, CertificationPostureProfile,
    CompatibilityPostureProfile, CurrentBasisBoundaryArtifact, DiagnosticRichnessProfile,
    FoundationalBoundaryArtifactSurface, FoundationalBoundaryMaterializationSeam,
    FoundationalBoundaryMaterializationSource, FoundationalProfileSet, FoundationalProfileSetInput,
    RetentionDeliveryProfile, SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

fn materialized_profile() -> worth_foundational::MaterializedFoundationalProfileSet {
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

    match materialize_admitted_foundational_profile(
        admitted,
        profile,
        None,
        worth_foundational::foundational_profile_progression_authority(),
    ) {
        TransitionOutcome::Success(materialized) => *materialized.payload(),
        _ => panic!("expected materialized profile"),
    }
}

fn requires_current_basis(_: &CurrentBasisBoundaryArtifact<FoundationalBoundaryArtifactSurface<Vec<u8>>>) {}

fn main() {
    let planned = admit_planned_work_boundary_artifact(
        materialize_descriptive_boundary_surface(
            claim_planned_work_boundary_surface(FoundationalBoundaryArtifactSurface::new(
                vec![1_u8],
                1,
            )),
            FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            materialized_profile(),
        )
        .expect("planned materialized"),
    )
    .expect("planned wrapper");
    requires_current_basis(&planned);
}
