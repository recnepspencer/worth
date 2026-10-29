use {
    worth_foundational::boundary_evidence, worth_foundational::boundary_evidence_api::common_path,
    worth_foundational::boundary_evidence_api::lower_lane,
    worth_foundational::evaluate_boundary_evidence_primitive_legality,
    worth_foundational::foundational_boundary_evidence_category_definitions,
    worth_foundational::foundational_boundary_evidence_descriptive_role_definitions,
    worth_foundational::foundational_boundary_evidence_execution_posture_definitions,
    worth_foundational::foundational_boundary_evidence_freshness_posture_definitions,
    worth_foundational::foundational_boundary_evidence_locality_definitions,
    worth_foundational::foundational_responsibilities,
    worth_foundational::FoundationalBoundaryEvidenceCategory,
    worth_foundational::FoundationalBoundaryEvidenceDescriptiveRole,
    worth_foundational::FoundationalBoundaryEvidenceExecutionPosture,
    worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture,
    worth_foundational::FoundationalBoundaryEvidenceLocality,
    worth_foundational::FoundationalBoundaryEvidencePrimitiveLegalityDenial,
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
        worth_foundational::FoundationalBoundaryEvidenceCategory::SupportTruth,
        worth_foundational::FoundationalBoundaryEvidenceCategory::Lineage,
        worth_foundational::FoundationalBoundaryEvidenceCategory::Receipt,
    ];
    categories.sort();
    assert_eq!(
        categories,
        vec![
            worth_foundational::FoundationalBoundaryEvidenceCategory::Lineage,
            worth_foundational::FoundationalBoundaryEvidenceCategory::Receipt,
            worth_foundational::FoundationalBoundaryEvidenceCategory::SupportTruth,
        ]
    );

    let mut localities = vec![
        worth_foundational::FoundationalBoundaryEvidenceLocality::ReplayDerived,
        worth_foundational::FoundationalBoundaryEvidenceLocality::Current,
        worth_foundational::FoundationalBoundaryEvidenceLocality::Historical,
    ];
    localities.sort();
    assert_eq!(
        localities,
        vec![
            worth_foundational::FoundationalBoundaryEvidenceLocality::Current,
            worth_foundational::FoundationalBoundaryEvidenceLocality::Historical,
            worth_foundational::FoundationalBoundaryEvidenceLocality::ReplayDerived,
        ]
    );

    let mut freshness = vec![
        worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReducedRetained,
        worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
        worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::RestoredFromCheckpoint,
    ];
    freshness.sort();
    assert_eq!(
        freshness,
        vec![
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReducedRetained,
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::RestoredFromCheckpoint,
        ]
    );
}

#[test]
fn primitive_legality_keeps_only_the_minimum_floor() {
    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            worth_foundational::FoundationalBoundaryEvidenceCategory::SupportTruth,
            worth_foundational::FoundationalBoundaryEvidenceLocality::Historical,
            worth_foundational::FoundationalBoundaryEvidenceExecutionPosture::Executed,
            worth_foundational::FoundationalBoundaryEvidenceDescriptiveRole::AuthorityAdjacentDescription,
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
        ),
        Err(
            worth_foundational::FoundationalBoundaryEvidencePrimitiveLegalityDenial::SupportTruthRequiresSupportGradeRole
        )
    );
    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            worth_foundational::FoundationalBoundaryEvidenceCategory::Lineage,
            worth_foundational::FoundationalBoundaryEvidenceLocality::Current,
            worth_foundational::FoundationalBoundaryEvidenceExecutionPosture::Executed,
            worth_foundational::FoundationalBoundaryEvidenceDescriptiveRole::SupportGrade,
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
        ),
        Err(
            worth_foundational::FoundationalBoundaryEvidencePrimitiveLegalityDenial::NonSupportTruthMustNotClaimSupportGradeRole
        )
    );
    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            worth_foundational::FoundationalBoundaryEvidenceCategory::Provenance,
            worth_foundational::FoundationalBoundaryEvidenceLocality::ReplayDerived,
            worth_foundational::FoundationalBoundaryEvidenceExecutionPosture::Planned,
            worth_foundational::FoundationalBoundaryEvidenceDescriptiveRole::AuthorityAdjacentDescription,
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
        ),
        Ok(())
    );
    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            worth_foundational::FoundationalBoundaryEvidenceCategory::Receipt,
            worth_foundational::FoundationalBoundaryEvidenceLocality::RestoredReadmitted,
            worth_foundational::FoundationalBoundaryEvidenceExecutionPosture::Planned,
            worth_foundational::FoundationalBoundaryEvidenceDescriptiveRole::AuthorityAdjacentDescription,
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::StaleRetained,
        ),
        Ok(())
    );

    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            worth_foundational::FoundationalBoundaryEvidenceCategory::SupportTruth,
            worth_foundational::FoundationalBoundaryEvidenceLocality::ReplayDerived,
            worth_foundational::FoundationalBoundaryEvidenceExecutionPosture::Executed,
            worth_foundational::FoundationalBoundaryEvidenceDescriptiveRole::SupportGrade,
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::ReconstructedFromReplay,
        ),
        Ok(())
    );

    assert_eq!(
        evaluate_boundary_evidence_primitive_legality(
            worth_foundational::FoundationalBoundaryEvidenceCategory::Lineage,
            worth_foundational::FoundationalBoundaryEvidenceLocality::Current,
            worth_foundational::FoundationalBoundaryEvidenceExecutionPosture::Planned,
            worth_foundational::FoundationalBoundaryEvidenceDescriptiveRole::AuthorityAdjacentDescription,
            worth_foundational::FoundationalBoundaryEvidenceFreshnessPosture::FreshRetained,
        ),
        Ok(())
    );
}

#[test]
fn common_path_and_lower_lane_expose_the_same_surface() {
    assert_eq!(
        boundary_evidence().category_definitions(),
        worth_foundational::boundary_evidence_api::common_path::boundary_evidence()
            .category_definitions()
    );
    assert_eq!(
        boundary_evidence().locality_definitions(),
        worth_foundational::boundary_evidence_api::lower_lane::primitives::foundational_boundary_evidence_locality_definitions()
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
