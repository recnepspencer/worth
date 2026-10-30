#[allow(unused_imports)]
use crate::relational_certification::ordinary_context::mvcc_preparation_owner::*;
pub(crate) use worth_relational::facade::mvcc::RelationalTransactionIntent;
pub(crate) use worth_relational::facade::runtime::RelationalRuntimeApi;
pub(crate) use worth_relational::facade::symbols::ClientKey;
pub(crate) use {
    worth_relational::facade::identity::KindId, worth_relational::facade::identity::PartitionId,
};
pub(crate) use {
    worth_relational::facade::schema::EntityKindRegistration,
    worth_relational::facade::schema::KindAspectContractDeclarations,
    worth_relational::facade::schema::RelationalSchemaRegistry,
    worth_relational::facade::schema::SchemaId, worth_relational::facade::schema::SchemaVersionId,
};
pub(crate) use {
    worth_relational::facade::transactions::CreateIntent,
    worth_relational::facade::transactions::EntitySpec,
    worth_relational::facade::transactions::MutationIntent,
    worth_relational::facade::transactions::WorkerIntentBatch,
};

#[test]
fn production_preparation_port_observes_initial_schema_replacement() {
    let host_request =
        worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .build()
        .expect("valid native runtime configuration");
    let preparation = runtime.preparation_port();
    runtime
        .prepare_initial_schema_installation()
        .expect("initial schema installation prepares")
        .install(
            worth_relational::facade::schema::RelationalSchemaRegistry::new()
                .register_entity_kind(installed_entity_kind())
                .expect("installed entity kind is valid"),
        )
        .expect("initial schema installation succeeds");

    let identity = runtime.main_branch_identity();
    let basis = runtime
        .admit_branch_basis(&identity)
        .expect("installed main branch basis is admitted");
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .expect("transaction begins against the installed basis");
    transaction
        .push_batch(
            worth_relational::facade::transactions::WorkerIntentBatch::new(
                "production-held-preparation-port",
            )
            .push(
                worth_relational::facade::transactions::MutationIntent::Create(
                    worth_relational::facade::transactions::CreateIntent::Entity(EntitySpec {
                        partition_id: worth_relational::facade::identity::PartitionId::main(),
                        kind_id: KindId(7),
                        client_key: worth_relational::facade::symbols::ClientKey::raw(
                            "production-held-port-entity",
                        ),
                        fields: Default::default(),
                    }),
                ),
            ),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("transaction stages against the installed schema");

    let candidate = preparation
        .prepare_branch_transaction(execution, transaction)
        .expect("production port observes the atomically replaced schema world");
    preparation
        .discard_prepared_candidate(candidate)
        .expect("production port discards through the same live owner");
}
