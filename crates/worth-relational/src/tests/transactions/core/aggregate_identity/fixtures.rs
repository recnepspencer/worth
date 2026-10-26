use crate::aggregate_identity::record::{AggregateRecord, AGGREGATE_ASPECT};
use crate::facade::transactions::{CreatedEntityRef, EntityReference, EntitySpec, RelationSpec};
use crate::identity::data::EntityId;
use crate::schema::data::*;
use crate::tests::support::*;
use worth_foundational::facade::{
    AggregateDefinitionIdentity, AggregateIdentity, AspectIdentity,
    ContractValidatedAspectValueView,
};

pub(super) fn descriptor() -> AggregateIdentityContractDeclaration {
    AggregateIdentityContractDeclaration {
        definition: AggregateDefinitionIdentity::from_bytes([7; 32]),
        source_relation: KindId(2),
        member_kind: KindId(1),
        membership_kind: KindId(5),
        storage: AggregateIdentityStorage {
            aggregate_kind: KindId(3),
            lineage_kind: KindId(4),
            aggregate_aspect: AspectIdentity(3001),
            lineage_aspect: AspectIdentity(3002),
        },
    }
}
pub(super) fn registry() -> RelationalSchemaRegistry {
    let source = RelationIntegrityDeclarations::new(
        vec![EndpointKindContractDeclaration {
            contract_id: "successor:endpoints".into(),
            allowed_source_kinds: vec![KindId(1)],
            allowed_target_kinds: vec![KindId(1)],
            self_edges_allowed: true,
            cross_context_policy: CrossContextPolicy::AllowExplicit,
        }],
        vec![CardinalityContractDeclaration {
            contract_id: "successor:orbit".into(),
            source_min: Some(1),
            source_max: Some(1),
            target_min: Some(1),
            target_max: Some(1),
            pair_min: None,
            pair_max: None,
            pair_min_semantics: PairMinimumSemantics::ObservedDirectedPairs,
            minimum_enforcement: MinimumCardinalityEnforcement::CommitBoundary,
        }],
        vec![],
        vec![],
        vec![],
    );
    let registry = RelationIntegritySchemaFixture {
        relation_integrity: source,
        ..RelationIntegritySchemaFixture::default()
    }
    .build_registry();
    let registry = crate::schema::aggregate_identity::register_aggregate_identity_storage(
        registry,
        &SchemaId("test".into()),
        SchemaVersionId(1),
        descriptor().storage,
    )
    .unwrap();
    crate::schema::aggregate_identity::register_aggregate_membership(
        registry,
        &SchemaId("test".into()),
        SchemaVersionId(1),
        descriptor(),
    )
    .unwrap()
}
pub(super) fn runtime() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(registry())
        .build()
}
pub(super) fn persisted() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(registry())
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(DurableStoreLayout {
            root_path: unique_test_store_path("aggregate-identity"),
            segment_commit_capacity: 8,
        })
        .build()
}
pub(super) fn created(key: usize) -> EntityReference {
    EntityReference::Created(CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw(format!("member-{key}")),
    })
}
pub(super) fn birth(permutation: &[usize]) -> WorkerIntentBatch {
    let mut batch = WorkerIntentBatch::new("aggregate-birth");
    for index in 0..permutation.len() {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: crate::symbols::data::ClientKey::raw(format!("member-{index}")),
            fields: Default::default(),
        })));
    }
    for (index, &target) in permutation.iter().enumerate() {
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw(format!("successor-{index}")),
                source: created(index),
                target: created(target),
                fields: Default::default(),
            },
        )));
    }
    batch
}
pub(super) fn commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &str,
    batch: WorkerIntentBatch,
) -> CommitResult {
    let mut transaction = test_owner_begin_transaction_for_branch(runtime, BranchId(branch.into()));
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(execution, runtime).unwrap()
}
pub(super) fn members(result: &CommitResult, count: usize) -> Vec<EntityId> {
    (0..count)
        .map(|index| {
            let EntityReference::Created(reference) = created(index) else {
                unreachable!()
            };
            result.created_entity(&reference).unwrap()
        })
        .collect()
}
pub(super) fn records(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &crate::snapshots::data::SnapshotHandle,
) -> Vec<(EntityId, AggregateRecord)> {
    let read = runtime
        .read_truth()
        .read_snapshot(snapshot, execution)
        .unwrap()
        .unwrap();
    read.entities()
        .into_iter()
        .filter(|entity| entity.kind.kind_id == KindId(3))
        .map(|entity| {
            let state = entity.authoritative_aspect_state.as_ref().unwrap();
            let ContractValidatedAspectValueView::Struct(fields) =
                state.get(&aspect_key(AGGREGATE_ASPECT)).unwrap().view()
            else {
                panic!("aggregate grammar")
            };
            let row = AggregateRecord::decode(|locator| {
                fields.get(locator.field_path().fields().first()?)
            })
            .unwrap();
            (entity.entity_id, row)
        })
        .collect()
}
pub(super) fn identity(
    rows: &[(EntityId, AggregateRecord)],
    anchor: EntityId,
) -> AggregateIdentity {
    rows.iter()
        .find(|(_, row)| row.current_anchor == Some(anchor))
        .unwrap()
        .1
        .identity
}
pub(super) fn rewire(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &crate::snapshots::data::SnapshotHandle,
    changes: &[(EntityId, EntityId)],
) -> WorkerIntentBatch {
    let read = runtime
        .read_truth()
        .read_snapshot(snapshot, execution)
        .unwrap()
        .unwrap();
    let relations = read.relations();
    let mut batch = WorkerIntentBatch::new("rewire");
    for &(source, target) in changes {
        let edge = relations
            .iter()
            .find(|edge| edge.kind.kind_id == KindId(2) && edge.source == source)
            .unwrap();
        batch = batch.push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: edge.relation_id,
                kind_id: KindId(2),
                source: EntityReference::Existing(source),
                target: EntityReference::Existing(target),
            }),
        ));
    }
    batch
}
