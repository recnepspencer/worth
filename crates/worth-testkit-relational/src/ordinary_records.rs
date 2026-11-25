//! Named records for facade courts using the shared aspect schema.
use crate::{
    aspect_storage::publications::{
        begin_main_transaction, changed_entities, release_test_commit_snapshot,
    },
    durable_publication::create_entity_outcome,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::EntityId,
    runtime::RelationalRuntime,
    transactions::{
        CommitResult, DeleteEntityIntent, EntityMutationIntent, MutationIntent, WorkerIntentBatch,
    },
};

pub fn create_entity(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    key: &str,
) -> EntityId {
    let commit = create_entity_outcome(execution, runtime, key);
    let entity = changed_entities(&commit)[0];
    release_test_commit_snapshot(runtime, &commit);
    entity
}

pub fn delete_entity(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: EntityId,
) -> CommitResult {
    let mut transaction = begin_main_transaction(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("delete").push(MutationIntent::Entity(
                EntityMutationIntent::Delete(DeleteEntityIntent { entity_id }),
            )),
            execution.allocation_policy(),
        )
        .expect("delete stages within the original request");
    transaction.commit(execution, runtime).unwrap()
}
