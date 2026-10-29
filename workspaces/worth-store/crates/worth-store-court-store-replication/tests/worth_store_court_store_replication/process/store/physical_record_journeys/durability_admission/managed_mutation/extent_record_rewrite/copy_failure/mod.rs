#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::copy_failure::*;
use {worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationOutcome, worth_store::physical_runtime::ServingPhysicalRuntime, worth_signal::facade::TemporalDuration, worth_proof::TransitionOutcome, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::extent_payload, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_record, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
use {worth_store::physical_runtime::PhysicalCheckpointDeadline, worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey, worth_store::physical_runtime::PhysicalCheckpointOutcome, worth_store::physical_runtime::PhysicalCheckpointRequest, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress, worth_store::physical_runtime::PhysicalMutationIndeterminateStage};

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
        .issue_idempotency_key(worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([231; 32]))
        .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request(key), persisted)
        .unwrap()
    else {
        panic!("actual selected extent must admit a bounded copy");
    };
    for _ in 0..128 {
        if progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(progress.phase, worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption);

    let prepared = submission.prepare_completed_extent_copy().unwrap();
    serving.certification_fail_next_wal_member_before_effect();
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            assert_eq!(fate.stage(), worth_store::physical_runtime::PhysicalMutationIndeterminateStage::WalAppend);
            assert_eq!(fate.completed_effect_count(), 0);
        }
        _ => panic!("injected unstarted final WAL must be indeterminate"),
    }
    assert_eq!(serving.certification_pending_publication_count(), 0);
    assert_eq!(read_record(&serving, record), payload);
    completed(prepare(&serving, placement, [232; 32], b"root-still-progresses").execute());

    assert!(matches!(
        submission.cancel_extent_copy().unwrap(),
        worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress::AwaitingCheckpoint { .. }
    ));
    let checkpoint = checkpoint_copy_cancellation(&serving);
    assert_eq!(
        submission.finalize_extent_copy(&checkpoint).unwrap(),
        worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress::Resolved
    );
    completed(prepare(&serving, placement, [233; 32], b"after-copy-resolution").execute());
    serving.close();
}
