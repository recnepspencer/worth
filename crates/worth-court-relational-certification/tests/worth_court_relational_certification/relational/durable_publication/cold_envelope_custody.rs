//! Public shared-body identity, funding and provider observations protect custody.
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    storage::{
        RelationalStorageSubject, RelationalStorageUnavailable,
        RelationalStorageUnavailableCause as Cause,
    },
    transactions::RelationalExecutionDenialCause as Execution,
};
use worth_testkit_relational::{
    cold_recovery::{recovered_with_history, ColdRecoveryFixture},
    durable_medium::immutable_members::ColdRead,
};

#[test]
fn recovered_nonhead_envelope_custody_ends_with_the_last_inflight_read() {
    let setup = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let ColdRecoveryFixture {
        runtime,
        medium,
        source,
        ..
    } = recovered_with_history(&ExecutionRequest::serial(&setup), 16);
    let position = *source.commits.keys().nth(3).unwrap();
    let receipt = runtime
        .history()
        .immutable_commit_receipt_at_patch_stream_position(position)
        .unwrap();
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    assert_ne!(
        basis
            .reference()
            .target()
            .as_basis()
            .unwrap()
            .selected_commit_id(),
        receipt.commit_id.0
    );
    drop(basis);
    // Same unlimited policy as the original court; an independent ledger isolates
    // the actual read's backing from producer/recovery allocations.
    let budget = SerialMemoryBudget::new(None);
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = &ExecutionRequest::serial(&host);
    assert_eq!(budget.charged_memory_bytes(), 0);
    let read = || {
        runtime
            .replay()
            .canonical_commit_envelope(request, receipt.commit_id)
    };
    let first = read().unwrap().unwrap();
    assert_eq!(first.commit, receipt);
    let reads = source.commit_reads.lock().unwrap().len();
    source
        .failures
        .lock()
        .unwrap()
        .insert(ColdRead::Commit(position), Cause::SourceUnavailable);
    let simultaneous = read().unwrap().unwrap();
    assert!(
        std::ptr::eq(first.as_ref(), simultaneous.as_ref()),
        "in-flight demand shares the admitted body"
    );
    assert_eq!(source.commit_reads.lock().unwrap().len(), reads);
    let held = budget.charged_memory_bytes();
    assert!(held > 0);
    drop(first);
    assert_eq!(
        simultaneous.commit, receipt,
        "second reader retains the same body"
    );
    assert_eq!(budget.charged_memory_bytes(), held);
    drop(simultaneous);
    assert!(
        budget.charged_memory_bytes() < held,
        "the catalog must not retain the admitted body payload"
    );
    let frontier = runtime.history().latest_patch_stream_position();
    let offered = medium.offered().len();
    assert_eq!(
        read().unwrap_err(),
        Execution::StorageUnavailable(RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(position),
            cause: Cause::SourceUnavailable,
        })
    );
    assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
    assert_eq!(medium.offered().len(), offered);
    source.failures.lock().unwrap().clear();
    assert_eq!(read().unwrap().unwrap().commit, receipt);
    drop(runtime);
    assert_eq!(
        budget.charged_memory_bytes(),
        0,
        "fixed weak-observer control backing releases with its owner"
    );
}
