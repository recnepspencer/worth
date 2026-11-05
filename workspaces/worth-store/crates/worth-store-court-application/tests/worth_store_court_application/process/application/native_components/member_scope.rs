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
use worth_store_testkit_application_oracle::facade::world_home::WorldHomeFixture;

fn seeded_home(path: &std::path::Path, execution: &ExecutionRequest<'_, '_>) -> WorldHomeFixture {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "scope.seed".into(),
            schema_id: SchemaId("scope".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .unwrap();
    let fixture = super::coupled_home::open(
        path,
        RelationalRuntimeApi::builder().schema_setup(move |schema| {
            schema.schema_registry(registry);
        }),
        true,
        execution,
    );
    let home = &fixture.home;
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
    let committed = txn.commit(execution, home.relational().runtime()).unwrap();
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    fixture
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
        let first_fixture = seeded_home(&root.join("first"), &execution);
        let first = first_fixture.home;
        let second_fixture = seeded_home(&root.join("second"), &execution);
        let mut second = second_fixture.home;
        if checkpoint_before_fork {
            first.checkpoint(&execution).unwrap();
        }
        // A speculative fork deliberately has no attached publication medium.
        // Replacing a component cannot transfer the first Home's authority.
        second.replace_relational_runtime_for_test(first.relational().runtime().fork().unwrap());
        let before =
            crate::selected_checkpoint::generation_from_serving(second.physical_serving_for_test());
        let refusal = second.checkpoint(&execution).unwrap_err();
        assert!(matches!(&refusal, worth_store_application::ApplicationHomeCheckpointDenial::Native(
            worth_store_relational::RelationalSectionCheckpointDenial::Capture(
                worth_relational::facade::durability::RelationalNativeCheckpointCaptureDenial::OriginalAuthorityUnavailable))),
            "foreign Complete must refuse original source authority: {refusal:?}");
        assert_eq!(
            crate::selected_checkpoint::generation_from_serving(second.physical_serving_for_test()),
            before
        );
        // Release the deliberately foreign component before closing the Home
        // whose immutable source that speculative fork may still retain.
        drop(second);
        first.checkpoint(&execution).unwrap();
        first
            .close(&execution)
            .unwrap_or_else(|refused| panic!("World close: {:?}", refused.reason()));
    }
}
