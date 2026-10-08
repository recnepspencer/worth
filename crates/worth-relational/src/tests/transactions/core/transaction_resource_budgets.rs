use crate::facade::config::PublicationConfig;
use crate::facade::mvcc::RelationalTransactionStagingDenial;
use crate::tests::support::*;

#[test]
fn overlay_byte_exhaustion_rejects_without_partial_staging() {
    let runtime = runtime_with_limits(0, 8, 8);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);

    assert!(matches!(
        transaction.push_batch(batch_create("overlay-red-control"), worth_execution::ExecutionAllocationPolicy::SystemAllocation),
        Err(RelationalTransactionStagingDenial::OverlayCapacityExhausted {
            maximum_bytes: 0,
            required_bytes,
        }) if required_bytes > 0
    ));
    assert!(transaction.batches().is_empty());
    assert_eq!(transaction.footprint().writes().len(), 0);
}

#[test]
fn footprint_exhaustion_rejects_without_overlay_or_index_residue() {
    let runtime = runtime_with_limits(1_048_576, 0, 8);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);

    assert_eq!(
        transaction.push_batch(
            batch_create("footprint-red-control"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        ),
        Err(
            RelationalTransactionStagingDenial::FootprintCapacityExhausted {
                maximum_loci: 0,
                required_loci: 1,
            }
        )
    );
    assert!(transaction.batches().is_empty());
    assert_eq!(transaction.footprint().writes().len(), 0);
}

#[test]
fn savepoint_exhaustion_rejects_without_transaction_or_footprint_residue() {
    let runtime = runtime_with_limits(1_048_576, 8, 1);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("savepoint-budget-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let first = transaction.create_savepoint().unwrap();
    let batch_count = transaction.batches().len();
    let write_count = transaction.footprint().writes().len();

    assert_eq!(
        transaction.create_savepoint(),
        Err(
            RelationalTransactionStagingDenial::SavepointCapacityExhausted {
                maximum_savepoints: 1,
            }
        )
    );
    assert_eq!(transaction.batches().len(), batch_count);
    assert_eq!(transaction.footprint().writes().len(), write_count);
    transaction.rollback_to_savepoint(first).unwrap();
}

#[test]
fn cumulative_savepoint_footprints_deny_before_an_unbounded_clone() {
    let runtime = runtime_with_limits(1_048_576, 2, 8);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("savepoint-footprint-write"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction.create_savepoint().unwrap();
    transaction.create_savepoint().unwrap();
    let batch_count = transaction.batches().len();
    let write_count = transaction.footprint().writes().len();

    assert_eq!(
        transaction.create_savepoint(),
        Err(
            RelationalTransactionStagingDenial::SavepointFootprintCapacityExhausted {
                maximum_loci: 2,
                required_loci: 3,
            }
        )
    );
    assert_eq!(transaction.batches().len(), batch_count);
    assert_eq!(transaction.footprint().writes().len(), write_count);
}

fn runtime_with_limits(
    maximum_overlay_bytes: u64,
    maximum_footprint_loci: usize,
    maximum_savepoints: usize,
) -> crate::runtime::RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 8,
            max_active_snapshot_handles: 8,
            max_transaction_overlay_bytes: maximum_overlay_bytes,
            max_transaction_footprint_loci: maximum_footprint_loci,
            max_transaction_savepoints: maximum_savepoints,
            max_prepared_candidates: 8,
            candidate_max_lifetime_millis: 30_000,
            max_prepared_root_bytes: 268_435_456,
        })
        .build()
}
