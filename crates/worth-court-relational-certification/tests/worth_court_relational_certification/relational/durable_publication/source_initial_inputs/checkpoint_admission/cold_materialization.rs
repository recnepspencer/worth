//! Public cold DATA admission, value and provider-custody controls.
use super::super::selected_source::{CustodyControl, SelectedSource};
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    mvcc::RelationalTransactionReadDenial,
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction, durable_publication::field_patch,
};
pub(crate) mod byte_fixture;
mod selected_image;
mod shared_custody;
pub(crate) use selected_image::{member, recover};

pub(crate) fn update(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
) {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("retain-cold-backing").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: field_patch("name", "name", "edited"),
                }),
            )),
            request.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(request, runtime).unwrap();
    release_test_commit_snapshot(runtime, &result);
}

pub(crate) fn read_head(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity: EntityId,
) -> Result<(), RelationalTransactionReadDenial> {
    let mut transaction = begin_main_transaction(runtime);
    assert!(transaction.read_entity(request, entity)?.base().is_some());
    Ok(())
}
