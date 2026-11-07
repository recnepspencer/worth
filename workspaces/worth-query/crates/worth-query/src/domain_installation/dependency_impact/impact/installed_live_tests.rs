use super::*;

#[test]
fn conditional_aspect_and_broad_locality_require_signal_authority() {
    let conditional = AspectKey::new("conditional").unwrap();
    let unrelated = AspectKey::new("unrelated").unwrap();
    let aspects = BTreeSet::from([conditional.clone()]);
    let mutation = |aspect| {
        crate::memory_workspace::WorthQueryMutationDelta::from_touched_aspects(
            "Vertex",
            crate::memory_workspace::WorthQueryEntityIdentity::from_relational_record(
                worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(1, 1, 1),
            ),
            crate::memory_workspace::WorthQueryMutationKind::Updated,
            vec![crate::runtime::WorthQueryAspectTouch::whole_aspect(aspect)],
        )
    };

    assert!(mutation_requires_signal_authority(
        &aspects,
        false,
        &mutation(conditional).expect("fixture digest must fit")
    ));
    assert!(!mutation_requires_signal_authority(
        &aspects,
        false,
        &mutation(unrelated.clone()).expect("fixture digest must fit")
    ));
    assert!(mutation_requires_signal_authority(
        &aspects,
        true,
        &mutation(unrelated).expect("fixture digest must fit")
    ));
}

#[test]
fn same_basis_label_cannot_readmit_foreign_capability_affinity() {
    let affinity = |capability_identity| {
        crate::domain_installation::operation_authority_chain::WorthQueryOperationAuthorityBasis {
            runtime_authority: 7,
            installation_runtime_authority: 9,
            installation_generation: 3,
            domain_authority_identity: "domain".into(),
            operation_identity: "operation".into(),
            binding_identity: "binding".into(),
            capability_identity,
            basis_identity: "same-basis".into(),
            graph_authority_identities: vec!["graph".into()],
            required_domain_authority_identities: vec!["required".into()],
            resource_admission_identity: None,
        }
    };

    assert!(!exact_affinity_match(&affinity(11), &affinity(12)));
}

#[test]
fn impact_role_lookup_honors_parent_and_descendant_field_overlap() {
    let path = |fields: &[&str]| {
        CanonicalFieldPath::new(
            fields
                .iter()
                .map(|field| worth_foundational::facade::FieldKey::new(*field).unwrap()),
        )
        .unwrap()
    };
    let mut index =
        crate::canonical_field_path_overlap_index::WorthQueryCanonicalPathOverlapIndex::default();
    index.insert(
        &path(&["address"]),
        WorthQuerySemanticDependencyRole::ProjectedValue,
    );
    index.insert(
        &path(&["address", "city"]),
        WorthQuerySemanticDependencyRole::Ordering,
    );
    index.insert(
        &path(&["address", "postal"]),
        WorthQuerySemanticDependencyRole::Grouping,
    );

    let mut roles = BTreeSet::new();
    let mut lookups = 0;
    extend_overlapping_roles(
        &mut roles,
        Some(&index),
        &path(&["address", "city", "name"]),
        &mut lookups,
    );
    assert_eq!(
        roles,
        BTreeSet::from([
            WorthQuerySemanticDependencyRole::Ordering,
            WorthQuerySemanticDependencyRole::ProjectedValue,
        ])
    );
    assert_eq!(lookups, 4, "one map lookup plus three path-node probes");
}
