#[allow(unused_imports)] use crate::certification::ordinary_context::boundary_artifacts::descriptive_extensions::*;
use {worth_foundational::admit_planned_work_boundary_artifact, worth_foundational::admit_same_family_boundary_artifact, worth_foundational::claim_derived_projection_boundary_surface, worth_foundational::claim_planned_work_boundary_surface, worth_foundational::derive_same_family_boundary_identity, worth_foundational::evaluate_planned_work_reserved_authority_transition_legality, worth_foundational::evaluate_same_family_reserved_authority_transition_legality, worth_foundational::materialize_admitted_foundational_profile, worth_foundational::materialize_descriptive_boundary_surface, worth_foundational::request_foundational_profile_set, worth_foundational::AdmissionReadinessProfile, worth_foundational::CanonicalizationRuleVersion, worth_foundational::CertificationPostureProfile, worth_foundational::CompatibilityPostureProfile, worth_foundational::DiagnosticRichnessProfile, worth_foundational::FoundationalBoundaryArtifactSurface, worth_foundational::FoundationalBoundaryMaterializationSeam, worth_foundational::FoundationalBoundaryMaterializationSource, worth_foundational::FoundationalPlannedWorkBoundaryArtifactDenial, worth_foundational::FoundationalProfileSet, worth_foundational::FoundationalProfileSetInput, worth_foundational::FoundationalReservedAuthorityTransitionDenial, worth_foundational::FoundationalReservedAuthorityTransitionKind, worth_foundational::FoundationalSameFamilyBoundaryArtifactDenial, worth_foundational::FoundationalSameFamilyBoundaryFamily, worth_foundational::FoundationalSameFamilyBoundaryFamilyDenial, worth_foundational::RetentionDeliveryProfile, worth_foundational::SupportPostureProfile};
use {worth_proof::TransitionOutcome};

#[test]
fn planned_work_and_same_family_descriptive_artifacts_stay_out_of_authority_transition_ontology() {
    let profile = materialized_profile();
    let materialized = materialize_descriptive_boundary_surface(
        claim_planned_work_boundary_surface(worth_foundational::FoundationalBoundaryArtifactSurface::new(
            vec![1_u8, 2, 3],
            2,
        )),
        worth_foundational::FoundationalBoundaryMaterializationSource::DerivedSupport,
        worth_foundational::FoundationalBoundaryMaterializationSeam::SupportMaterialization,
        profile,
    )
    .expect_err("planned-work materialization stays unavailable");
    assert_eq!(
        materialized,
        worth_foundational::FoundationalBoundaryMaterializationDenial::SurfaceUnavailable
    );

    let descriptive_profile = materialized_profile();
    let planned_plan = worth_foundational::plan_descriptive_boundary_materialization(
        claim_planned_work_boundary_surface(worth_foundational::FoundationalBoundaryArtifactSurface::new(
            vec![9_u8, 8, 7],
            2,
        )),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        descriptive_profile,
    )
    .expect("planned plan");
    let planned_artifact =
        admit_planned_work_boundary_artifact(planned_plan.materialize().expect("present planned"))
            .expect("planned wrapper");
    let same_family = admit_same_family_boundary_artifact(
        materialize_descriptive_boundary_surface(
            claim_derived_projection_boundary_surface(worth_foundational::FoundationalBoundaryArtifactSurface::new(
                vec![9_u8, 8, 7],
                2,
            )),
            worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            descriptive_profile,
        )
        .expect("derived artifact"),
        worth_foundational::FoundationalSameFamilyBoundaryFamily::new("routing.summary")
            .expect("valid same-family symbol"),
    )
    .expect("same-family wrapper");

    assert_eq!(
        evaluate_planned_work_reserved_authority_transition_legality(
            &planned_artifact,
            worth_foundational::FoundationalReservedAuthorityTransitionKind::Commit,
        ),
        Err(
            worth_foundational::FoundationalReservedAuthorityTransitionDenial::PlannedWorkMustRemainDescriptive {
                attempted: worth_foundational::FoundationalReservedAuthorityTransitionKind::Commit,
            }
        )
    );
    assert_eq!(
        evaluate_same_family_reserved_authority_transition_legality(
            &same_family,
            worth_foundational::FoundationalReservedAuthorityTransitionKind::Merge,
        ),
        Err(
            worth_foundational::FoundationalReservedAuthorityTransitionDenial::SameFamilyMustRemainDescriptive {
                attempted: worth_foundational::FoundationalReservedAuthorityTransitionKind::Merge,
            }
        )
    );
}

