//! Naming admission uses the owning runtime without publishing any counters.
use super::*;
use worth_testkit_relational::aspect_storage::publications::begin_main_transaction;

#[test]
fn naming_admission_does_not_mutate_runtime_counters() {
    let runtime = memory_runtime();
    let before = runtime.observe_complexity_counters();
    let mut txn = begin_main_transaction(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("bulk").push(MutationIntent::Create(CreateIntent::BulkEntities(
            BulkEntityCreateIntent {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                client_keys: vec![ClientKey::raw("raw-key")],
                field_patches: vec![field_patch("name", "name", "raw")],
            },
        ))),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let admitted = txn
        .admit_naming_stable_bulk_mutation_batch(&runtime)
        .expect("admission should succeed");
    let after = runtime.observe_complexity_counters();

    assert!(admitted.is_some());
    assert_eq!(
        after.bulk_mutation_batch_count,
        before.bulk_mutation_batch_count
    );
    assert_eq!(
        after.bulk_mutation_naming_normalization_count,
        before.bulk_mutation_naming_normalization_count
    );
    assert_eq!(
        after.bulk_mutation_lineage_transition_count,
        before.bulk_mutation_lineage_transition_count
    );
}
