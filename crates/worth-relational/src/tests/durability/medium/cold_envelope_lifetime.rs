//! Recovered nonhead bodies are retained by readers, never by the catalog.
use super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

#[test]
fn recovered_nonhead_envelope_custody_ends_with_the_last_inflight_read() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let (recovery, source, _) = fixture_with_history(execution, false, 16);
    source.damaged.store(false, Ordering::Relaxed);
    let position = *source.commits.keys().nth(3).unwrap();
    let mut runtime = durable_runtime_with_test_schema();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    let receipt = runtime
        .history()
        .immutable_commit_receipt_at_patch_stream_position(position)
        .unwrap();
    assert_ne!(
        runtime
            .history()
            .branch_head(&BranchId("main".into()))
            .unwrap()
            .commit_id,
        receipt.commit_id
    );
    let read = || {
        runtime
            .history()
            .demand_commit_envelope(execution, receipt.commit_id)
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
        Arc::ptr_eq(&first, &simultaneous),
        "in-flight demand shares the admitted body"
    );
    assert_eq!(source.commit_reads.lock().unwrap().len(), reads);
    let weak = Arc::downgrade(&first);
    drop(first);
    assert!(
        weak.upgrade().is_some(),
        "the second reader still has custody"
    );
    drop(simultaneous);
    assert!(
        weak.upgrade().is_none(),
        "a catalog index must not retain an old queried body"
    );
    let frontier = runtime.history().latest_patch_stream_position();
    let offered = attached_test_medium(&runtime).offered().len();
    assert_eq!(
        read().unwrap_err(),
        Execution::StorageUnavailable(RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Envelope(position),
            cause: Cause::SourceUnavailable,
        })
    );
    assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
    assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
    source.failures.lock().unwrap().clear();
    assert_eq!(read().unwrap().unwrap().commit, receipt);
}
