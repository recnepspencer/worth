use crate::facade::identity::{EntityId, KindId, PartitionId};
use crate::facade::runtime::RelationalRuntime;
use crate::facade::transactions::{
    AspectFieldPatch, BulkEntityCreateIntent, CommitResult, CreateIntent, MutationIntent,
    RecordRef, WorkerIntentBatch,
};

pub(super) fn bulk_create_entities<I>(
    runtime: &RelationalRuntime,
    batch_name: &str,
    partition_id: PartitionId,
    specs: I,
) -> Vec<EntityId>
where
    I: IntoIterator<Item = (String, AspectFieldPatch)>,
{
    let (client_keys, field_patches): (Vec<_>, Vec<_>) = specs
        .into_iter()
        .map(|(key, fields)| (crate::facade::symbols::ClientKey::raw(key), fields))
        .unzip();
    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(runtime);
    txn.push_batch(
        WorkerIntentBatch::new(batch_name).push(MutationIntent::Create(
            CreateIntent::BulkEntities(BulkEntityCreateIntent {
                partition_id,
                kind_id: KindId(1),
                client_keys,
                field_patches,
            }),
        )),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");
    changed_entities(
        &txn.commit(
            runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap(),
    )
}

fn changed_entities(outcome: &CommitResult) -> Vec<EntityId> {
    outcome
        .changed_records
        .iter()
        .filter_map(|record| match record {
            RecordRef::Entity(id) => Some(*id),
            RecordRef::Relation(_) => None,
        })
        .collect()
}
