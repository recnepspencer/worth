use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::RecordArenaFixturePort as _;
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::metadata::RecordArenaFixturePort as _;
use super::super::reader::{
    kind_scan_visibility::KindScanVisibility, projection_record_reader::ProjectionRecordReader,
};
use crate::config::data::runtime_config::RelationalRuntimeConfig;
use crate::config::data::CascadeDeletePolicy;
use crate::config::data::CrossContextPolicy;
use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
use crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation;
use crate::schema::data::EntityKindRegistration;
use crate::schema::data::KindAspectContractDeclarations;
use crate::schema::data::RelationIntegrityDeclarations;
use crate::schema::data::RelationKindRegistration;
use crate::schema::data::RelationalSchemaRegistry;
use crate::schema::data::SchemaId;
use crate::schema::data::SchemaVersionId;
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionState;
use crate::storage::partition::AdjacencySet;
use crate::storage::substrate::{EntityExtra, RelationEndpoints, RelationExtra};
use std::collections::BTreeMap;

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

#[test]
fn stale_entity_id_does_not_materialize_reused_slot_at_current_version() {
    let partition_id = PartitionId(7);
    let adjacency_policy = RelationalRuntimeConfig::default().storage.adjacency_policy;
    let mut entity_arena = crate::storage::substrate::EntityArena::with_capacity(1);
    let (slot, generation, _) =
        entity_arena.push_slot_for_test(crate::storage::substrate::SlotInit {
            partition_id,
            kind_id: KindId(11),
            version_id: VersionId(1),
            extra: EntityExtra::default(),
        });
    let stale_id = crate::identity::data::EntityId::new(partition_id, slot as u64, generation);
    entity_arena
        .retire_record_for_test(slot, VersionId(2))
        .expect("fixture slot is materialized");
    entity_arena.lifecycle[slot] = RecordLifecycleState::Reusable;
    entity_arena
        .reset_slot(slot)
        .expect("fixture slot is materialized");
    let (_, reused_generation, _) =
        entity_arena.push_slot_for_test(crate::storage::substrate::SlotInit {
            partition_id,
            kind_id: KindId(12),
            version_id: VersionId(3),
            extra: EntityExtra::default(),
        });
    assert_eq!(reused_generation, 2);

    let state = BTreeMap::from([(
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
    )]);
    let registry = visibility_regression_schema_registry();
    let current_version = VersionId(3);
    let instrumentation = RuntimeInstrumentation::new();
    let read = ProjectionRecordReader::new(current_version, &instrumentation);
    let reused_id = EntityId::new(partition_id, slot as u64, reused_generation);
    assert_eq!(
        read.authoritative_entity_record_for_id_at_version_with_registry(
            &state,
            &registry,
            reused_id,
            current_version,
        )
        .unwrap()
        .entity_id,
        reused_id,
    );
    assert!(read
        .authoritative_entity_record_for_id_at_version_with_registry(
            &state,
            &registry,
            stale_id,
            current_version,
        )
        .is_none());
}

