//! Shared region cells confer no carried-member membership in another home.
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
use worth_store_application::ReadmittedApplicationHome;

fn seeded_home(
    path: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> ReadmittedApplicationHome {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "scope.seed".into(),
            schema_id: SchemaId("scope".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let home = worth_store_testkit_application_oracle::facade::home_opening::open(
        worth_store_testkit_application_oracle::facade::home_opening::request(
            path,
            RelationalRuntimeApi::builder().schema_setup(move |schema| {
                schema.schema_registry(registry);
            }),
        ),
        execution,
    )
    .unwrap();
    let (_, basis) = home
        .relational()
        .runtime()
        .observe_branch(&home.relational().runtime().main_branch_identity())
        .unwrap();
    let mut txn = home
        .relational()
        .runtime()
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
    txn.commit(execution, home.relational().runtime()).unwrap();
    home
}

#[test]
fn a_speculative_foreign_runtime_cannot_publish_a_home_checkpoint() {
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
            first.checkpoint(&execution).unwrap();
        }
        // A speculative fork deliberately has no attached publication medium.
        // Replacing a component cannot transfer the first Home's authority.
        second.replace_relational_runtime_for_test(first.relational().runtime().fork().unwrap());
        let before = crate::selected_checkpoint::generation(&second);
        let refusal = second.checkpoint(&execution).unwrap_err();
        assert!(matches!(refusal, worth_store_application::ApplicationHomeCheckpointDenial::Native(
            worth_store_relational::RelationalSectionCheckpointDenial::PublicationAdmission(
                worth_relational::facade::durability::RelationalCheckpointAcknowledgementDenial::CaptureNotBound))));
        assert_eq!(crate::selected_checkpoint::generation(&second), before);
        first.checkpoint(&execution).unwrap();
        first.close(&execution);
        second.close(&execution);
    }
}
