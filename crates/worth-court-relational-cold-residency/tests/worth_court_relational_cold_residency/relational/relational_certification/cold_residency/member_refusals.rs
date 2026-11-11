//! Cold member failures cross real public Native recovery and transaction owners.
use std::sync::atomic::Ordering;
use worth_relational::facade::{
    durability::{RecoveryFailureClass, RecoveryVerificationMode, RelationalRecoveryStop},
    storage::RelationalStorageUnavailableCause as Cause,
    transactions::WorkerIntentBatch,
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction, cold_recovery::source_with_history,
    durable_medium::TestDurableMedium, durable_publication::durable_runtime,
};

#[test]
fn a_damaged_cold_member_is_a_sticky_typed_transaction_read_refusal() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let original = source_with_history(execution, 0, false);
    let counter = original.source;
    let entity = original.entity;
    counter.damaged.store(true, Ordering::Relaxed);
    let source = original.input;
    let receiving_medium = TestDurableMedium::new();
    let mut receiving = durable_runtime(&receiving_medium);
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiving
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        counter.reads.load(Ordering::Relaxed),
        0,
        "reopen must not inspect the damaged member"
    );
    let mut transaction = begin_main_transaction(&receiving);
    let first = transaction.read_entity(execution, entity).unwrap_err();
    let worth_relational::facade::mvcc::RelationalTransactionReadDenial::StorageUnavailable(
        failure,
    ) = first
    else {
        panic!("{first:?}")
    };
    assert_eq!(
        failure.subject,
        worth_relational::facade::storage::RelationalStorageSubject::Region(entity.partition_id)
    );
    assert_eq!(failure.cause, Cause::IntegrityMismatch);
    let reads = counter.reads.load(Ordering::Relaxed);
    counter.damaged.store(false, Ordering::Relaxed);
    let mut retry = begin_main_transaction(&receiving);
    assert_eq!(
        retry.read_entity(execution, entity).unwrap_err(),
        worth_relational::facade::mvcc::RelationalTransactionReadDenial::StorageUnavailable(
            failure
        )
    );
    assert_eq!(
        counter.reads.load(Ordering::Relaxed),
        reads,
        "damage cannot be silently retried"
    );
    let head = |runtime: &worth_relational::facade::runtime::RelationalRuntime| {
        let (_, basis) = runtime
            .observe_branch(&runtime.main_branch_identity())
            .unwrap();
        runtime
            .history()
            .branch_head_for_observation(&basis.observation())
            .unwrap()
    };
    let before = head(&receiving);
    let mut writer = begin_main_transaction(&receiving);
    writer
        .push_batch(
            WorkerIntentBatch::new("damaged-member-write").push(
                worth_relational::facade::transactions::MutationIntent::Entity(
                    worth_relational::facade::transactions::EntityMutationIntent::Delete(
                        worth_relational::facade::transactions::DeleteEntityIntent {
                            entity_id: entity,
                        },
                    ),
                ),
            ),
            execution.allocation_policy(),
        )
        .unwrap();
    let worth_relational::facade::transactions::TransactionCommitError::Preparation {
        error, ..
    } = writer.commit(execution, &receiving).unwrap_err()
    else {
        panic!("a cold write must refuse before publication")
    };
    assert_eq!(
        error.reason(),
        worth_relational::facade::transactions::CommitPreparationReason::StorageUnavailable(
            failure
        )
    );
    assert_eq!(head(&receiving), before);
    assert_eq!(counter.reads.load(Ordering::Relaxed), reads);
}

#[test]
fn tail_replay_refuses_a_damaged_touched_member_as_cold_storage_unavailable() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let original = source_with_history(execution, 0, true);
    original.source.damaged.store(true, Ordering::Relaxed);
    let source = original.input;
    let receiving_medium = TestDurableMedium::new();
    let mut receiving = durable_runtime(&receiving_medium);
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            execution,
            source,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let RelationalRecoveryStop::Durability(error) = receiving
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err()
    else {
        panic!("durable storage refusal required")
    };
    assert_eq!(error.class, RecoveryFailureClass::ColdStorageUnavailable);
    assert_eq!(
        error.storage_unavailable.unwrap().cause,
        Cause::IntegrityMismatch
    );
    assert!(
        {
            let (_, basis) = receiving
                .observe_branch(&receiving.main_branch_identity())
                .unwrap();
            receiving
                .history()
                .branch_head_for_observation(&basis.observation())
                .unwrap()
                .is_none()
        },
        "replay refusal must leave the receiving runtime untouched"
    );
}
