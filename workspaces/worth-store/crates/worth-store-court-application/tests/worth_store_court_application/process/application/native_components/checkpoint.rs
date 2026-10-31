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
    let home = worth_store_testkit_application_oracle::facade::home_opening::open(
        worth_store_testkit_application_oracle::facade::home_opening::request(
            directory.path().join("home"),
            RelationalRuntimeApi::builder().schema_setup(move |schema| {
                schema.schema_registry(registry);
            }),
        ),
        &execution,
    )
    .unwrap();
    let (_, basis) = home
        .relational()
        .runtime()
        .observe_branch(&home.relational().runtime().main_branch_identity())
        .unwrap();
    let mut transaction = home
        .relational()
        .runtime()
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
    transaction
        .commit(&execution, home.relational().runtime())
        .unwrap();
    let earlier = home
        .relational()
        .runtime()
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            ExecutionAllocationPolicy::SystemAllocation,
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let (_, source) = home
        .relational()
        .runtime()
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    home.relational()
        .runtime()
        .fork_branch(&execution, BranchId("later".into()), source)
        .unwrap();
    let selected = home.checkpoint(&execution).unwrap();
    assert!(selected.covered_transitions().0 > earlier.covered_branch_transitions().unwrap().0);
    let refused = home
        .publish_captured_relational_checkpoint_for_test(earlier, &execution)
        .unwrap_err();
    assert!(matches!(
        refused,
        worth_store_application::ApplicationHomeCheckpointDenial::Native(
            worth_store_relational::RelationalSectionCheckpointDenial::StaleCapture
        )
    ));
    let records = home.physical_serving_for_test().anchored_records().unwrap();
    let record = records
        .read(
            worth_store::physical_runtime::AnchoredRecordName::new("home.checkpoint").unwrap(),
            1 << 30,
        )
        .unwrap()
        .unwrap();
    assert_eq!(record.version().generation(), selected.generation());
    drop(record);
    home.close(&execution);
}
