//! Shared region cells confer no carried-member membership in another home.
use super::*;
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    identity::{KindId, PartitionId},
    mvcc::RelationalTransactionIntent,
    runtime::RelationalRuntimeApi,
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};

fn seeded_home(
    path: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> RelationalStoreHome {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "scope.seed".into(),
            schema_id: SchemaId("scope".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let home = RelationalStoreHome::open(
        crate::RelationalStoreHomeRequest::new(
            path,
            RelationalRuntimeApi::builder().schema_setup(move |schema| {
                schema.schema_registry(registry);
            }),
        ),
        execution,
    )
    .unwrap();
    let (_, basis) = home
        .runtime
        .observe_branch(&home.runtime.main_branch_identity())
        .unwrap();
    let mut txn = home
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    txn.push_batch(
        WorkerIntentBatch::new("scope").push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_key: ClientKey::raw("seed"),
                fields: Default::default(),
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    txn.commit(execution, &home.runtime).unwrap();
    home
}

#[test]
fn forked_cells_checkpoint_into_two_homes_without_foreign_carried_locators() {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    for checkpoint_before_fork in [false, true] {
        let root = directory.path().join(checkpoint_before_fork.to_string());
        let first = seeded_home(&root.join("first"), &execution);
        let mut second = seeded_home(&root.join("second"), &execution);
        if checkpoint_before_fork {
            first
                .checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
        }
        // Both journals contain the real seed. Their live roots now share
        // cells, while Store membership is owned by each destination home.
        second.runtime = first.runtime.fork().unwrap();
        for _ in 0..2 {
            for home in [&first, &second] {
                let before = home.cold_storage_reads();
                home.checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
                    .unwrap();
                assert_eq!(
                    home.cold_storage_reads(),
                    before,
                    "capture cannot load foreign members"
                );
            }
        }
        first.close();
        second.close();
    }
}
