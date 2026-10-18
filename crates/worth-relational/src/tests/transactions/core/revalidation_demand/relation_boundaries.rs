//! Original request and staging custody for relation-only read demands.
use super::relation_fixtures::*;
use crate::tests::support::*;
use worth_execution::{ExecutionAllocationPolicy as Policy, ExecutionRequest};

#[test]
fn relation_demand_refused_backing_keeps_footprint_and_rollback_reports_no_write() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = runtime();
    let entity = create_entity(execution, &runtime, "lax");
    let relation = create_relation(execution, &runtime, entity, entity, "ok");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    let before = transaction.footprint().clone();
    let lease = test_execution_authority()
        .request_lease(worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(1).unwrap(),
                    0,
                    1,
                ),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    assert!(transaction
        .push_batch(batch(relation), Policy::Execution(&lease))
        .unwrap_err()
        .allocation_denial()
        .is_some());
    assert!(transaction.batches().is_empty());
    assert_eq!(transaction.footprint(), &before);
    let savepoint = transaction.create_savepoint().unwrap();
    transaction
        .push_batch(batch(relation), Policy::SystemAllocation)
        .unwrap();
    assert!(!transaction
        .rollback_to_savepoint(savepoint)
        .unwrap()
        .has_effects());
    transaction
        .push_batch(batch(relation), Policy::SystemAllocation)
        .unwrap();
    let outcome = transaction.commit(execution, &runtime).unwrap();
    assert!(outcome.changed_records.is_empty());
    release_test_commit_snapshot(&runtime, &outcome);
}

#[test]
fn cancelled_original_relation_request_cannot_publish_and_fresh_request_succeeds() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let runtime = runtime();
    let entity = create_entity(execution, &runtime, "lax");
    let relation = create_relation(execution, &runtime, entity, entity, "ok");
    let before = runtime.current_version_id();
    let source = worth_execution::CancellationSource::new();
    let stopped = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(64 * 1024 * 1024),
        source.token(),
        None,
    );
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch(relation), Policy::SystemAllocation)
        .unwrap();
    source.cancel();
    let error = transaction
        .commit(&ExecutionRequest::serial(&stopped), &runtime)
        .unwrap_err();
    assert!(
        matches!(
            error,
            TransactionCommitError::Execution {
                denial: crate::transactions::data::CommitExecutionDenial {
                    kind: crate::transactions::data::CommitExecutionDenialKind::Cause(
                        crate::execution::RelationalExecutionDenialCause::Cancelled
                    ),
                    ..
                },
                ..
            }
        ),
        "original cancellation: {error:?}"
    );
    assert_eq!(runtime.current_version_id(), before);
    let mut retry = test_owner_begin_transaction_for_main(&runtime);
    retry
        .push_batch(batch(relation), Policy::SystemAllocation)
        .unwrap();
    let outcome = retry.commit(execution, &runtime).unwrap();
    assert!(outcome.changed_records.is_empty());
    release_test_commit_snapshot(&runtime, &outcome);
}
