use super::*;
use worth_relational::facade::{
    history::BranchId,
    identity::{EntityId, KindId, PartitionId},
    runtime::RelationalRuntimeApi,
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::{AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};

#[test]
fn history_basis_requires_same_owner_branch_version_and_live_candidate() {
    let mut runtime = history_runtime();
    let first = commit(&mut runtime, "first");
    let retained = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let second_lease = runtime
        .snapshots()
        .snapshot_for_observation(&retained.observation())
        .unwrap();
    assert_ne!(first.snapshot_id(), second_lease.snapshot_id());
    assert!(remains_equal(&runtime, &second_lease, &first, 2, 1));
    assert!(!remains_equal(&runtime, &second_lease, &first, 1, 1));
    let (_, source) = runtime.observe_fork_source(first.branch_id()).unwrap();
    let fork = runtime
        .fork_branch(BranchId("history-fork".to_owned()), source)
        .unwrap();
    let fork_basis = runtime.admit_branch_basis(fork.target_identity()).unwrap();
    let fork_snapshot = runtime
        .snapshots()
        .snapshot_for_observation(&fork_basis.observation())
        .unwrap();
    assert_eq!(fork_snapshot.version_id(), first.version_id());
    assert!(!remains_equal(&runtime, &fork_snapshot, &first, 2, 1));
    let changed = commit(&mut runtime, "second");
    assert!(!remains_equal(&runtime, &changed, &first, 2, 1));
    assert!(remains_equal(&runtime, &second_lease, &first, 2, 1));
    let mut foreign = history_runtime();
    let foreign_snapshot = commit(&mut foreign, "first");
    assert_eq!(foreign_snapshot.version_id(), first.version_id());
    assert!(!remains_equal(&foreign, &foreign_snapshot, &first, 2, 1));
    assert!(!remains_equal(&foreign, &second_lease, &first, 2, 1));
    crate::relational_snapshot_release::release_query_snapshot(&mut runtime, &second_lease);
    assert!(!remains_equal(&runtime, &second_lease, &first, 2, 1));
    // An invalid lease must be distinguished before reading any anchor.
    assert_eq!(
        super::super::observe_adjacency_checked(
            &runtime,
            &second_lease,
            KindId::new(1),
            EntityId::new(PartitionId::main(), 0, 1),
            super::super::WorthQueryApplicationAdjacencyDirection::Outgoing,
            0,
        ),
        Err(super::super::AdjacencyObservationDenial::SnapshotUnavailable)
    );
    crate::relational_snapshot_release::release_query_snapshot(&mut runtime, &first);
    crate::relational_snapshot_release::release_query_snapshot(&mut runtime, &changed);
    crate::relational_snapshot_release::release_query_snapshot(&mut runtime, &fork_snapshot);
    crate::relational_snapshot_release::release_query_snapshot(&mut foreign, &foreign_snapshot);
}

fn history_runtime() -> RelationalRuntime {
    let schema = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId::new(1),
            kind_name: "history".to_owned(),
            schema_id: SchemaId("history-basis".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap();
    RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
}

fn commit(runtime: &mut RelationalRuntime, name: &str) -> SnapshotHandle {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(WorkerIntentBatch::new(name).push(MutationIntent::Create(
            CreateIntent::Entity(EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId::new(1),
                client_key: ClientKey::raw(name),
                fields: AspectFieldPatch::default(),
            }),
        )))
        .unwrap();
    transaction.commit(runtime).unwrap().snapshot.clone()
}