#[test]
fn historical_entity_kind_reads_follow_visible_metadata_not_current_slot_kind() {
    let partition_id = PartitionId(7);
    let adjacency_policy = RelationalRuntimeConfig::default().storage.adjacency_policy;
    let mut entity_arena = crate::storage::substrate::EntityArena::with_capacity(1);
    let (slot, first_generation, _) =
        entity_arena.push_slot_for_test(crate::storage::substrate::SlotInit {
            partition_id,
            kind_id: KindId(11),
            version_id: VersionId(1),
            extra: EntityExtra::default(),
        });
    entity_arena
        .retire_record_for_test(slot, VersionId(2))
        .expect("fixture slot is materialized");
    entity_arena.lifecycle[slot] = RecordLifecycleState::Reusable;
    entity_arena
        .reset_slot(slot)
        .expect("fixture slot is materialized");
    let (_, reused_generation, _) =
        entity_arena.push_slot_for_test(crate::storage::substrate::SlotInit {
            partition_id,
            kind_id: KindId(12),
            version_id: VersionId(3),
            extra: EntityExtra::default(),
        });
    assert_eq!(reused_generation, 2);

    let state = BTreeMap::from([(
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
    )]);
    let registry = visibility_regression_schema_registry();
    let current_version = VersionId(3);
    let instrumentation = RuntimeInstrumentation::new();
    let read = ProjectionRecordReader::new(current_version, &instrumentation);
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let historical_records = super::super::read_work::metered_with_memory(
        &execution,
        |context, memory| {
            read.paid_entities_in_partition(
                &state,
                &registry,
                partition_id,
                Some(KindId(11)),
                KindScanVisibility::for_selected_version(current_version, VersionId(1)),
                &mut |units| context.checkpoint(units),
                memory,
            )
        },
        |answer| {
            answer
                .as_ref()
                .map_or(0, super::super::read_footprint::entities)
        },
    )
    .unwrap()
    .unwrap();
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let reused_kind_records = super::super::read_work::metered_with_memory(
        &execution,
        |context, memory| {
            read.paid_entities_in_partition(
                &state,
                &registry,
                partition_id,
                Some(KindId(12)),
                KindScanVisibility::for_selected_version(current_version, VersionId(1)),
                &mut |units| context.checkpoint(units),
                memory,
            )
        },
        |answer| {
            answer
                .as_ref()
                .map_or(0, super::super::read_footprint::entities)
        },
    )
    .unwrap()
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
    let partition_id = PartitionId(9);
    let adjacency_policy = RelationalRuntimeConfig::default().storage.adjacency_policy;
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
        relation_arena.push_slot_for_test(crate::storage::substrate::SlotInit {
            partition_id,
            kind_id: KindId(21),
            version_id: VersionId(1),
            extra: RelationExtra::from_unbudgeted_parts(Some(first_endpoints), None),
        });
    relation_arena
        .retire_record_for_test(slot, VersionId(2))
        .expect("fixture slot is materialized");
    relation_arena.lifecycle[slot] = RecordLifecycleState::Reusable;
    relation_arena
        .reset_slot(slot)
        .expect("fixture slot is materialized");
    let (_, reused_generation, _) =
        relation_arena.push_slot_for_test(crate::storage::substrate::SlotInit {
            partition_id,
            kind_id: KindId(22),
            version_id: VersionId(3),
            extra: RelationExtra::from_unbudgeted_parts(Some(second_endpoints), None),
        });
    assert_eq!(reused_generation, 2);

    let state = BTreeMap::from([(
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
    )]);
    let registry = visibility_regression_schema_registry();
    let current_version = VersionId(3);
    let instrumentation = RuntimeInstrumentation::new();
    let read = ProjectionRecordReader::new(current_version, &instrumentation);
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let historical_records = super::super::read_work::metered_with_memory(
        &execution,
        |context, memory| {
            read.paid_relations_in_partition(
                &state,
                &registry,
                partition_id,
                Some(KindId(21)),
                KindScanVisibility::for_selected_version(current_version, VersionId(1)),
                &mut |units| context.checkpoint(units),
                memory,
            )
        },
        |answer| {
            answer
                .as_ref()
                .map_or(0, super::super::read_footprint::relations)
        },
    )
    .unwrap()
    .unwrap();
    let host = worth_harness::facade::execution_fixture::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let reused_kind_records = super::super::read_work::metered_with_memory(
        &execution,
        |context, memory| {
            read.paid_relations_in_partition(
                &state,
                &registry,
                partition_id,
                Some(KindId(22)),
                KindScanVisibility::for_selected_version(current_version, VersionId(1)),
                &mut |units| context.checkpoint(units),
                memory,
            )
        },
        |answer| {
            answer
                .as_ref()
                .map_or(0, super::super::read_footprint::relations)
        },
    )
    .unwrap()
    .unwrap();

    assert_eq!(historical_records.len(), 1);
    assert_eq!(
        historical_records[0].relation_id,
        RelationId::new(partition_id, slot as u64, first_generation)
    );
    assert_eq!(historical_records[0].kind.kind_id, KindId(21));
    assert!(reused_kind_records.is_empty());
}
