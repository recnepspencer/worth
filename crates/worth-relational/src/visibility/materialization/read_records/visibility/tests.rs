use crate::facade::config::{CascadeDeletePolicy, CrossContextPolicy};
use crate::facade::runtime::RelationalRuntimeApi;
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::runtime::{RelationalRuntime, RelationalRuntimeConfig};
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::AdjacencySet;
use crate::storage::substrate::{EntityExtra, RelationEndpoints, RelationExtra};

fn visibility_regression_schema_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(11),
            kind_name: "test.entity.alpha".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_entity_kind(EntityKindRegistration {
                kind_id: KindId(12),
                kind_name: "test.entity.beta".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(21),
                kind_name: "test.relation.alpha".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(22),
                kind_name: "test.relation.beta".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .unwrap()
}

fn visibility_regression_runtime() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(visibility_regression_schema_registry())
        .build()
}

#[test]
fn stale_entity_id_does_not_materialize_reused_slot_at_current_version() {
    let runtime = RelationalRuntime::new(RelationalRuntimeConfig::default());
    let partition_id = PartitionId(7);
    let adjacency_policy = runtime.config.storage.adjacency_policy.clone();
    let mut entity_arena = crate::storage::substrate::EntityArena::with_capacity(1);
    let (slot, generation, _) = entity_arena.push_slot(crate::storage::substrate::SlotInit {
        partition_id,
        kind_id: KindId(11),
        version_id: VersionId(1),
        extra: EntityExtra::default(),
    });
    let stale_id = crate::identity::data::EntityId::new(partition_id, slot as u64, generation);
    entity_arena.retire(slot, VersionId(2));
    entity_arena.lifecycle[slot] = RecordLifecycleState::Reusable;
    entity_arena.reset_slot(slot);
    let (_, reused_generation, _) = entity_arena.push_slot(crate::storage::substrate::SlotInit {
        partition_id,
        kind_id: KindId(12),
        version_id: VersionId(3),
        extra: EntityExtra::default(),
    });
    assert_eq!(reused_generation, 2);

    runtime
        .history
        .with_ledger_mut(|ledger| ledger.next_version_id = 4);
    runtime.edit_partitions().insert(
        partition_id,
        PartitionState {
            partition_id,
            adjacency_policy: adjacency_policy.clone(),
            relation_overlay_is_sparse: false,
            entity_arena,
            relation_arena: crate::storage::substrate::RelationArena::with_capacity(0),
            adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
            reverse_adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
        },
    );

    let current_state = runtime.storage_access().current_edition();
    assert!(runtime
        .read_truth()
        .authoritative_entity_record_for_id_at_version(
            &current_state,
            stale_id,
            runtime.current_version_id()
        )
        .is_none());
}

#[test]
fn historical_entity_kind_reads_follow_visible_metadata_not_current_slot_kind() {
    let runtime = visibility_regression_runtime();
    let partition_id = PartitionId(7);
    let adjacency_policy = runtime.config.storage.adjacency_policy.clone();
    let mut entity_arena = crate::storage::substrate::EntityArena::with_capacity(1);
    let (slot, first_generation, _) = entity_arena.push_slot(crate::storage::substrate::SlotInit {
        partition_id,
        kind_id: KindId(11),
        version_id: VersionId(1),
        extra: EntityExtra::default(),
    });
    entity_arena.retire(slot, VersionId(2));
    entity_arena.lifecycle[slot] = RecordLifecycleState::Reusable;
    entity_arena.reset_slot(slot);
    let (_, reused_generation, _) = entity_arena.push_slot(crate::storage::substrate::SlotInit {
        partition_id,
        kind_id: KindId(12),
        version_id: VersionId(3),
        extra: EntityExtra::default(),
    });
    assert_eq!(reused_generation, 2);

    runtime
        .history
        .with_ledger_mut(|ledger| ledger.next_version_id = 4);
    runtime.edit_partitions().insert(
        partition_id,
        PartitionState {
            partition_id,
            adjacency_policy: adjacency_policy.clone(),
            relation_overlay_is_sparse: false,
            entity_arena,
            relation_arena: crate::storage::substrate::RelationArena::with_capacity(0),
            adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
            reverse_adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
        },
    );

    let state = runtime.storage_access().current_edition();
    let historical_records = runtime
        .read_truth()
        .paid_entities_in_partition(
            &state,
            &runtime.config.schema.registry,
            partition_id,
            Some(KindId(11)),
            super::super::reader::KindScanVisibility::for_version(&runtime, VersionId(1)),
            &mut |_| Ok(()),
        )
        .unwrap();
    let reused_kind_records = runtime
        .read_truth()
        .paid_entities_in_partition(
            &state,
            &runtime.config.schema.registry,
            partition_id,
            Some(KindId(12)),
            super::super::reader::KindScanVisibility::for_version(&runtime, VersionId(1)),
            &mut |_| Ok(()),
        )
        .unwrap();

    assert_eq!(historical_records.len(), 1);
    assert_eq!(
        historical_records[0].entity_id,
        EntityId::new(partition_id, slot as u64, first_generation)
    );
    assert_eq!(historical_records[0].kind.kind_id, KindId(11));
    assert!(reused_kind_records.is_empty());
}

