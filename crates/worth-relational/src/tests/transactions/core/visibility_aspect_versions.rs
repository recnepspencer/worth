use crate::facade::identity::EntityId;
use crate::tests::support::*;
use worth_foundational::facade::AspectKey;

#[test]
fn visibility_aspect_versions_follow_canonical_delta_truth_and_ignore_undeclared_fields() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(&runtime, "alpha");
    let entity = changed_entities(&created)[0];
    let updated = update_entity(&runtime, entity, "beta");
    let versions = runtime.read_truth().entity_aspect_versions(entity).unwrap();

    assert_eq!(
        versions
            .iter()
            .map(|(aspect, _)| aspect.clone())
            .collect::<Vec<_>>(),
        vec![
            AspectKey::new("lifecycle").unwrap(),
            AspectKey::new("name").unwrap(),
        ]
    );
    assert_eq!(
        versions,
        vec![
            (AspectKey::new("lifecycle").unwrap(), created.version_id.0),
            (AspectKey::new("name").unwrap(), updated.version_id.0),
        ]
    );

    let relation = create_relation(&runtime, entity, entity, "edge");
    let relation_versions = runtime
        .read_truth()
        .relation_aspect_versions(relation)
        .unwrap();
    assert_eq!(
        relation_versions
            .iter()
            .map(|(aspect, _)| aspect.clone())
            .collect::<Vec<_>>(),
        vec![
            AspectKey::new("label").unwrap(),
            AspectKey::new("lifecycle").unwrap(),
            AspectKey::new("source").unwrap(),
            AspectKey::new("target").unwrap(),
        ]
    );
}

#[test]
fn visibility_aspect_versions_reject_stale_generation_ids() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let entity = create_entity(&runtime, "before");
    let stale = EntityId::new(
        entity.partition_id,
        entity.local_slot.0,
        entity.generation.0 + 1,
    );

    assert!(runtime.read_truth().entity_aspect_versions(stale).is_none());
    assert!(runtime
        .read_truth()
        .entity_aspect_versions(entity)
        .is_some());
}

#[test]
fn selected_basis_aspect_revision_reads_only_the_requested_aspect() {
    let runtime = runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(&runtime, "alpha");
    let entity = changed_entities(&created)[0];
    let updated = update_entity(&runtime, entity, "beta");
    let view = runtime
        .read_truth()
        .project_snapshot(&updated.snapshot)
        .expect("the committed basis remains projectable");

    assert_eq!(
        view.entity_aspect_version(entity, &AspectKey::new("name").unwrap()),
        Some(Some(updated.version_id.0))
    );
    assert_eq!(
        view.entity_aspect_version(entity, &AspectKey::new("unrelated").unwrap()),
        Some(None)
    );
    let stale = EntityId::new(
        entity.partition_id,
        entity.local_slot.0,
        entity.generation.0 + 1,
    );
    assert_eq!(
        view.entity_aspect_version(stale, &AspectKey::new("name").unwrap()),
        None
    );
}
