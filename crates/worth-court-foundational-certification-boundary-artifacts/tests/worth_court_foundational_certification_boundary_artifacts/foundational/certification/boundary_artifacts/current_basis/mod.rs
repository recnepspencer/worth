#[allow(unused_imports)]
use crate::certification::ordinary_context::boundary_artifacts::current_basis::*;
use worth_proof::TransitionOutcome;
use {
    worth_foundational::admit_current_basis_boundary_artifact,
    worth_foundational::admit_current_basis_boundary_bundle,
    worth_foundational::bridge_current_basis_boundary_artifact_trust_boundary,
    worth_foundational::bridge_current_basis_boundary_bundle_trust_boundary,
    worth_foundational::claim_derived_projection_boundary_surface,
    worth_foundational::foundational_boundary_current_basis_authority,
    worth_foundational::foundational_boundary_current_basis_proof_lane,
    worth_foundational::foundational_boundary_current_basis_readmission_authority,
    worth_foundational::materialize_admitted_foundational_profile,
    worth_foundational::plan_artifact_boundary_bundle,
    worth_foundational::plan_descriptive_boundary_materialization,
    worth_foundational::prepare_materialized_boundary_artifact_for_canonical_basis,
    worth_foundational::readmit_current_basis_boundary_artifact_after_boundary,
    worth_foundational::readmit_current_basis_boundary_bundle_after_boundary,
    worth_foundational::request_foundational_profile_set,
    worth_foundational::AdmissionReadinessProfile, worth_foundational::CanonicalBasisDomain,
    worth_foundational::CanonicalizationRuleVersion,
    worth_foundational::CertificationPostureProfile,
    worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile,
    worth_foundational::FoundationalBoundaryArtifactSurface,
    worth_foundational::FoundationalBoundaryCurrentBasisCertified,
    worth_foundational::FoundationalBoundaryCurrentBasisProofLane,
    worth_foundational::FoundationalBoundaryMaterializationSeam,
    worth_foundational::FoundationalBoundaryMaterializationSource,
    worth_foundational::FoundationalBoundarySummarySurface,
    worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput,
    worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile,
};

#[test]
fn current_basis_boundary_artifact_reuses_basis_ready_artifact() {
    let profile = materialized_profile();
    let materialized = worth_foundational::materialize_descriptive_boundary_surface(
        claim_derived_projection_boundary_surface(
            worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![1_u8, 2, 3], 2),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("materialized artifact");
    let expected_basis = match prepare_materialized_boundary_artifact_for_canonical_basis(
        version("m4.phase4_5.single"),
        &materialized,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected basis ready artifact"),
    };

    let strengthened = match admit_current_basis_boundary_artifact(
        version("m4.phase4_5.single"),
        materialized,
        foundational_boundary_current_basis_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(strengthened) => strengthened,
        _ => panic!("expected strengthened artifact"),
    };

    assert_eq!(
        foundational_boundary_current_basis_proof_lane(),
        worth_foundational::FoundationalBoundaryCurrentBasisProofLane::CurrentBasisArtifactWithBoundaryReadmission
    );
    assert_eq!(
        strengthened.strong_basis().payload().domain(),
        worth_foundational::CanonicalBasisDomain::BoundaryArtifact
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
        claim_derived_projection_boundary_surface(
            worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![9_u8, 8, 7], 2),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("materialized artifact");
    let rebound_basis = match prepare_materialized_boundary_artifact_for_canonical_basis(
        version("m4.phase4_5.readmit"),
        &materialized,
    ) {
        worth_proof::TransitionOutcome::Success(ready) => ready,
        _ => panic!("expected basis ready artifact"),
    };
    let strengthened = match admit_current_basis_boundary_artifact(
        version("m4.phase4_5.readmit"),
        materialized,
        foundational_boundary_current_basis_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(strengthened) => strengthened,
        _ => panic!("expected strengthened artifact"),
    };
    let readmitted = readmit_current_basis_boundary_artifact_after_boundary(
        bridge_current_basis_boundary_artifact_trust_boundary(strengthened),
        rebound_basis,
        foundational_boundary_current_basis_readmission_authority(),
    );
    assert_eq!(
        readmitted.strong_basis().payload().domain(),
        worth_foundational::CanonicalBasisDomain::BoundaryArtifact
    );
    accepts_current_basis_proof(readmitted.proofs());

    let primary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![3_u8, 4, 5], 2),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("primary plan");
    let summary = plan_descriptive_boundary_materialization(
        claim_derived_projection_boundary_surface(
            worth_foundational::FoundationalBoundarySummarySurface::new("summary basis", 1)
                .expect("summary"),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
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
            worth_proof::TransitionOutcome::Success(ready) => ready,
            _ => panic!("expected bundle basis ready artifact"),
        };
    let strengthened_bundle = match admit_current_basis_boundary_bundle(
        version("m4.phase4_5.bundle"),
        bundle,
        foundational_boundary_current_basis_authority(),
    ) {
        worth_proof::TransitionOutcome::Success(strengthened) => strengthened,
        _ => panic!("expected strengthened bundle"),
    };
    let readmitted_bundle = readmit_current_basis_boundary_bundle_after_boundary(
        bridge_current_basis_boundary_bundle_trust_boundary(strengthened_bundle),
        bundle_basis,
        foundational_boundary_current_basis_readmission_authority(),
    );
    assert_eq!(
        readmitted_bundle.strong_basis().payload().domain(),
        worth_foundational::CanonicalBasisDomain::BoundaryArtifact
    );
    assert!(readmitted_bundle.bundle().summary().is_some());
    accepts_current_basis_proof(readmitted_bundle.proofs());
}
