//! Exact traversal order and lifetime of an installed pinned walk.
use super::pinned_read_publication::snapshot_for_main;
use super::proposal_records::{create, create_relation_in_partition};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::RelationDirection;
use worth_relational::facade::{
    config::CascadeDeletePolicy,
    identity::{EntityId, KindId, PartitionId},
    runtime::{PinnedStructuralWalkReads, PinnedStructuralWalkScope, RelationalRuntime},
    transactions::{CreateIntent, EntitySpec, RecordRef},
};
use worth_testkit_relational::{
    aspect_storage::AspectSchemaFixture, durable_publication::field_patch,
    shared_relational_certification_test_execution_policy::serial_request,
};

fn scope(roots: &[EntityId], direction: RelationDirection) -> PinnedStructuralWalkScope<'_> {
    PinnedStructuralWalkScope::new(roots, KindId(1), KindId(2), direction)
}

#[test]
fn complete_walk_is_global_canonical_and_keeps_physical_cycle_edges() {
    let host = serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    )
    .build_runtime();
    let ids: Vec<_> = (0..7)
        .map(|i| {
            create_entity_in_partition(&request, &runtime, &format!("walk-{i}"), PartitionId(7))
        })
        .collect();
    let pairs = [(0, 0), (0, 1), (0, 2), (1, 3), (2, 3), (3, 0), (3, 4)];
    let edges: Vec<_> = pairs
        .into_iter()
        .enumerate()
        .map(|(i, (a, b))| {
            create_relation_in_partition(
                &request,
                &runtime,
                ids[a],
                ids[b],
                &format!("edge-{i}"),
                PartitionId(7),
            )
        })
        .collect();
    let pin = runtime
        .read_truth()
        .pin_read_capability(&snapshot_for_main(&runtime), &[], &request)
        .unwrap();
    let walk = pin
        .complete_relation_walk(&request, scope(&ids[..1], RelationDirection::Forward))
        .unwrap();
    assert_eq!(
        walk.visits()
            .iter()
            .map(|v| (v.entity.entity_id, v.distance))
            .collect::<Vec<_>>(),
        vec![
            (ids[0], 0),
            (ids[1], 1),
            (ids[2], 1),
            (ids[3], 2),
            (ids[4], 3)
        ]
    );
    assert_eq!(
        walk.relations()
            .iter()
            .map(|r| r.relation_id)
            .collect::<Vec<_>>(),
        edges
    );
    assert_eq!(
        walk.reads(),
        PinnedStructuralWalkReads {
            adjacency_lists: 5,
            relation_records: 7,
            entity_records: 5
        }
    );
    let reverse = pin
        .complete_relation_walk(&request, scope(&ids[4..5], RelationDirection::Reverse))
        .unwrap();
    assert_eq!(
        reverse
            .visits()
            .iter()
            .map(|v| (v.entity.entity_id, v.distance))
            .collect::<Vec<_>>(),
        vec![
            (ids[4], 0),
            (ids[3], 1),
            (ids[1], 2),
            (ids[2], 2),
            (ids[0], 3)
        ]
    );
    assert_eq!(reverse.relations(), walk.relations());
    let roots = [ids[6], ids[3], ids[0], ids[2], ids[1], ids[5], ids[3]];
    let all = pin
        .complete_relation_walk(&request, scope(&roots, RelationDirection::Forward))
        .unwrap();
    let shuffled: Vec<_> = roots.into_iter().rev().collect();
    let twin = pin
        .complete_relation_walk(&request, scope(&shuffled, RelationDirection::Forward))
        .unwrap();
    assert_eq!(all.visits(), twin.visits());
    assert_eq!(all.relations(), twin.relations());
    assert_eq!(all.visits().len(), 7);
    assert_eq!(all.visits().last().unwrap().distance, 1);
    assert_eq!(all.reads().entity_records, 7);
    assert_eq!(all.reads().relation_records, 7);
}

#[test]
fn pinned_walk_survives_owner_drop_and_excludes_later_entities() {
    let host = serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = AspectSchemaFixture::with_default_declared_aspects(
        CascadeDeletePolicy::CascadeDeleteRelations,
    )
    .build_runtime();
    let a = create_entity_in_partition(&request, &runtime, "a", PartitionId(7));
    let b = create_entity_in_partition(&request, &runtime, "b", PartitionId(7));
    create_relation_in_partition(&request, &runtime, a, b, "ab", PartitionId(7));
    let before = runtime
        .read_truth()
        .pin_read_capability(&snapshot_for_main(&runtime), &[], &request)
        .unwrap();
    let c = create_entity_in_partition(&request, &runtime, "c", PartitionId(7));
    create_relation_in_partition(&request, &runtime, b, c, "bc", PartitionId(7));
    let after = runtime
        .read_truth()
        .pin_read_capability(&snapshot_for_main(&runtime), &[], &request)
        .unwrap();
    drop(runtime);
    let old = before
        .complete_relation_walk(&request, scope(&[a], RelationDirection::Forward))
        .unwrap();
    let new = after
        .complete_relation_walk(&request, scope(&[a], RelationDirection::Forward))
        .unwrap();
    assert_eq!(old.visits().len(), 2);
    assert_eq!(new.visits().len(), 3);
}

fn create_entity_in_partition(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    partition_id: PartitionId,
) -> EntityId {
    let RecordRef::Entity(entity) = create(
        execution,
        runtime,
        key,
        CreateIntent::Entity(EntitySpec {
            partition_id,
            kind_id: KindId(1),
            client_key: key.into(),
            fields: field_patch("name", "name", key),
        }),
    ) else {
        panic!("expected the authored entity");
    };
    entity
}
