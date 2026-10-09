use crate::tests::support::*;

#[test]
fn truth_sequence_overflow_is_rejected_before_publication_effects() {
    let runtime = runtime_with_test_schema();
    let before_cells = runtime.history.branch_cells_snapshot();
    let before_catalog = runtime.history.catalog_len();
    let before_graph = runtime.history.commit_graph_len();
    let before_envelopes = runtime.history.recorded_commit_envelope_count();
    let before_patch_index = runtime.history.recorded_patch_position_count();
    let before_slots: usize = runtime
        .acquire_partition_edition()
        .partitions()
        .map(|partition| partition.entity_arena.slot_count())
        .sum();

    runtime
        .history
        .with_ledger_mut(|ledger| ledger.set_sequence(u64::MAX, u64::MAX));
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("overflow-denial"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("test staging stays within configured resource budgets");
    let error = transaction
        .commit(
            &runtime,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect_err("sequence overflow must be a typed publication denial");

    match &error {
        TransactionCommitError::Publication { error, .. } => {
            assert_eq!(error.stage, PublicationStage::BundleAssembly);
            assert!(error.detail.contains("sequence overflow"));
        }
        other => panic!("sequence overflow escaped as an untyped error: {other:?}"),
    }
    assert_eq!(runtime.history.branch_cells_snapshot(), before_cells);
    assert_eq!(runtime.history.catalog_len(), before_catalog);
    assert_eq!(runtime.history.commit_graph_len(), before_graph);
    assert_eq!(
        runtime.history.recorded_commit_envelope_count(),
        before_envelopes
    );
    assert_eq!(
        runtime.history.recorded_patch_position_count(),
        before_patch_index
    );
    let after_slots: usize = runtime
        .acquire_partition_edition()
        .partitions()
        .map(|partition| partition.entity_arena.slot_count())
        .sum();
    assert_eq!(after_slots, before_slots);
}
