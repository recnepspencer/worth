use std::collections::BTreeMap;

use worth_relational::facade::commit_strategies::{
    CommitStrategyId, CommitStrategyRegistration, IntentReconciliationStrategy,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    CreateIntent, EntitySpec, MutationIntent, WorkerIntentBatch,
};

pub fn public_relational_merge_runtime() -> RelationalRuntime {
    let descriptor = IntentReconciliationStrategy::descriptor(CommitStrategyId(913));
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(
            RelationalSchemaRegistry::new()
                .register_entity_kind(EntityKindRegistration {
                    kind_id: KindId(1),
                    kind_name: "public.merge.entity".to_string(),
                    schema_id: SchemaId("public-merge".to_string()),
                    schema_version_id: SchemaVersionId(1),
                    aspect_contract_declarations: KindAspectContractDeclarations::new(vec![]),
                })
                .expect("public merge entity kind should register"),
        )
        .commit_strategy(
            CommitStrategyRegistration::new(descriptor.clone())
                .expect("public merge strategy should register"),
        )
        .commit_strategy_executor(IntentReconciliationStrategy::execution_registration(
            &descriptor,
        ))
        .build();
    create_empty_entity(&mut runtime, "main", "main-seed");
    let (_, basis) = runtime
        .observe_fork_source(&BranchId("main".to_string()))
        .expect("main branch should expose an exact fork source");
    runtime
        .fork_branch(BranchId("candidate".to_string()), basis)
        .expect("candidate branch should be created");
    create_empty_entity(&mut runtime, "candidate", "candidate-seed");
    runtime
}

fn create_empty_entity(runtime: &mut RelationalRuntime, branch: &str, key: &str) {
    let branch_id = BranchId(branch.to_string());
    let mut transaction = {
        let identity = runtime
            .branch_identity(&branch_id)
            .expect("branch identity");
        let transaction_validation_input = runtime
            .admit_branch_basis(&identity)
            .expect("branch binding");
        runtime
            .begin_branch_transaction(
                &transaction_validation_input,
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("owner-admitted transaction context")
    };
    transaction
        .push_batch(
            WorkerIntentBatch::new(format!("create-{key}")).push(MutationIntent::Create(
                CreateIntent::Entity(EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: ClientKey::raw(key),
                    fields: BTreeMap::new().into(),
                }),
            )),
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let committed = transaction
        .commit(
            runtime,
            worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("public merge seed should commit");
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .expect("public merge seed snapshot should close exactly once");
}
