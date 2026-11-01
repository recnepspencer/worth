use worth_relational::facade::{
    identity::{KindId, PartitionId},
    mvcc::RelationalTransactionIntent,
    runtime::{RelationalRuntime, RelationalRuntimeApi, RelationalRuntimeDefinition},
    schema::{
        EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
        SchemaVersionId,
    },
    symbols::ClientKey,
    transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
};

pub(super) fn definition() -> RelationalRuntimeDefinition {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "Record".into(),
            schema_id: SchemaId("HomeSchema".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    RelationalRuntimeApi::builder()
        .schema_setup(move |setup| {
            setup.schema_registry(registry);
        })
        .resolve()
}

pub(super) fn commit(
    runtime: &RelationalRuntime,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    key: &str,
) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("home.journey").push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(key),
                    fields: Default::default(),
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let completed = transaction.commit(execution, runtime).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&completed.snapshot)
        .unwrap();
}

pub(super) fn artifacts(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) -> Vec<Vec<u8>> {
    runtime
        .durability()
        .canonical_commit_artifacts(execution)
        .map(|artifact| artifact.unwrap().as_bytes().to_vec())
        .collect()
}
