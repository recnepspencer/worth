//! A bounded history operation demands its selected journal envelope only.
use std::num::NonZeroUsize;
use std::sync::atomic::Ordering;
use worth_relational::facade::durability::RecoveryVerificationMode;
use worth_relational::facade::history::BoundedCanonicalCommitPatchDenial as Denial;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
use worth_relational::facade::storage::{RelationalStorageSubject, RelationalStorageUnavailable};
use worth_relational::facade::transactions::RelationalExecutionDenialCause;
use worth_testkit_relational::{
    cold_recovery::source_with_history, durable_medium::TestDurableMedium,
    durable_publication::durable_runtime,
};

#[test]
fn bounded_history_reads_only_the_selected_envelope_and_refuses_sticky_body_damage() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for damaged in [false, true] {
        let original = source_with_history(execution, 16, false);
        let counter = original.source;
        counter.damaged.store(true, Ordering::Relaxed);
        let source = original.input;
        let selected_bytes = counter.commits.values().nth(3).unwrap().clone();
        let selected = worth_relational::facade::durability::RelationalCanonicalCommitArtifact::from_untrusted_bytes(selected_bytes).unwrap();
        let commit = selected.commit_id();
        let position = selected.position();
        let medium = TestDurableMedium::new();
        let mut runtime = durable_runtime(&medium);
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
        assert!(
            counter.journal_reads.lock().unwrap().is_empty(),
            "selected history never falls back to its retired journal"
        );
        let selected_before = counter.canonical_member_reads.lock().unwrap().len();
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
        let offered_before = medium.offered().len();
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
        assert!(counter.journal_reads.lock().unwrap().is_empty());
        assert!(counter.canonical_member_reads.lock().unwrap().len() > selected_before);
        let reads = counter.commit_reads.lock().unwrap();
        assert!(!reads[reads_before..].is_empty());
        assert!(reads[reads_before..].iter().all(|read| *read == position));
        assert!(
            reads.len() - reads_before < 3,
            "warm reads and sticky damage do not reload"
        );
        assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
        assert_eq!(counter.reads.load(Ordering::Relaxed), member_reads_before);
        assert_eq!(medium.offered().len(), offered_before);
    }
}

#[test]
fn patch_stream_demands_only_the_requested_batch_and_refuses_sticky_journal_damage() {
    use worth_relational::facade::publication::{PatchStreamReadErrorClass, PatchStreamRequest};
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for damaged in [false, true] {
        let original = source_with_history(execution, 16, false);
        let counter = original.source;
        counter.damaged.store(true, Ordering::Relaxed);
        let source = original.input;
        let selected = worth_relational::facade::durability::RelationalCanonicalCommitArtifact::from_untrusted_bytes(counter.commits.values().nth(3).unwrap().clone()).unwrap();
        let position = selected.position();
        let after_position = *counter.commits.keys().nth(2).unwrap();
        let medium = TestDurableMedium::new();
        let mut runtime = durable_runtime(&medium);
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
        assert!(
            counter.journal_reads.lock().unwrap().is_empty(),
            "selected history never falls back to its retired journal"
        );
        let selected_before = counter.canonical_member_reads.lock().unwrap().len();
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
        let offered = medium.offered().len();
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
        assert!(counter.journal_reads.lock().unwrap().is_empty());
        assert!(counter.canonical_member_reads.lock().unwrap().len() > selected_before);
        let reads = counter.commit_reads.lock().unwrap();
        assert!(!reads[reads_before..].is_empty());
        assert!(reads[reads_before..].iter().all(|read| *read == position));
        assert!(
            reads.len() - reads_before < 3,
            "warm reads and sticky damage do not reload"
        );
        assert_eq!(counter.reads.load(Ordering::Relaxed), member_reads);
        assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
        assert_eq!(medium.offered().len(), offered);
    }
}
