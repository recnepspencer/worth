//! The real snapshot owner refuses duplicate release after publication.
use worth_execution::ExecutionRequest;
use worth_relational::facade::snapshots::RelationalSnapshotReleaseDenial;
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction,
    durable_publication::{batch_create, memory_runtime},
    shared_relational_certification_test_execution_policy::serial_request,
};

#[test]
fn duplicate_snapshot_release_retains_the_performed_commit_receipt() {
    let host = serial_request();
    let request = ExecutionRequest::serial(&host);
    let runtime = memory_runtime();
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            batch_create("closeout-refusal"),
            request.allocation_policy(),
        )
        .unwrap();
    let committed = transaction.commit(&request, &runtime).unwrap();
    runtime
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let release = runtime.snapshots().release_snapshot(&committed.snapshot);
    assert_eq!(
        release.unwrap_err(),
        RelationalSnapshotReleaseDenial::UnknownSnapshot
    );
    assert!(runtime
        .history()
        .immutable_commit_receipt(committed.commit.commit_id)
        .is_some());
}
