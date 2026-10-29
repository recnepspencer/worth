use worth_foundational::{
    boundary_evidence,
    boundary_evidence_api::{common_path, lower_lane},
    evaluate_boundary_evidence_primitive_legality,
    foundational_boundary_evidence_category_definitions,
    foundational_boundary_evidence_descriptive_role_definitions,
    foundational_boundary_evidence_execution_posture_definitions,
    foundational_boundary_evidence_freshness_posture_definitions,
    foundational_boundary_evidence_locality_definitions, foundational_responsibilities,
    FoundationalBoundaryEvidenceCategory, FoundationalBoundaryEvidenceDescriptiveRole,
    FoundationalBoundaryEvidenceExecutionPosture, FoundationalBoundaryEvidenceFreshnessPosture,
    FoundationalBoundaryEvidenceLocality, FoundationalBoundaryEvidencePrimitiveLegalityDenial,
};

#[test]
fn boundary_evidence_responsibility_home_is_named_in_the_facade_topology() {
    let names: Vec<_> = foundational_responsibilities()
        .iter()
        .map(|area| area.name())
        .collect();

    assert_eq!(
        names,
        vec![
            "canonical_values",
            "aspect_state_and_patches",
            "identity_categories",
            "locators",
            "compatibility_bridges",
            "canonical_ordering_and_equality",
            "profiles",
            "boundary_artifacts",
            "transitions",
            "diagnostics",
            "boundary_evidence",
            "performance",
            "budget_limits",
            "expressions",
        ]
    );
}

#[test]
fn category_and_locality_definitions_are_blind_consumer_interpretable() {
    let categories = foundational_boundary_evidence_category_definitions();
    let localities = foundational_boundary_evidence_locality_definitions();

    assert_eq!(
        categories
            .iter()
            .map(|definition| definition.name())
            .collect::<Vec<_>>(),
        vec!["lineage", "provenance", "receipt", "support_truth"]
    );
    assert_eq!(
        localities
            .iter()
            .map(|definition| definition.name())
            .collect::<Vec<_>>(),
        vec![
            "current",
            "branch_local",
            "historical",
            "comparison_paired",
            "snapshot_bound",
            "replay_derived",
            "restored_readmitted",
        ]
    );

    assert!(categories
        .iter()
        .all(|definition| !definition.intended_use().trim().is_empty()));
    assert!(categories
        .iter()
        .all(|definition| !definition.must_not_mean().trim().is_empty()));
    assert!(localities
        .iter()
        .all(|definition| !definition.intended_use().trim().is_empty()));
}

#[test]
fn primitive_families_preserve_deterministic_ordering() {
    let mut categories = vec![
        FoundationalBoundaryEvidenceCategory::SupportTruth,
        FoundationalBoundaryEvidenceCategory::Lineage,
        FoundationalBoundaryEvidenceCategory::Receipt,
    ];
    categories.sort();
    assert_eq!(
        categories,
        vec![
            FoundationalBoundaryEvidenceCategory::Lineage,
            FoundationalBoundaryEvidenceCategory::Receipt,
            FoundationalBoundaryEvidenceCategory::SupportTruth,
        ]
    );

    let mut localities = vec![
        FoundationalBoundaryEvidenceLocality::ReplayDerived,
        FoundationalBoundaryEvidenceLocality::Current,
        FoundationalBoundaryEvidenceLocality::Historical,
    ];
    localities.sort();
    assert_eq!(
        localities,
        vec![
            FoundationalBoundaryEvidenceLocality::Current,
            FoundationalBoundaryEvidenceLocality::Historical,
            FoundationalBoundaryEvidenceLocality::ReplayDerived,
        ]
    );

    let mut freshness = vec![
        FoundationalBoundaryEvidenceFreshnessPosture::ReducedRetained,
        FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
        FoundationalBoundaryEvidenceFreshnessPosture::RestoredFromCheckpoint,
    ];
    freshness.sort();
    assert_eq!(
        freshness,
        vec![
            FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
            FoundationalBoundaryEvidenceFreshnessPosture::ReducedRetained,
            FoundationalBoundaryEvidenceFreshnessPosture::RestoredFromCheckpoint,
        ]
    );
}

#[test]
fn primitive_legality_keeps_only_the_phase1_minimum_floor() {
    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            FoundationalBoundaryEvidenceCategory::SupportTruth,
            FoundationalBoundaryEvidenceLocality::Historical,
            FoundationalBoundaryEvidenceExecutionPosture::Executed,
            FoundationalBoundaryEvidenceDescriptiveRole::AuthorityAdjacentDescription,
            FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
        ),
        Err(
            FoundationalBoundaryEvidencePrimitiveLegalityDenial::SupportTruthRequiresSupportGradeRole
        )
    );
    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            FoundationalBoundaryEvidenceCategory::Lineage,
            FoundationalBoundaryEvidenceLocality::Current,
            FoundationalBoundaryEvidenceExecutionPosture::Executed,
            FoundationalBoundaryEvidenceDescriptiveRole::SupportGrade,
            FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
        ),
        Err(
            FoundationalBoundaryEvidencePrimitiveLegalityDenial::NonSupportTruthMustNotClaimSupportGradeRole
        )
    );
    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            FoundationalBoundaryEvidenceCategory::Provenance,
            FoundationalBoundaryEvidenceLocality::ReplayDerived,
            FoundationalBoundaryEvidenceExecutionPosture::Planned,
            FoundationalBoundaryEvidenceDescriptiveRole::AuthorityAdjacentDescription,
            FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
        ),
        Ok(())
    );
    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            FoundationalBoundaryEvidenceCategory::Receipt,
            FoundationalBoundaryEvidenceLocality::RestoredReadmitted,
            FoundationalBoundaryEvidenceExecutionPosture::Planned,
            FoundationalBoundaryEvidenceDescriptiveRole::AuthorityAdjacentDescription,
            FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
        ),
        Ok(())
    );

    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            FoundationalBoundaryEvidenceCategory::SupportTruth,
            FoundationalBoundaryEvidenceLocality::ReplayDerived,
            FoundationalBoundaryEvidenceExecutionPosture::Executed,
            FoundationalBoundaryEvidenceDescriptiveRole::SupportGrade,
            FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay,
        ),
        Ok(())
    );

    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            FoundationalBoundaryEvidenceCategory::Lineage,
            FoundationalBoundaryEvidenceLocality::Current,
            FoundationalBoundaryEvidenceExecutionPosture::Planned,
            FoundationalBoundaryEvidenceDescriptiveRole::AuthorityAdjacentDescription,
            FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
        ),
        Ok(())
    );
}

#[test]
fn common_path_and_lower_lane_expose_the_same_phase1_surface() {
    assert_eq!(
        boundary_evidence().category_definitions(),
        common_path::boundary_evidence().category_definitions()
    );
    assert_eq!(
        boundary_evidence().locality_definitions(),
        lower_lane::primitives::foundational_boundary_evidence_locality_definitions()
    );
    assert_eq!(
        boundary_evidence().execution_posture_definitions(),
        foundational_boundary_evidence_execution_posture_definitions()
    );
    assert_eq!(
        boundary_evidence().descriptive_role_definitions(),
        foundational_boundary_evidence_descriptive_role_definitions()
    );
    assert_eq!(
        boundary_evidence().freshness_posture_definitions(),
        foundational_boundary_evidence_freshness_posture_definitions()
    );
}
