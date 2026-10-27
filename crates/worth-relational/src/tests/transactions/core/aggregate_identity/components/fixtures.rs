use super::super::fixtures;
use crate::facade::runtime::{PinnedAggregateRead, PinnedAggregateReadScope};
pub(super) use crate::facade::snapshots::SnapshotHandle;
pub(super) use crate::facade::transactions::EntityReference;
use crate::facade::transactions::{CreatedRelationRef, EntitySpec, RelationSpec};
pub(super) use crate::identity::data::EntityId;
pub(super) use crate::schema::data::*;
pub(super) use crate::tests::support::*;
pub(super) use worth_execution::ExecutionRequest;
pub(super) use worth_foundational::facade::AggregateDefinitionIdentity;

pub(super) fn descriptor(second: bool) -> AggregateIdentityContractDeclaration {
    let mut contract = fixtures::descriptor();
    contract.definition = AggregateDefinitionIdentity::from_bytes([if second { 9 } else { 8 }; 32]);
    contract.membership_kind = KindId(if second { 8 } else { 7 });
    contract.source = AggregateIdentitySource::Components {
        relations: if second {
            vec![KindId(2)]
        } else {
            vec![KindId(6), KindId(2), KindId(6)]
        },
    };
    contract
}
pub(super) fn registry() -> RelationalSchemaRegistry {
    let mut registry = fixtures::registry();
    registry.relation_kinds.remove(&KindId(5));
    let mut source = registry.relation_kinds.remove(&KindId(2)).unwrap();
    source.relation_integrity.cardinality_contracts.clear();
    registry = registry.register_relation_kind(source).unwrap();
    registry = registry
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(6),
            kind_name: "other-coupling".into(),
            schema_id: SchemaId("test".into()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: Default::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![EndpointKindContractDeclaration {
                    contract_id: "coupling:endpoints".into(),
                    allowed_source_kinds: vec![KindId(1)],
                    allowed_target_kinds: vec![KindId(1)],
                    self_edges_allowed: true,
                    cross_context_policy: CrossContextPolicy::AllowExplicit,
                }],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
        })
        .unwrap();
    for second in [false, true] {
        registry = crate::schema::aggregate_identity::register_aggregate_membership(
            registry,
            &SchemaId("test".into()),
            SchemaVersionId(1),
            descriptor(second),
        )
        .unwrap();
    }
    registry
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
            root_path: unique_test_store_path("component-identity"),
            segment_commit_capacity: 8,
        })
        .build()
}
const COUPLINGS: [(usize, u32, usize, usize); 6] = [
    (0, 2, 0, 1),
    (1, 6, 1, 0),
    (2, 6, 1, 2),
    (3, 2, 3, 3),
    (4, 2, 4, 5),
    (5, 6, 0, 1),
];

pub(super) fn birth() -> WorkerIntentBatch {
    let mut batch = WorkerIntentBatch::new("component-birth");
    for index in 0..6 {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            client_key: crate::symbols::data::ClientKey::raw(format!("member-{index}")),
            fields: Default::default(),
        })));
    }
    for (key, kind, source, target) in COUPLINGS {
        batch = batch.push(edge(
            key,
            kind,
            fixtures::created(source),
            fixtures::created(target),
        ));
    }
    batch
}
pub(super) fn edge(
    key: usize,
    kind: u32,
    source: EntityReference,
    target: EntityReference,
) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: KindId(kind),
        client_key: crate::symbols::data::ClientKey::raw(format!("coupling-{key}")),
        source,
        target,
        fields: Default::default(),
    }))
}
pub(super) fn delete_edge(born: &CommitResult, key: usize, kind: u32) -> MutationIntent {
    let &(_, _, source, target) = COUPLINGS
        .iter()
        .find(|&&(candidate, candidate_kind, _, _)| candidate == key && candidate_kind == kind)
        .unwrap();
    let id = born
        .created_relation(&CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(kind),
            client_key: crate::symbols::data::ClientKey::raw(format!("coupling-{key}")),
            source: fixtures::created(source),
            target: fixtures::created(target),
        })
        .unwrap();
    MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
        relation_id: id,
    }))
}
pub(super) fn read(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    second: bool,
    member: EntityId,
) -> PinnedAggregateRead {
    runtime
        .read_truth()
        .pin_read_capability(snapshot, &[], execution)
        .unwrap()
        .read_aggregate(
            execution,
            PinnedAggregateReadScope::for_definition(descriptor(second).definition, member),
        )
        .unwrap()
}
pub(super) fn commit(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    batch: WorkerIntentBatch,
) -> CommitResult {
    fixtures::commit(execution, runtime, "main", batch)
}
pub(super) fn ids(born: &CommitResult) -> Vec<EntityId> {
    fixtures::members(born, 6)
}
