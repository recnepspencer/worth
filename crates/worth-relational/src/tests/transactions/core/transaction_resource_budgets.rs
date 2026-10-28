use crate::facade::config::PublicationConfig;
use crate::facade::mvcc::RelationalTransactionStagingDenial;
use crate::tests::support::*;

#[test]
fn savepoint_exhaustion_rejects_without_transaction_or_footprint_residue() {
    let runtime = runtime_with_savepoint_limit(1);
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

fn runtime_with_savepoint_limit(maximum_savepoints: usize) -> crate::runtime::RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: None,
            max_published_snapshot_handles: Some(8),
            max_active_snapshot_handles: Some(8),
            max_transaction_savepoints: maximum_savepoints,
            max_prepared_candidates: Some(8),
            candidate_max_lifetime_millis: 30_000,
        })
        .build()
        .expect("valid native runtime configuration")
}

#[test]
fn materialization_transaction_savepoints_refuse_without_consuming_identity_or_staging() {
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let intent = crate::mvcc::RelationalTransactionIntent::materialization(
        crate::mvcc::RelationalMaterializationTransactionMode::Suspend,
    );
    let mut transaction = runtime.begin_branch_transaction(&basis, intent).unwrap();
    for _ in 0..2 {
        assert_eq!(
            transaction.create_savepoint(),
            Err(
                RelationalTransactionStagingDenial::SavepointCapacityExhausted {
                    maximum_savepoints: 0
                }
            )
        );
    }
    assert!(transaction.batches().is_empty());
    assert_eq!(transaction.next_savepoint_ordinal, 1);
    assert!(transaction.savepoints.is_empty());
}
