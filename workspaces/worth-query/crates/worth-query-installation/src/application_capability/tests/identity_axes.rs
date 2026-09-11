use super::*;
use worth_foundational::facade::{
    compare_canonical_basis, prepare_canonical_comparison, CanonicalComparisonOutcome,
    CanonicalEquivalenceBasis,
};

pub(super) fn grouped_graph_rule(conjunctive: bool) -> ApplicationCapabilityGraphRule {
    let first = ApplicationCapabilityGraphClause::new(graph_path(true, "PrincipalResource"));
    let second =
        ApplicationCapabilityGraphClause::new(graph_path(true, "ChangedPrincipalResource"));
    if conjunctive {
        ApplicationCapabilityGraphRule::all([
            ApplicationCapabilityGraphRequirement::any([first]),
            ApplicationCapabilityGraphRequirement::any([second]),
        ])
    } else {
        ApplicationCapabilityGraphRule::any([first, second])
    }
}

pub(super) fn anchored_graph_rule() -> ApplicationCapabilityGraphRule {
    let relation = ApplicationRelationRef::<Schema, PrincipalResource, Principal, Resource>::
        from_schema_identifiers("PrincipalResource", "Principal", "Resource",
                worth_query_declaration::facade::application_schema::ApplicationRelationIntegrity::same_context_unbounded_retain_dangling(),
            );
    let slot = ApplicationCapabilityContextEntitySlotRef::<
        Schema,
        Context,
        ResourceSlot,
        Resource,
    >::from_declaration();
    ApplicationCapabilityGraphRule::any([ApplicationCapabilityGraphClause::new(graph_path(
        true,
        "PrincipalResource",
    ))
    .anchored([ApplicationCapabilityPathContextAnchor::after_forward(
        relation, slot,
    )])])
}

#[test]
fn every_scope_and_composition_axis_changes_structured_and_digest_identity() {
    let package = worth_foundational::facade::CanonicalDigestId::new([1; 32]);
    let schema = worth_foundational::facade::CanonicalDigestId::new([2; 32]);
    let baseline = prepare_capability_basis(&package, &schema, &contract(None)).unwrap();
    let axes = [
        Axis::Action,
        Axis::Resource,
        Axis::Relation,
        Axis::Field,
        Axis::Purpose,
        Axis::Magnitude,
        Axis::Cardinality,
        Axis::Workflow,
        Axis::ResourceWorkflow,
        Axis::Status,
        Axis::ValidityTimeline,
        Axis::Validity,
        Axis::Delegation,
        Axis::DelegationDepth,
        Axis::Provenance,
        Axis::Context,
        Axis::Rule(0),
        Axis::Rule(1),
        Axis::Rule(2),
        Axis::Rule(3),
        Axis::Rule(4),
        Axis::Rule(5),
        Axis::Rule(6),
        Axis::ContextAnchor,
    ];
    for axis in axes {
        let changed = prepare_capability_basis(&package, &schema, &contract(Some(axis))).unwrap();
        assert_distinct(&baseline, &changed);
    }
}

#[test]
fn context_and_provenance_marker_types_are_identity_bearing() {
    let package = worth_foundational::facade::CanonicalDigestId::new([1; 32]);
    let schema = worth_foundational::facade::CanonicalDigestId::new([2; 32]);
    let baseline = contract_with_markers(
        ApplicationCapabilityContextRef::<Schema, Context>::from_declaration(),
        ApplicationCapabilityProvenanceRef::<Schema, Provenance>::from_declaration(),
    );
    let changed_context = contract_with_markers(
        ApplicationCapabilityContextRef::<Schema, OtherContext>::from_declaration(),
        ApplicationCapabilityProvenanceRef::<Schema, Provenance>::from_declaration(),
    );
    let changed_provenance = contract_with_markers(
        ApplicationCapabilityContextRef::<Schema, Context>::from_declaration(),
        ApplicationCapabilityProvenanceRef::<Schema, OtherProvenance>::from_declaration(),
    );
    let baseline = prepare_capability_basis(&package, &schema, &baseline).unwrap();
    for changed in [changed_context, changed_provenance] {
        let changed = prepare_capability_basis(&package, &schema, &changed).unwrap();
        assert_distinct(&baseline, &changed);
    }
}

#[test]
fn boolean_grouping_is_identity_bearing_with_the_same_leaf_paths() {
    let package = worth_foundational::facade::CanonicalDigestId::new([1; 32]);
    let schema = worth_foundational::facade::CanonicalDigestId::new([2; 32]);
    let alternatives = prepare_capability_basis(
        &package,
        &schema,
        &contract(Some(Axis::AlternativeGrouping)),
    )
    .unwrap();
    let conjunction = prepare_capability_basis(
        &package,
        &schema,
        &contract(Some(Axis::ConjunctiveGrouping)),
    )
    .unwrap();
    assert_distinct(&alternatives, &conjunction);
}

fn assert_distinct(
    baseline: &super::super::WorthQueryCapabilityCanonicalArtifact,
    changed: &super::super::WorthQueryCapabilityCanonicalArtifact,
) {
    assert_ne!(baseline.digest(), changed.digest());
    assert!(matches!(
        compare(baseline.basis(), changed.basis()),
        CanonicalComparisonOutcome::Mismatched(_)
    ));
}

fn compare(
    left: &worth_foundational::facade::CanonicalBasisReadyArtifact,
    right: &worth_foundational::facade::CanonicalBasisReadyArtifact,
) -> CanonicalComparisonOutcome {
    let ready = prepare_canonical_comparison(
        CanonicalEquivalenceBasis::ExactCanonicalBasis,
        left.clone(),
        right.clone(),
    )
    .into_result()
    .expect("capability basis comparison is supported");
    compare_canonical_basis(&ready)
}
