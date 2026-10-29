use worth_foundational::{
    admit_current_basis_boundary_artifact, admit_current_basis_boundary_bundle,
    bridge_current_basis_boundary_artifact_trust_boundary,
    bridge_current_basis_boundary_bundle_trust_boundary, claim_derived_projection_boundary_surface,
    foundational_boundary_current_basis_authority, foundational_boundary_current_basis_proof_lane,
    foundational_boundary_current_basis_readmission_authority,
    materialize_admitted_foundational_profile, plan_artifact_boundary_bundle,
    plan_descriptive_boundary_materialization,
    prepare_materialized_boundary_artifact_for_canonical_basis,
    readmit_current_basis_boundary_artifact_after_boundary,
    readmit_current_basis_boundary_bundle_after_boundary, request_foundational_profile_set,
    AdmissionReadinessProfile, CanonicalBasisDomain, CanonicalizationRuleVersion,
    CertificationPostureProfile, CompatibilityPostureProfile, DiagnosticRichnessProfile,
    FoundationalBoundaryArtifactSurface, FoundationalBoundaryCurrentBasisCertified,
    FoundationalBoundaryCurrentBasisProofLane, FoundationalBoundaryMaterializationSeam,
    FoundationalBoundaryMaterializationSource, FoundationalBoundarySummarySurface,
    FoundationalProfileSet, FoundationalProfileSetInput, RetentionDeliveryProfile,
    SupportPostureProfile,
};
use worth_proof::TransitionOutcome;

fn version(name: &str) -> CanonicalizationRuleVersion {
    CanonicalizationRuleVersion::new(name).expect("valid canonicalization version")
}

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
    .expect("coherent profile");
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

#[test]
fn current_basis_boundary_artifact_reuses_phase_4_basis_ready_artifact() {
    let profile = materialized_profile();
    let materialized = worth_foundational::materialize_descriptive_boundary_surface(
        claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![1_u8, 2, 3],
            2,
        )),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("materialized artifact");
    let expected_basis = match prepare_materialized_boundary_artifact_for_canonical_basis(
        version("m4.phase4_5.single"),
        &materialized,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected basis ready artifact"),
    };

    let strengthened = match admit_current_basis_boundary_artifact(
        version("m4.phase4_5.single"),
        materialized,
        foundational_boundary_current_basis_authority(),
    ) {
        TransitionOutcome::Success(strengthened) => strengthened,
        _ => panic!("expected strengthened artifact"),
    };

    assert_eq!(
        foundational_boundary_current_basis_proof_lane(),
        FoundationalBoundaryCurrentBasisProofLane::CurrentBasisArtifactWithBoundaryReadmission
    );
    assert_eq!(
        strengthened.strong_basis().payload().domain(),
        CanonicalBasisDomain::BoundaryArtifact
    );
    assert_eq!(
        strengthened.strong_basis().payload(),
        expected_basis.payload()
    );
    assert_eq!(
        strengthened.materialized().category(),
        worth_foundational::FoundationalBoundaryArtifactCategory::Artifact
    );
    accepts_current_basis_proof(strengthened.proofs());
}

#[test]
fn current_basis_boundary_artifact_and_bundle_require_explicit_readmission_after_boundary() {
    let profile = materialized_profile();
    let materialized = worth_foundational::materialize_descriptive_boundary_surface(
        claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![9_u8, 8, 7],
            2,
        )),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("materialized artifact");
    let rebound_basis = match prepare_materialized_boundary_artifact_for_canonical_basis(
        version("m4.phase4_5.readmit"),
        &materialized,
    ) {
        TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected basis ready artifact"),
    };
    let strengthened = match admit_current_basis_boundary_artifact(
        version("m4.phase4_5.readmit"),
        materialized,
        foundational_boundary_current_basis_authority(),
    ) {
        TransitionOutcome::Success(strengthened) => strengthened,
        _ => panic!("expected strengthened artifact"),
    };
    let readmitted = readmit_current_basis_boundary_artifact_after_boundary(
        bridge_current_basis_boundary_artifact_trust_boundary(strengthened),
        rebound_basis,
        foundational_boundary_current_basis_readmission_authority(),
    );
    assert_eq!(
        readmitted.strong_basis().payload().domain(),
        CanonicalBasisDomain::BoundaryArtifact
    );
    accepts_current_basis_proof(readmitted.proofs());

    let primary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![3_u8, 4, 5],
            2,
        )),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("primary plan");
    let summary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            FoundationalBoundarySummarySurface::new("summary basis", 1).expect("summary"),
        ),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("summary plan");
    let bundle = plan_artifact_boundary_bundle(primary)
        .with_summary(summary)
        .expect("summary legality")
        .materialize()
        .expect("bundle materialized");
    let bundle_basis =
        match worth_foundational::prepare_materialized_boundary_bundle_for_canonical_basis(
            version("m4.phase4_5.bundle"),
            &bundle,
        ) {
            TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected bundle basis ready artifact"),
        };
    let strengthened_bundle = match admit_current_basis_boundary_bundle(
        version("m4.phase4_5.bundle"),
        bundle,
        foundational_boundary_current_basis_authority(),
    ) {
        TransitionOutcome::Success(strengthened) => strengthened,
        _ => panic!("expected strengthened bundle"),
    };
    let readmitted_bundle = readmit_current_basis_boundary_bundle_after_boundary(
        bridge_current_basis_boundary_bundle_trust_boundary(strengthened_bundle),
        bundle_basis,
        foundational_boundary_current_basis_readmission_authority(),
    );
    assert_eq!(
        readmitted_bundle.strong_basis().payload().domain(),
        CanonicalBasisDomain::BoundaryArtifact
    );
    assert!(readmitted_bundle.bundle().summary().is_some());
    accepts_current_basis_proof(readmitted_bundle.proofs());
}

fn accepts_current_basis_proof(
    _: &worth_proof::Proof<
        FoundationalBoundaryCurrentBasisCertified,
        worth_foundational::FoundationalBoundaryCurrentBasisAuthority,
    >,
) {
}
