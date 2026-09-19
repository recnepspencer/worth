use super::*;

#[test]
fn relevance_index_returns_only_exact_installed_targets() {
    let mut index = WorthQueryInstalledLiveTargetIndex::default();
    for ordinal in 0..64 {
        index.register(
            WorthQueryLiveArtifactTarget::from_view_name(format!("unrelated-{ordinal}")),
            "Vertex".into(),
            selector("identity", "name"),
        );
    }
    let first = WorthQueryLiveArtifactTarget::from_view_name("installed-first");
    let second = WorthQueryLiveArtifactTarget::from_view_name("installed-second");
    index.register(first.clone(), "Vertex".into(), selector("identity", "id"));
    index.register(second.clone(), "Vertex".into(), selector("identity", "id"));

    let expected = BTreeSet::from([first, second]);
    assert_eq!(
        index.affected_targets(&mutation("identity")).targets,
        expected
    );
}

#[test]
fn canonical_parent_path_selects_descendant_dependency_without_sibling_scan() {
    let mut index = WorthQueryInstalledLiveTargetIndex::default();
    let target = WorthQueryLiveArtifactTarget::from_view_name("nested-city");
    let mut nested = selector("profile", "city");
    nested.aspect_routes.clear();
    nested.field_routes.insert((
        worth_foundational::facade::AspectKey::new("profile").unwrap(),
        path(&["address", "city"]),
    ));
    index.register(target.clone(), "Vertex".into(), nested);

    assert_eq!(
        index.affected_targets(&path_mutation(&["address"])).targets,
        BTreeSet::from([target])
    );
    assert!(index
        .affected_targets(&path_mutation(&["address", "postal"]))
        .targets
        .is_empty());
}

fn selector(
    aspect: &str,
    field: &str,
) -> crate::domain_installation::WorthQueryInstalledLiveRoutingSelector {
    let aspect = worth_foundational::facade::AspectKey::new(aspect).unwrap();
    crate::domain_installation::WorthQueryInstalledLiveRoutingSelector {
        aspect_routes: BTreeSet::from([aspect.clone()]),
        whole_aspect_routes: BTreeSet::new(),
        field_routes: BTreeSet::from([(
            aspect,
            worth_foundational::facade::CanonicalFieldPath::single(
                worth_foundational::facade::FieldKey::new(field).unwrap(),
            ),
        )]),
        structural_creation: false,
        broad: false,
        empty_touch: false,
    }
}

fn mutation(aspect: &str) -> WorthQueryMutationDelta {
    WorthQueryMutationDelta::from_touched_aspects(
        "Vertex",
        crate::memory_workspace::WorthQueryEntityIdentity::from_relational_record(
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(1, 1, 1),
        ),
        WorthQueryMutationKind::Updated,
        vec![crate::runtime::WorthQueryAspectTouch::aspect_field_path(
            worth_foundational::facade::AspectKey::new(aspect).unwrap(),
            worth_foundational::facade::CanonicalFieldPath::single(
                worth_foundational::facade::FieldKey::new("id").unwrap(),
            ),
        )],
    )
}

fn path_mutation(fields: &[&str]) -> WorthQueryMutationDelta {
    WorthQueryMutationDelta::from_touched_aspects(
        "Vertex",
        crate::memory_workspace::WorthQueryEntityIdentity::from_relational_record(
            worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(1, 1, 1),
        ),
        WorthQueryMutationKind::Updated,
        vec![crate::runtime::WorthQueryAspectTouch::aspect_field_path(
            worth_foundational::facade::AspectKey::new("profile").unwrap(),
            path(fields),
        )],
    )
}

fn path(fields: &[&str]) -> worth_foundational::facade::CanonicalFieldPath {
    worth_foundational::facade::CanonicalFieldPath::new(
        fields
            .iter()
            .map(|field| worth_foundational::facade::FieldKey::new(*field).unwrap()),
    )
    .unwrap()
}
