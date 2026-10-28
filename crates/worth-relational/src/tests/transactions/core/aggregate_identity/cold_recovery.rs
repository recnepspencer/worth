//! Aggregate consequences survive cold recovery and retain typed source failures.
use super::{cold_recovery_fixture::reopen, fixtures::*};
use crate::tests::support::*;
use crate::transactions::data::{
    CreatedRelationRef, EntityReference, RelationMutationIntent, UpdateRelationEndpointsIntent,
};
use std::sync::atomic::Ordering;

fn split(born: &CommitResult) -> WorkerIntentBatch {
    let ids = members(born, 4);
    let mut batch = WorkerIntentBatch::new("cold-aggregate-split");
    for (index, target) in [(1, 0), (3, 2)] {
        let relation = born
            .created_relation(&CreatedRelationRef {
                partition_id: PartitionId::main(),
                kind_id: KindId(2),
                client_key: crate::symbols::data::ClientKey::raw(format!("successor-{index}")),
                source: created(index),
                target: created((index + 1) % 4),
            })
            .unwrap();
        batch = batch.push(MutationIntent::Relation(
            RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                relation_id: relation,
                kind_id: KindId(2),
                source: EntityReference::Existing(ids[index]),
                target: EntityReference::Existing(ids[target]),
            }),
        ));
    }
    batch
}
#[test]
fn cold_split_and_warm_split_publish_identical_aggregate_names_without_history_loads() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let producer = persisted();
    let born = commit(execution, &producer, "main", birth(&[1, 2, 3, 0]));
    let (cold, source) = reopen(execution, &producer);
    let batch = split(&born);
    let warm = commit(execution, &producer, "main", batch.clone());
    let actual = commit(execution, &cold, "main", batch);
    assert!(
        source
            .reads
            .lock()
            .unwrap()
            .iter()
            .all(|id| !source.histories.contains(id)),
        "current aggregate mutation must not demand historical members"
    );
    assert_eq!(
        records(execution, &cold, &actual.snapshot),
        records(execution, &producer, &warm.snapshot)
    );
    assert_eq!(actual.envelope(), warm.envelope());
}
#[test]
fn cold_aggregate_source_refusal_publishes_nothing_and_can_retry() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let producer = persisted();
    let born = commit(execution, &producer, "main", birth(&[1, 2, 3, 0]));
    let (cold, source) = reopen(execution, &producer);
    let before = cold.history().branch_head(&BranchId("main".into()));
    source.fail_regions.store(true, Ordering::Relaxed);
    let mut transaction = test_owner_begin_transaction_for_main(&cold);
    transaction
        .push_batch(split(&born), execution.allocation_policy())
        .unwrap();
    let error = transaction.commit(execution, &cold).unwrap_err();
    let storage_refused = match &error {
        TransactionCommitError::Preparation { error, .. } => matches!(
            error.reason(),
            crate::transactions::data::CommitPreparationReason::StorageUnavailable(_)
        ),
        TransactionCommitError::Execution { denial, .. } => matches!(
            denial.kind,
            crate::transactions::data::CommitExecutionDenialKind::Cause(
                crate::execution::RelationalExecutionDenialCause::StorageUnavailable(_)
            )
        ),
        _ => false,
    };
    assert!(
        storage_refused,
        "source refusal must keep its typed cause: {error:?}"
    );
    assert_eq!(cold.history().branch_head(&BranchId("main".into())), before);
    assert!(cold.record_identity.pending_snapshot().is_empty());
    source.fail_regions.store(false, Ordering::Relaxed);
    let result = commit(execution, &cold, "main", split(&born));
    assert_eq!(
        records(execution, &cold, &result.snapshot)
            .iter()
            .filter(|(_, row)| row.current_anchor.is_some())
            .count(),
        2
    );
}
