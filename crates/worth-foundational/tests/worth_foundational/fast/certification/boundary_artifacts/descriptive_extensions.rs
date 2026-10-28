use worth_foundational::{
    admit_planned_work_boundary_artifact, admit_same_family_boundary_artifact,
    claim_derived_projection_boundary_surface, claim_planned_work_boundary_surface,
    derive_same_family_boundary_identity,
    evaluate_planned_work_reserved_authority_transition_legality,
    evaluate_same_family_reserved_authority_transition_legality,
    materialize_admitted_foundational_profile, materialize_descriptive_boundary_surface,
    request_foundational_profile_set, AdmissionReadinessProfile, CanonicalizationRuleVersion,
    CertificationPostureProfile, CompatibilityPostureProfile, DiagnosticRichnessProfile,
    FoundationalBoundaryArtifactSurface, FoundationalBoundaryMaterializationSeam,
    FoundationalBoundaryMaterializationSource, FoundationalPlannedWorkBoundaryArtifactDenial,
    FoundationalProfileSet, FoundationalProfileSetInput,
    FoundationalReservedAuthorityTransitionDenial, FoundationalReservedAuthorityTransitionKind,
    FoundationalSameFamilyBoundaryArtifactDenial, FoundationalSameFamilyBoundaryFamily,
    FoundationalSameFamilyBoundaryFamilyDenial, RetentionDeliveryProfile, SupportPostureProfile,
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
fn planned_work_and_same_family_descriptive_artifacts_stay_out_of_authority_transition_ontology() {
    let profile = materialized_profile();
    let materialized = materialize_descriptive_boundary_surface(
        claim_planned_work_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![1_u8, 2, 3],
            2,
        )),
        FoundationalBoundaryMaterializationSource::DerivedSupport,
        FoundationalBoundaryMaterializationSeam::SupportMaterialization,
        profile,
    )
    .expect_err("planned-work materialization stays unavailable");
    assert_eq!(
        materialized,
        worth_foundational::FoundationalBoundaryMaterializationDenial::SurfaceUnavailable
    );

    let descriptive_profile = materialized_profile();
    let planned_plan = worth_foundational::plan_descriptive_boundary_materialization(
        claim_planned_work_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![9_u8, 8, 7],
            2,
        )),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        descriptive_profile,
    )
    .expect("planned plan");
    let planned_artifact =
        admit_planned_work_boundary_artifact(planned_plan.materialize().expect("present planned"))
            .expect("planned wrapper");
    let same_family = admit_same_family_boundary_artifact(
        materialize_descriptive_boundary_surface(
            claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
                vec![9_u8, 8, 7],
                2,
            )),
            FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            descriptive_profile,
        )
        .expect("derived artifact"),
        FoundationalSameFamilyBoundaryFamily::new("routing.summary")
            .expect("valid same-family symbol"),
    )
    .expect("same-family wrapper");

    assert_eq!(
        evaluate_planned_work_reserved_authority_transition_legality(
            &planned_artifact,
            FoundationalReservedAuthorityTransitionKind::Commit,
        ),
        Err(
            FoundationalReservedAuthorityTransitionDenial::PlannedWorkMustRemainDescriptive {
                attempted: FoundationalReservedAuthorityTransitionKind::Commit,
            }
        )
    );
    assert_eq!(
        evaluate_same_family_reserved_authority_transition_legality(
            &same_family,
            FoundationalReservedAuthorityTransitionKind::Merge,
        ),
        Err(
            FoundationalReservedAuthorityTransitionDenial::SameFamilyMustRemainDescriptive {
                attempted: FoundationalReservedAuthorityTransitionKind::Merge,
            }
        )
    );
}

#[test]
fn same_family_identity_is_stable_across_independent_producers_and_family_validation_is_explicit() {
    assert_eq!(
        FoundationalSameFamilyBoundaryFamily::new(""),
        Err(FoundationalSameFamilyBoundaryFamilyDenial::FamilyMustNotBeBlank)
    );
    assert_eq!(
        FoundationalSameFamilyBoundaryFamily::new("routing summary"),
        Err(FoundationalSameFamilyBoundaryFamilyDenial::FamilyMustNotContainWhitespace)
    );

    let profile = materialized_profile();
    let left = admit_same_family_boundary_artifact(
        materialize_descriptive_boundary_surface(
            claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
                vec![4_u8, 5, 6],
                2,
            )),
            FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("left artifact"),
        FoundationalSameFamilyBoundaryFamily::new("routing.summary")
            .expect("valid same-family symbol"),
    )
    .expect("left wrapper");
    let right = admit_same_family_boundary_artifact(
        materialize_descriptive_boundary_surface(
            claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
                vec![4_u8, 5, 6],
                2,
            )),
            FoundationalBoundaryMaterializationSource::CompatibilityLowered,
            FoundationalBoundaryMaterializationSeam::BoundaryExchange,
            profile,
        )
        .expect("right artifact"),
        FoundationalSameFamilyBoundaryFamily::new("routing.summary")
            .expect("valid same-family symbol"),
    )
    .expect("right wrapper");

    let left_identity =
        match derive_same_family_boundary_identity(version("m4.phase5.same_family"), &left) {
            TransitionOutcome::Success(identity) => identity,
            _ => panic!("expected same-family identity"),
        };
    let right_identity =
        match derive_same_family_boundary_identity(version("m4.phase5.same_family"), &right) {
            TransitionOutcome::Success(identity) => identity,
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
        claim_derived_projection_boundary_surface(FoundationalBoundaryArtifactSurface::new(
            vec![1_u8],
            1,
        )),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("derived artifact");
    assert_eq!(
        admit_planned_work_boundary_artifact(derived_artifact),
        Err(FoundationalPlannedWorkBoundaryArtifactDenial::BoundaryRoleMustBePlannedWork)
    );

    let receipt_artifact = materialize_descriptive_boundary_surface(
        worth_foundational::claim_receipt_evidence_boundary_surface(
            worth_foundational::FoundationalBoundaryReceiptSurface::new("done", 1)
                .expect("receipt"),
        ),
        FoundationalBoundaryMaterializationSource::CompatibilityLowered,
        FoundationalBoundaryMaterializationSeam::BoundaryExchange,
        profile,
    )
    .expect("receipt artifact");
    assert_eq!(
        admit_same_family_boundary_artifact(
            receipt_artifact,
            FoundationalSameFamilyBoundaryFamily::new("routing.summary")
                .expect("valid same-family symbol"),
        ),
        Err(FoundationalSameFamilyBoundaryArtifactDenial::BoundaryRoleMustRemainDescriptive)
    );
}