#[test]
fn historical_relation_kind_reads_follow_visible_metadata_not_current_slot_kind() {
    let runtime = visibility_regression_runtime();
    let partition_id = PartitionId(9);
    let adjacency_policy = runtime.config.storage.adjacency_policy.clone();
    let mut relation_arena = crate::storage::substrate::RelationArena::with_capacity(1);
    let first_endpoints = RelationEndpoints {
        source: EntityId::new(partition_id, 1, 1),
        target: EntityId::new(partition_id, 2, 1),
    };
    let second_endpoints = RelationEndpoints {
        source: EntityId::new(partition_id, 3, 1),
        target: EntityId::new(partition_id, 4, 1),
    };
    let (slot, first_generation, _) =
        relation_arena.push_slot(crate::storage::substrate::SlotInit {
            partition_id,
            kind_id: KindId(21),
            version_id: VersionId(1),
            extra: RelationExtra {
                endpoints: Some(first_endpoints),
                authoritative_aspect_state: None,
            },
        });
    relation_arena.retire(slot, VersionId(2));
    relation_arena.lifecycle[slot] = RecordLifecycleState::Reusable;
    relation_arena.reset_slot(slot);
    let (_, reused_generation, _) = relation_arena.push_slot(crate::storage::substrate::SlotInit {
        partition_id,
        kind_id: KindId(22),
        version_id: VersionId(3),
        extra: RelationExtra {
            endpoints: Some(second_endpoints),
            authoritative_aspect_state: None,
        },
    });
    assert_eq!(reused_generation, 2);

    runtime
        .history
        .with_ledger_mut(|ledger| ledger.next_version_id = 4);
    runtime.edit_partitions().insert(
        partition_id,
        PartitionState {
            partition_id,
            adjacency_policy: adjacency_policy.clone(),
            relation_overlay_is_sparse: false,
            entity_arena: crate::storage::substrate::EntityArena::with_capacity(0),
            relation_arena,
            adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
            reverse_adjacency: vec![AdjacencySet::new(&adjacency_policy)].into(),
        },
    );

    let state = runtime.storage_access().current_edition();
    let historical_records = runtime
        .read_truth()
        .paid_relations_in_partition(
            &state,
            &runtime.config.schema.registry,
            partition_id,
            Some(KindId(21)),
            super::super::reader::KindScanVisibility::for_version(&runtime, VersionId(1)),
            &mut |_| Ok(()),
        )
        .unwrap();
    let reused_kind_records = runtime
        .read_truth()
        .paid_relations_in_partition(
            &state,
            &runtime.config.schema.registry,
            partition_id,
            Some(KindId(22)),
            super::super::reader::KindScanVisibility::for_version(&runtime, VersionId(1)),
            &mut |_| Ok(()),
        )
        .unwrap();

    assert_eq!(historical_records.len(), 1);
    assert_eq!(
        historical_records[0].relation_id,
        RelationId::new(partition_id, slot as u64, first_generation)
    );
    assert_eq!(historical_records[0].kind.kind_id, KindId(21));
    assert!(reused_kind_records.is_empty());
}
