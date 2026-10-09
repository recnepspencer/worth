//! Actual existing-record normalization and rollback retain the same storage.
use crate::{
    facade::mvcc::RelationalTransactionIntent,
    tests::support::{create_entity, runtime_with_test_schema},
    transactions::data::{
        DeleteEntityIntent, EntityMutationIntent, MutationIntent, RollbackEffect, WorkerIntentBatch,
    },
};
use worth_execution::ExecutionAllocationPolicy as Allocation;

#[test]
fn no_raw_client_keys_preserve_staged_backing_and_exact_rollback_effects() {
    let runtime = runtime_with_test_schema();
    let first = create_entity(&runtime, "retained-existing-delete");
    let second = create_entity(&runtime, "discarded-existing-delete");
    let identity = runtime.main_branch_identity();
    let (head, basis) = runtime.observe_branch(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    assert!(transaction
        .client_key_symbol_policy
        .interns_requested_strings());
    let batch = |id| {
        WorkerIntentBatch::new("existing-delete").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: id }),
        ))
    };
    transaction
        .push_batch(batch(first), Allocation::SystemAllocation)
        .unwrap();
    let index = transaction.overlay.index.iter().next().unwrap() as *const _;
    let write = transaction.footprint().writes().next().unwrap() as *const _;
    assert_eq!(transaction.overlay.normalization_generation, 0);
    assert_eq!(
        transaction
            .merged_plan(&runtime, Allocation::SystemAllocation,)
            .unwrap()
            .merged_intents,
        batch(first).intents
    );
    assert_eq!(transaction.overlay.normalization_generation, 0);
    assert!(std::ptr::eq(
        index,
        transaction.overlay.index.iter().next().unwrap()
    ));
    assert!(std::ptr::eq(
        write,
        transaction.footprint().writes().next().unwrap()
    ));
    let savepoint = transaction.create_savepoint().unwrap();
    transaction
        .push_batch(batch(second), Allocation::SystemAllocation)
        .unwrap();
    assert_eq!(
        transaction
            .merged_plan(&runtime, Allocation::SystemAllocation,)
            .unwrap()
            .merged_intents
            .len(),
        2
    );
    assert_eq!(transaction.overlay.normalization_generation, 0);
    let rollback = transaction.rollback_to_savepoint(savepoint).unwrap();
    assert_eq!(
        rollback.effects(),
        &[RollbackEffect::RestoredEntity(second)]
    );
    assert_eq!(rollback.summary().restored_entity_count, 1);
    assert_eq!(rollback.summary().discarded_creation_count(), 0);
    assert_eq!(transaction.batches(), &[batch(first)]);
    assert!(std::ptr::eq(
        index,
        transaction.overlay.index.iter().next().unwrap()
    ));
    assert!(std::ptr::eq(
        write,
        transaction.footprint().writes().next().unwrap()
    ));
    assert_eq!(
        transaction
            .merged_plan(&runtime, Allocation::SystemAllocation,)
            .unwrap()
            .merged_intents,
        batch(first).intents
    );
    assert_eq!(transaction.overlay.normalization_generation, 0);
    assert_eq!(runtime.observe_branch(&identity).unwrap().0, head);
}
