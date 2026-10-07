use super::canonical_digest::query_result_reduction_digest_checked;
use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::schema::data::{KindResolution, SchemaId, SchemaVersionId};
use crate::storage::data::{EntityReadRecord, RecordLifecycleState, RelationReadRecord};

use super::{
    query_authoritative_entity_record_digest, reduce_query_fragments,
    reduce_query_fragments_checked, DeterministicQueryFragmentKey, DeterministicQueryPlanKey,
    QueryExecutionShape, QueryFragmentCounters, QueryOrderingContract, QueryWorkerFragment,
    TraversalEntityVisitKey, TraversalReductionBasis, TraversalRelationVisitKey,
};

fn test_authoritative_entity_record(
    entity_id: crate::identity::data::EntityId,
    kind: KindResolution,
) -> EntityReadRecord {
    EntityReadRecord {
        entity_id,
        lineage_id: None,
        kind,
        lifecycle: RecordLifecycleState::Live,
        created_at_version: VersionId(1),
        retired_at_version: None,
        authoritative_aspect_state: None,
    }
}

#[test]
fn checked_reduction_matches_serial_ordering_and_can_stop_during_digest() {
    let entity = crate::identity::data::EntityId::new(PartitionId(1), 1, 1);
    let kind = KindResolution {
        kind_id: KindId(1),
        kind_name: "test.entity".to_string(),
        schema_id: SchemaId("test".to_string()),
        schema_version_id: SchemaVersionId(1),
    };
    for ordering in [
        QueryOrderingContract::CanonicalEntityIdOrder,
        QueryOrderingContract::CanonicalRelationIdOrder,
        QueryOrderingContract::CanonicalRecordRefOrder,
        QueryOrderingContract::CanonicalTraversalOrder,
    ] {
        let fragments = vec![QueryWorkerFragment {
            plan_key: DeterministicQueryPlanKey(1),
            fragment_key: DeterministicQueryFragmentKey(1),
            ordering,
            entities: vec![test_authoritative_entity_record(entity, kind.clone())],
            relations: vec![],
            counters: QueryFragmentCounters {
                target_count: 1,
                authoritative_entity_records_emitted: 1,
                authoritative_relation_records_emitted: 0,
                touched_partitions: 1,
            },
            traversal_basis: None,
        }];
        let serial = reduce_query_fragments(
            QueryExecutionShape::BulkPacketized,
            ordering,
            fragments.clone(),
        );
        let checked = reduce_query_fragments_checked(
            QueryExecutionShape::BulkPacketized,
            ordering,
            fragments.clone(),
            |_, _| Ok::<(), ()>(()),
        )
        .unwrap();
        assert_eq!(checked, serial);
    }
    let record = test_authoritative_entity_record(entity, kind);
    let stopped = query_result_reduction_digest_checked(
        QueryOrderingContract::CanonicalEntityIdOrder,
        &[record],
        &[],
        |work, _| if work > 0 { Err(()) } else { Ok(()) },
    );
    assert_eq!(stopped, Err(()));
}

fn test_authoritative_relation_record(
    relation_id: crate::identity::data::RelationId,
    kind: KindResolution,
    source: crate::identity::data::EntityId,
    target: crate::identity::data::EntityId,
) -> RelationReadRecord {
    RelationReadRecord {
        relation_id,
        kind,
        lifecycle: RecordLifecycleState::Live,
        created_at_version: VersionId(1),
        retired_at_version: None,
        source,
        target,
        authoritative_aspect_state: None,
    }
}

#[test]
fn traversal_reduction_uses_visit_keys_for_cross_fragment_determinism() {
    let seed = crate::identity::data::EntityId::new(PartitionId(1), 1, 1);
    let mid = crate::identity::data::EntityId::new(PartitionId(1), 2, 1);
    let leaf = crate::identity::data::EntityId::new(PartitionId(1), 3, 1);
    let relation = crate::identity::data::RelationId::new(PartitionId(1), 1, 1);
    let kind = KindResolution {
        kind_id: KindId(1),
        kind_name: "test.entity".to_string(),
        schema_id: SchemaId("test".to_string()),
        schema_version_id: SchemaVersionId(1),
    };
    let relation_kind = KindResolution {
        kind_id: KindId(2),
        kind_name: "test.relation".to_string(),
        schema_id: SchemaId("test".to_string()),
        schema_version_id: SchemaVersionId(1),
    };
    let mid_record = test_authoritative_entity_record(mid, kind.clone());
    let mid_duplicate_record = test_authoritative_entity_record(
        mid,
        KindResolution {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
        },
    );
    let expected_mid_record = if query_authoritative_entity_record_digest(&mid_record)
        <= query_authoritative_entity_record_digest(&mid_duplicate_record)
    {
        mid_record.clone()
    } else {
        mid_duplicate_record.clone()
    };

    let reduced = reduce_query_fragments(
        QueryExecutionShape::BulkPacketized,
        QueryOrderingContract::CanonicalTraversalOrder,
        vec![
            QueryWorkerFragment {
                plan_key: DeterministicQueryPlanKey(1),
                fragment_key: DeterministicQueryFragmentKey(2),
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                entities: vec![
                    mid_record,
                    test_authoritative_entity_record(leaf, kind.clone()),
                ],
                relations: vec![test_authoritative_relation_record(
                    relation,
                    relation_kind,
                    seed,
                    mid,
                )],
                counters: QueryFragmentCounters {
                    target_count: 1,
                    authoritative_entity_records_emitted: 2,
                    authoritative_relation_records_emitted: 1,
                    touched_partitions: 1,
                },
                traversal_basis: Some(TraversalReductionBasis {
                    entity_visit_keys: vec![
                        TraversalEntityVisitKey {
                            depth: 1,
                            root_seed: seed,
                            via_relation: Some(relation),
                            entity_id: mid,
                        },
                        TraversalEntityVisitKey {
                            depth: 2,
                            root_seed: seed,
                            via_relation: Some(relation),
                            entity_id: leaf,
                        },
                    ],
                    relation_visit_keys: vec![TraversalRelationVisitKey {
                        depth: 0,
                        root_seed: seed,
                        relation_id: relation,
                    }],
                }),
            },
            QueryWorkerFragment {
                plan_key: DeterministicQueryPlanKey(1),
                fragment_key: DeterministicQueryFragmentKey(1),
                ordering: QueryOrderingContract::CanonicalTraversalOrder,
                entities: vec![
                    test_authoritative_entity_record(seed, kind),
                    mid_duplicate_record,
                ],
                relations: vec![],
                counters: QueryFragmentCounters {
                    target_count: 1,
                    authoritative_entity_records_emitted: 2,
                    authoritative_relation_records_emitted: 0,
                    touched_partitions: 1,
                },
                traversal_basis: Some(TraversalReductionBasis {
                    entity_visit_keys: vec![
                        TraversalEntityVisitKey {
                            depth: 0,
                            root_seed: seed,
                            via_relation: None,
                            entity_id: seed,
                        },
                        TraversalEntityVisitKey {
                            depth: 1,
                            root_seed: seed,
                            via_relation: Some(relation),
                            entity_id: mid,
                        },
                    ],
                    relation_visit_keys: vec![],
                }),
            },
        ],
    );

    assert_eq!(
        reduced
            .entities
            .iter()
            .map(|record| record.entity_id)
            .collect::<Vec<_>>(),
        vec![seed, mid, leaf]
    );
    assert_eq!(
        reduced
            .relations
            .iter()
            .map(|record| record.relation_id)
            .collect::<Vec<_>>(),
        vec![relation]
    );
    assert_eq!(reduced.entities[1], expected_mid_record);
}
