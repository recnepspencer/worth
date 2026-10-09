use super::*;
use worth_store::physical_runtime::{
    PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey, PhysicalCheckpointOutcome,
    PhysicalCheckpointRequest, PhysicalExtentCopyPhase, PhysicalExtentCopyResolutionProgress,
    PhysicalMutationIndeterminateStage,
};

/// Direct selection keeps this lifecycle proof cheap. The independent 64 MiB
/// producer test checks sparse-arena eligibility; all effects here use the real
/// source pin, arena reservation, copy, WAL, root, and checkpoint owners.
#[test]
fn unstarted_final_copy_wal_does_not_fence_root_or_prevent_durable_cancellation() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let payload = extent_payload();
    let appended = completed(prepare(&serving, placement, [230; 32], &payload).execute());
    let persisted = appended.persisted_records()[0];
    let record = appended.into_acknowledgment().record_ids().next().unwrap();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([231; 32]))
        .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request(key), persisted)
        .unwrap()
    else {
        panic!("actual selected extent must admit a bounded copy");
    };
    for _ in 0..128 {
        if progress.phase == PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(progress.phase, PhysicalExtentCopyPhase::ReadyForAdoption);

    let prepared = submission.prepare_completed_extent_copy().unwrap();
    serving.certification_fail_next_wal_member_before_effect();
    match prepared.execute() {
        PhysicalMutationOutcome::Indeterminate(fate) => {
            assert_eq!(fate.stage(), PhysicalMutationIndeterminateStage::WalAppend);
            assert_eq!(fate.completed_effect_count(), 0);
        }
        _ => panic!("injected unstarted final WAL must be indeterminate"),
    }
    assert_eq!(serving.certification_pending_publication_count(), 0);
    assert_eq!(read_record(&serving, record), payload);
    completed(prepare(&serving, placement, [232; 32], b"root-still-progresses").execute());

    assert!(matches!(
        submission.cancel_extent_copy().unwrap(),
        PhysicalExtentCopyResolutionProgress::AwaitingCheckpoint { .. }
    ));
    let checkpoint = checkpoint_copy_cancellation(&serving);
    assert_eq!(
        submission.finalize_extent_copy(&checkpoint).unwrap(),
        PhysicalExtentCopyResolutionProgress::Resolved
    );
    completed(prepare(&serving, placement, [233; 32], b"after-copy-resolution").execute());
    serving.close();
}

fn checkpoint_copy_cancellation(
    serving: &ServingPhysicalRuntime,
) -> worth_store::physical_runtime::CompletedPhysicalCheckpoint {
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([234; 32]),
        PhysicalCheckpointDeadline::at(TemporalDuration::temporal_duration(30_000).unwrap()),
    );
    let TransitionOutcome::Success(handle) = serving.checkpoints().start(request).into_raw() else {
        panic!("copy cancellation checkpoint must admit");
    };
    let PhysicalCheckpointOutcome::Completed(checkpoint) = handle.wait() else {
        panic!("copy cancellation checkpoint must complete");
    };
    checkpoint
}
