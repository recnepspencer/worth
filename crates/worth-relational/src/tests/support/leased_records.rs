use super::*;

pub(crate) fn create_entity_outcome_with_lease(
    runtime: &RelationalRuntime,
    name: &str,
    lease: &worth_execution::ExecutionResourceLease<'_>,
) -> CommitResult {
    let fields = super::records::entity_fields_for_runtime(runtime, name);
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new(format!("batch-{name}")).push(MutationIntent::Create(
                CreateIntent::Entity(crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw(name),
                    fields,
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("leased entity stages");
    runtime
        .commit_branch_transaction_with_lease(transaction, lease)
        .expect("leased entity commits")
}