#[test]
fn same_family_identity_is_stable_across_independent_producers_and_family_validation_is_explicit() {
    assert_eq!(
        worth_foundational::FoundationalSameFamilyBoundaryFamily::new(""),
        Err(worth_foundational::FoundationalSameFamilyBoundaryFamilyDenial::FamilyMustNotBeBlank)
    );
    assert_eq!(
        worth_foundational::FoundationalSameFamilyBoundaryFamily::new("routing summary"),
        Err(worth_foundational::FoundationalSameFamilyBoundaryFamilyDenial::FamilyMustNotContainWhitespace)
    );

    let profile = materialized_profile();
    let left = admit_same_family_boundary_artifact(
        materialize_descriptive_boundary_surface(
            claim_derived_projection_boundary_surface(worth_foundational::FoundationalBoundaryArtifactSurface::new(
                vec![4_u8, 5, 6],
                2,
            )),
            worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("left artifact"),
        worth_foundational::FoundationalSameFamilyBoundaryFamily::new("routing.summary")
            .expect("valid same-family symbol"),
    )
    .expect("left wrapper");
    let right = admit_same_family_boundary_artifact(
        materialize_descriptive_boundary_surface(
            claim_derived_projection_boundary_surface(worth_foundational::FoundationalBoundaryArtifactSurface::new(
                vec![4_u8, 5, 6],
                2,
            )),
            worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("right artifact"),
        worth_foundational::FoundationalSameFamilyBoundaryFamily::new("routing.summary")
            .expect("valid same-family symbol"),
    )
    .expect("right wrapper");

    let left_identity =
        match derive_same_family_boundary_identity(version("m4.phase5.same_family"), &left) {
            worth_proof::TransitionOutcome::Success(identity) => identity,
            _ => panic!("expected same-family identity"),
        };
    let right_identity =
        match derive_same_family_boundary_identity(version("m4.phase5.same_family"), &right) {
            worth_proof::TransitionOutcome::Success(identity) => identity,
            _ => panic!("expected same-family identity"),
        };

    assert_eq!(left_identity, right_identity);
    assert_eq!(left_identity.family().as_str(), "routing.summary");
    assert_eq!(
        left_identity
            .basis()
            .entries()
            .first()
            .expect("family basis entry")
            .locus(),
        &worth_foundational::CanonicalBasisLocus::Named("same_family.family".into())
    );
}

#[test]
fn same_family_and_planned_work_admission_denials_are_explicit() {
    let profile = materialized_profile();
    let derived_artifact = materialize_descriptive_boundary_surface(
        claim_derived_projection_boundary_surface(worth_foundational::FoundationalBoundaryArtifactSurface::new(
            vec![1_u8],
            1,
        )),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("derived artifact");
    assert_eq!(
        admit_planned_work_boundary_artifact(derived_artifact),
        Err(worth_foundational::FoundationalPlannedWorkBoundaryArtifactDenial::BoundaryRoleMustBePlannedWork)
    );

    let receipt_artifact = materialize_descriptive_boundary_surface(
        worth_foundational::claim_receipt_evidence_boundary_surface(
            worth_foundational::FoundationalBoundaryReceiptSurface::new("done", 1)
                .expect("receipt"),
        ),
        worth_foundational::FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        worth_foundational::FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("receipt artifact");
    assert_eq!(
        admit_same_family_boundary_artifact(
            receipt_artifact,
            worth_foundational::FoundationalSameFamilyBoundaryFamily::new("routing.summary")
                .expect("valid same-family symbol"),
        ),
        Err(worth_foundational::FoundationalSameFamilyBoundaryArtifactDenial::BoundaryRoleMustRemainDescriptive)
    );
}
