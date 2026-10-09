use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
};

#[test]
fn production_preparation_port_observes_initial_schema_replacement() {
    let mut runtime = RelationalRuntimeApi::builder().build();
    let preparation = runtime.preparation_port();
    runtime
        .prepare_initial_schema_installation()
        .expect("initial schema installation prepares")
        .install(
            RelationalSchemaRegistry::new()
                .register_entity_kind(installed_entity_kind())
                .expect("installed entity kind is valid"),
        )
        .expect("initial schema installation succeeds");

    let identity = runtime.main_branch_identity();
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("installed main branch basis is admitted");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("transaction begins against the installed basis");
    transaction
        .push_batch(
            WorkerIntentBatch::new("production-held-preparation-port").push(
                MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(7),
                    client_key: ClientKey::raw("production-held-port-entity"),
                    fields: Default::default(),
                })),
            ),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("transaction stages against the installed schema");

    let candidate = preparation
        .prepare_branch_transaction(
            transaction,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("production port observes the atomically replaced schema world");
    preparation
        .discard_prepared_candidate(candidate)
        .expect("production port discards through the same live owner");
}

fn installed_entity_kind() -> EntityKindRegistration {
    EntityKindRegistration {
        kind_id: KindId(7),
        kind_name: "production-held-port-installed".to_owned(),
        schema_id: SchemaId("production-preparation-owner".to_owned()),
        schema_version_id: SchemaVersionId(1),
        aspect_contract_declarations: KindAspectContractDeclarations::default(),
    }
}
