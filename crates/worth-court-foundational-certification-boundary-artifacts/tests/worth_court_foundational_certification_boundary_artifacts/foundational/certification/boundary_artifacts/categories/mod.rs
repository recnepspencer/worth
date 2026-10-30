use {
    worth_foundational::boundary_artifact_category_definitions,
    worth_foundational::boundary_artifact_category_of,
    worth_foundational::boundary_receipt_category_of,
    worth_foundational::boundary_summary_category_of,
    worth_foundational::foundational_responsibilities,
    worth_foundational::FoundationalBoundaryCategorySurface,
    worth_foundational::FoundationalBoundaryReportSurface,
};

#[test]
fn boundary_artifact_responsibility_home_is_named_in_the_facade_topology() {
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
fn category_definitions_are_blind_consumer_interpretable() {
    let definitions = boundary_artifact_category_definitions();
    let names: Vec<_> = definitions
        .iter()
        .map(|definition| definition.name())
        .collect();

    assert_eq!(names, vec!["summary", "report", "artifact", "receipt"]);
    assert!(definitions
        .iter()
        .all(|definition| !definition.intended_use().trim().is_empty()));
    assert!(definitions
        .iter()
        .all(|definition| definition.must_not_mean().contains("report")
            || definition.must_not_mean().contains("summary")
            || definition.must_not_mean().contains("artifact")
            || definition.must_not_mean().contains("receipt")));
}

#[test]
fn category_local_surfaces_are_mutually_non_substitutable() {
    let summary =
        worth_foundational::FoundationalBoundarySummarySurface::new("compact exchange overview", 2)
            .expect("summary");
    let report =
        worth_foundational::FoundationalBoundaryReportSurface::new(vec!["row-1", "row-2"], 1)
            .expect("report");
    let artifact =
        worth_foundational::FoundationalBoundaryArtifactSurface::new(vec![1_u8, 2, 3], 3);
    let receipt =
        worth_foundational::FoundationalBoundaryReceiptSurface::new("commit persisted", 1)
            .expect("receipt");

    assert_eq!(
        boundary_summary_category_of(&summary),
        worth_foundational::FoundationalBoundaryArtifactCategory::Summary
    );
    assert_eq!(
        boundary_artifact_category_of(&report),
        worth_foundational::FoundationalBoundaryArtifactCategory::Report
    );
    assert_eq!(
        boundary_artifact_category_of(&artifact),
        worth_foundational::FoundationalBoundaryArtifactCategory::Artifact
    );
    assert_eq!(
        boundary_receipt_category_of(&receipt),
        worth_foundational::FoundationalBoundaryArtifactCategory::Receipt
    );

    assert_ne!(summary.definition(), report.definition());
    assert_ne!(report.definition(), artifact.definition());
    assert_ne!(artifact.definition(), receipt.definition());
}

#[test]
fn category_specific_construction_denials_are_explicit() {
    assert_eq!(
        worth_foundational::FoundationalBoundarySummarySurface::new("   ", 1),
        Err(worth_foundational::FoundationalBoundaryCategoryConstructionDenial::SummaryRequiresOverviewText)
    );
    assert_eq!(
        FoundationalBoundaryReportSurface::<&str>::new(Vec::new(), 1),
        Err(worth_foundational::FoundationalBoundaryCategoryConstructionDenial::ReportRequiresAtLeastOneRow)
    );
    assert_eq!(
        worth_foundational::FoundationalBoundaryReceiptSurface::new("", 1),
        Err(
            worth_foundational::FoundationalBoundaryCategoryConstructionDenial::ReceiptRequiresCompletedBoundaryDescription
        )
    );
}
