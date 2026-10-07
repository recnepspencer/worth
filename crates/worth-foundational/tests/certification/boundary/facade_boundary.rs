use worth_foundational::{
    declared_foundational_boundary, foundational_responsibilities, FoundationalBoundaryArtifact,
};

#[test]
fn facade_exposes_named_responsibility_topology() {
    let responsibilities = foundational_responsibilities();
    let names: Vec<_> = responsibilities.iter().map(|area| area.name()).collect();

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

    assert!(responsibilities
        .iter()
        .all(|area| !area.owns().contains("helper")));
    assert!(responsibilities
        .iter()
        .any(|area| area.does_not_own().contains("runtime storage")));
}

#[test]
fn boundary_declaration_is_a_worth_proof_artifact() {
    let artifact: FoundationalBoundaryArtifact = declared_foundational_boundary();
    let payload = artifact.payload();

    assert_eq!(payload.crate_name(), "worth-foundational");
    assert_eq!(payload.standardizes(), "shared boundary meaning");
    assert!(payload
        .does_not_standardize()
        .contains("proof progression law"));
}

#[test]
fn root_exports_are_curated_through_the_facade() {
    let root_artifact = worth_foundational::declared_foundational_boundary();
    let facade_artifact = worth_foundational::facade::declared_foundational_boundary();

    assert_eq!(root_artifact.payload(), facade_artifact.payload());
}
