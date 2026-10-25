//! A bounded history operation demands its selected journal envelope only.
use super::*;
use crate::execution::RelationalExecutionDenialCause;
use crate::history::BoundedCanonicalCommitPatchDenial as Denial;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};
use std::num::NonZeroUsize;

#[test]
fn bounded_history_reads_only_the_selected_envelope_and_refuses_sticky_journal_damage() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for damaged in [false, true] {
        let (source, counter, _) = fixture_with_history(execution, false, 16);
        let selected_bytes = counter.commits.values().nth(3).unwrap().clone();
        let selected = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(selected_bytes).unwrap();
        let commit = selected.commit_id();
        let position = selected.position();
        let mut runtime = durable_runtime_with_test_schema();
        let plan = runtime
            .durability()
            .recovery_plan_from_source(
                execution,
                source,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        runtime
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        let reads_before = counter.commit_reads.lock().unwrap().len();
        let receipt = runtime
            .history()
            .immutable_commit_receipt_at_patch_stream_position(position)
            .unwrap();
        assert_eq!(receipt.commit_id, selected.commit_id());
        assert_eq!(counter.commit_reads.lock().unwrap().len(), reads_before);
        let member_reads_before = counter.reads.load(Ordering::Relaxed);
        assert!(
            reads_before < 4,
            "reopen must not read the history population"
        );
        let offered_before = attached_test_medium(&runtime).offered().len();
        let frontier = runtime.history().latest_patch_stream_position();
        counter.commits_damaged.store(damaged, Ordering::Relaxed);
        let read = || {
            runtime.history().bounded_canonical_commit_patches(
                execution,
                commit,
                NonZeroUsize::new(32).unwrap(),
            )
        };
        if damaged {
            let expected = Denial::ReadRequest(RelationalExecutionDenialCause::StorageUnavailable(
                RelationalStorageUnavailable {
                    subject: RelationalStorageSubject::Envelope(position),
                    cause: Cause::Damaged,
                },
            ));
            assert_eq!(read().unwrap_err(), expected);
            counter.commits_damaged.store(false, Ordering::Relaxed);
            assert_eq!(read().unwrap_err(), expected);
        } else {
            let patches = read().unwrap();
            assert!(!patches.is_empty());
            assert_eq!(read().unwrap(), patches);
        }
        let reads = counter.commit_reads.lock().unwrap();
        assert!(!reads[reads_before..].is_empty());
        assert!(reads[reads_before..].iter().all(|read| *read == position));
        assert!(
            reads.len() - reads_before < 3,
            "warm reads and sticky damage do not reload"
        );
        assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
        assert_eq!(counter.reads.load(Ordering::Relaxed), member_reads_before);
        assert_eq!(
            attached_test_medium(&runtime).offered().len(),
            offered_before
        );
    }
}

#[test]
fn patch_stream_demands_only_the_requested_batch_and_refuses_sticky_journal_damage() {
    use crate::publication::patch::data::{PatchStreamReadErrorClass, PatchStreamRequest};
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for damaged in [false, true] {
        let (source, counter, _) = fixture_with_history(execution, false, 16);
        let selected = crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(counter.commits.values().nth(3).unwrap().clone()).unwrap();
        let position = selected.position();
        let after_position = *counter.commits.keys().nth(2).unwrap();
        let mut runtime = durable_runtime_with_test_schema();
        let plan = runtime
            .durability()
            .recovery_plan_from_source(
                execution,
                source,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        runtime
            .durability_recovery()
            .recover(execution, plan)
            .unwrap();
        let reads_before = counter.commit_reads.lock().unwrap().len();
        let receipt = runtime
            .history()
            .immutable_commit_receipt_at_patch_stream_position(position)
            .unwrap();
        assert_eq!(receipt.commit_id, selected.commit_id());
        assert_eq!(counter.commit_reads.lock().unwrap().len(), reads_before);
        assert!(
            reads_before < 4,
            "reopen must not read the history population"
        );
        let member_reads = counter.reads.load(Ordering::Relaxed);
        let frontier = runtime.history().latest_patch_stream_position();
        let offered = attached_test_medium(&runtime).offered().len();
        counter.commits_damaged.store(damaged, Ordering::Relaxed);
        let read = || {
            runtime.publication().read_patch_stream(
                execution,
                PatchStreamRequest {
                    after_position: Some(after_position),
                    max_commits: 1,
                },
            )
        };
        if damaged {
            let failure = read().unwrap_err();
            assert_eq!(
                failure.class,
                PatchStreamReadErrorClass::ReadRequest(
                    RelationalExecutionDenialCause::StorageUnavailable(
                        RelationalStorageUnavailable {
                            subject: RelationalStorageSubject::Envelope(position),
                            cause: Cause::Damaged,
                        }
                    )
                )
            );
            counter.commits_damaged.store(false, Ordering::Relaxed);
            assert_eq!(read().unwrap_err(), failure);
        } else {
            let batch = read().unwrap();
            assert_eq!(batch.patches.len(), 1);
            assert_eq!(batch.patches[0].position, position);
            assert_eq!(batch.latest_position, frontier);
            assert_eq!(read().unwrap(), batch);
        }
        let reads = counter.commit_reads.lock().unwrap();
        assert!(!reads[reads_before..].is_empty());
        assert!(reads[reads_before..].iter().all(|read| *read == position));
        assert!(
            reads.len() - reads_before < 3,
            "warm reads and sticky damage do not reload"
        );
        assert_eq!(counter.reads.load(Ordering::Relaxed), member_reads);
        assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
        assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
    }
}
