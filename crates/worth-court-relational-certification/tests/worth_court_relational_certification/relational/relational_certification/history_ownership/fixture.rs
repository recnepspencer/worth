//! Generic model/member facts created only through public Native transactions.
pub(super) use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
pub(super) use worth_execution::{
    CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest,
};
pub(super) use worth_relational::facade::{
    branch::{
        RelationalModelOwnership, RelationalModelOwnershipDenial as Denial,
        RelationalModelOwnershipDomain,
    },
    durability::*,
    identity::{EntityId, KindId, PartitionId},
    mvcc::{BranchBoundRelationalTransaction, RelationalTransactionIntent},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::*,
    storage::{RelationalStorageSubject, RelationalStorageUnavailableCause as Cause},
    symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;
pub(super) mod cold;
use cold::reopen;
pub(super) use cold::MemberSource;

pub(super) fn serial_request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationToken::new(),
        None,
    )
}
pub(super) fn registry() -> RelationalSchemaRegistry {
    registry_with(AspectSchemaFixture::default())
}
pub(super) fn registry_with(fixture: AspectSchemaFixture) -> RelationalSchemaRegistry {
    fixture
        .build_registry()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(3),
            kind_name: "Member".into(),
            schema_id: fixture.schema_id.clone(),
            schema_version_id: fixture.schema_version_id,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(4),
            kind_name: "Reference".into(),
            schema_id: fixture.schema_id.clone(),
            schema_version_id: fixture.schema_version_id,
            cross_context_policy:
                worth_relational::facade::config::CrossContextPolicy::AllowExplicit,
            cascade_delete_policy:
                worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![],
                vec![],
                vec![],
                vec![],
                vec![EndpointDeletionIntegrityDeclaration {
                    contract_id: ContractId::new("References:deletion"),
                    mode: EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations,
                }],
            ),
        })
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(5),
            kind_name: "OtherOwnership".into(),
            schema_id: fixture.schema_id.clone(),
            schema_version_id: fixture.schema_version_id,
            cross_context_policy:
                worth_relational::facade::config::CrossContextPolicy::AllowExplicit,
            cascade_delete_policy:
                worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: RelationIntegrityDeclarations::default(),
        })
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(6),
            kind_name: "UncontractedReference".into(),
            schema_id: fixture.schema_id.clone(),
            schema_version_id: fixture.schema_version_id,
            cross_context_policy:
                worth_relational::facade::config::CrossContextPolicy::AllowExplicit,
            cascade_delete_policy:
                worth_relational::facade::config::CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: RelationIntegrityDeclarations::default(),
        })
        .unwrap()
}
pub(super) fn runtime() -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(registry())
        .build()
        .unwrap()
}
pub(super) fn installed_runtime() -> RelationalRuntime {
    let mut runtime = runtime();
    runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install_with_custom_invariants(
            RelationalSchemaRegistry::new(),
            vec![ownership().invariant_registration().unwrap()],
        )
        .unwrap();
    runtime
}
pub(super) fn ownership() -> RelationalModelOwnership {
    RelationalModelOwnership::new(vec![RelationalModelOwnershipDomain {
        root: KindId(1),
        relations: vec![KindId(2)],
        entities: vec![KindId(3)],
    }])
}
pub(super) fn begin(runtime: &RelationalRuntime) -> BranchBoundRelationalTransaction {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap()
}
pub(super) fn release_test_commit_snapshot(runtime: &RelationalRuntime, committed: &CommitResult) {
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
}
fn entity(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    partition_id: PartitionId,
    kind_id: KindId,
) -> EntityId {
    let mut transaction = begin(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id,
                    kind_id,
                    client_key: ClientKey::raw(key),
                    fields: AspectFieldPatch::default(),
                },
            ))),
            execution.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(execution, runtime).unwrap();
    let id = result
        .changed_records
        .iter()
        .find_map(|r| match r {
            RecordRef::Entity(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    release_test_commit_snapshot(runtime, &result);
    id
}
pub(super) fn create_entity(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> EntityId {
    create_entity_in_partition(execution, runtime, key, PartitionId::main())
}
pub(super) fn create_entity_in_partition(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
    partition: PartitionId,
) -> EntityId {
    entity(execution, runtime, key, partition, KindId(1))
}
pub(super) fn member(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> EntityId {
    entity(execution, runtime, key, PartitionId(3), KindId(3))
}
pub(super) fn create_relation(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    key: &str,
) -> worth_relational::facade::identity::RelationId {
    create_relation_of_kind(execution, runtime, source, target, key, KindId(2))
}
pub(super) fn create_relation_of_kind(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: EntityId,
    target: EntityId,
    key: &str,
    kind: KindId,
) -> worth_relational::facade::identity::RelationId {
    let mut transaction = begin(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new(key).push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: kind,
                    client_key: ClientKey::raw(key),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(target),
                    fields: AspectFieldPatch::default(),
                },
            ))),
            execution.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(execution, runtime).unwrap();
    let id = result
        .changed_records
        .iter()
        .find_map(|r| match r {
            RecordRef::Relation(id) => Some(*id),
            _ => None,
        })
        .unwrap();
    release_test_commit_snapshot(runtime, &result);
    id
}
pub(super) struct Graph {
    pub runtime: RelationalRuntime,
    pub source: Arc<MemberSource>,
    pub model: EntityId,
    pub nested: EntityId,
}
pub(super) fn graph(execution: &ExecutionRequest<'_, '_>) -> Graph {
    let producer = runtime();
    let model = create_entity_in_partition(execution, &producer, "model", PartitionId(2));
    let direct = member(execution, &producer, "direct");
    let nested = member(execution, &producer, "nested");
    create_relation(execution, &producer, model, direct, "owns");
    create_relation(execution, &producer, direct, nested, "child");
    let (runtime, source) = reopen(execution, &producer, registry());
    assert_eq!(source.reads.load(Ordering::Relaxed), 0);
    Graph {
        runtime,
        source,
        model,
        nested,
    }
}
