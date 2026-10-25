use super::*;
use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy, ExecutionRequest, SerialMemoryBudget,
    SerialRequest,
};
use worth_relational::facade::{
    durability::RelationalCheckpointArtifactPolicy, history::BranchId,
    runtime::RelationalRuntimeApi,
};

#[test]
fn a_late_capture_cannot_regress_the_selected_transition_frontier_or_membership() {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    use worth_relational::facade::{
        identity::{KindId, PartitionId},
        mvcc::RelationalTransactionIntent,
        schema::{
            EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry,
            SchemaId, SchemaVersionId,
        },
        symbols::ClientKey,
        transactions::{CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch},
    };
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "checkpoint.seed".into(),
            schema_id: SchemaId("checkpoint".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let home = RelationalStoreHome::open(
        crate::RelationalStoreHomeRequest::new(
            directory.path().join("home"),
            RelationalRuntimeApi::builder().schema_setup(move |schema| {
                schema.schema_registry(registry);
            }),
        ),
        &execution,
    )
    .unwrap();
    let (_, basis) = home
        .runtime
        .observe_branch(&home.runtime.main_branch_identity())
        .unwrap();
    let mut transaction = home
        .runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("seed").push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw("seed"),
                    fields: Default::default(),
                },
            ))),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.commit(&execution, &home.runtime).unwrap();
    let earlier = home
        .runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            ExecutionAllocationPolicy::SystemAllocation,
            RelationalCheckpointArtifactPolicy::ExternalMembers,
        )
        .unwrap();
    let (_, source) = home
        .runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    home.runtime
        .fork_branch(&execution, BranchId("later".into()), source)
        .unwrap();
    let selected = home
        .checkpoint(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(selected.covered_transitions().0 > earlier.covered_branch_transitions().unwrap().0);
    let refused = checkpoint_record::publish(
        &home.serving,
        &home.cold_source,
        earlier.covered_position().unwrap(),
        earlier.covered_branch_transitions().unwrap(),
        &earlier,
    )
    .unwrap_err();
    assert!(matches!(
        refused,
        RelationalStoreCheckpointRecordRefusal::StaleCapture
    ));
    let records = home.serving.anchored_records().unwrap();
    let record = records
        .read(
            worth_store::physical_runtime::AnchoredRecordName::new("relational.checkpoint")
                .unwrap(),
            1 << 30,
        )
        .unwrap()
        .unwrap();
    assert_eq!(record.version().generation(), selected.generation());
    drop(record);
    drop(records);
    home.close();
}
