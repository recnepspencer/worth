#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::copy_failure::*;
use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::extent_frame_fixture::extent_payload, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_record, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};

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
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([231; 32]),
        )
        .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request(key), persisted)
        .unwrap()
    else {
        panic!("actual selected extent must admit a bounded copy");
    };
    for _ in 0..128 {
        if progress.phase
            == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption
        {
            break;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(
        progress.phase,
        worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption
    );

    let prepared = submission.prepare_completed_extent_copy().unwrap();
    serving.certification_fail_next_wal_member_before_effect();
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            assert_eq!(
                fate.stage(),
                worth_store::physical_runtime::PhysicalMutationIndeterminateStage::WalAppend
            );
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
    let checkpoint = checkpoint_copy_cancellation(&serving, 234);
    assert_eq!(
        submission.finalize_extent_copy(&checkpoint).unwrap(),
        worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress::Resolved
    );
    completed(prepare(&serving, placement, [233; 32], b"after-copy-resolution").execute());
    serving.close();
}

/// S8b: a cancelled candidate extent in a published arena re-enters the
/// allocator through `WritebackFenceDrain` once a checkpoint covers its
/// cancellation (with zero root publication), survives close + reopen without
/// generation rewind, and is reused at the exact cancelled `(arena, offset,
/// length)` with different bytes and zero stale residue.
#[test]
fn cancelled_candidate_extent_reuses_at_newer_generation_without_root_publication() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let payload_a = vec![0x5a; extent_payload().len()];
    let payload_b = vec![0xaa; extent_payload().len()];

    // Session 1: publish arena 2 by copying `a` from arena 1 -> arena 2, then
    // retire `a`'s displaced slot in arena 1 so both arena 1 and arena 2 are
    // published (< next_arena = 3) and arena 1 is free from offset 0.
    let (a_persisted, a_record, expected_len) = {
        let serving = serving_from_initialization(&root);
        let a = completed(prepare(&serving, placement, [240; 32], &payload_a).execute());
        let a_persisted = a.persisted_records()[0];
        let a_record = a.into_acknowledgment().record_ids().next().unwrap();
        let orig_route = current_extent_route(&root, a_persisted);
        assert_eq!(orig_route.arena, 1);
        assert_eq!(orig_route.offset, 0);
        copy_and_publish_extent(&serving, a_persisted, 241);
        while serving.retire_displaced_segment().is_ok() {}
        let moved_route = current_extent_route(&root, a_persisted);
        assert_eq!(moved_route.arena, 2);
        serving.close();
        (a_persisted, a_record, orig_route.length)
    };

    // Session 2: append `b` (0xaa) — best-fit places it in arena 2 (tighter
    // free tail than empty arena 1). Copy `b` into published arena 1 at offset
    // 0, then durably cancel + checkpoint + finalize with zero root pub.
    let arena1 = worth_store_physical_format::ExtentArenaId::new(1).unwrap();
    let cancelled_range =
        worth_store_physical_format::ExtentArenaRange::new(arena1, 0, expected_len).unwrap();
    let (b_record, reissued) = {
        let serving = serving_from_open(&root);
        let b = completed(prepare(&serving, placement, [244; 32], &payload_b).execute());
        let b_persisted = b.persisted_records()[0];
        let b_record = b.into_acknowledgment().record_ids().next().unwrap();
        let b_route = current_extent_route(&root, b_persisted);
        assert_eq!(b_route.arena, 2, "best-fit places b in tighter arena 2");
        let reissued = cancel_copy_without_root_publication(&serving, b_persisted, 245);
        assert!(reissued > worth_store_physical_format::ExtentAllocationGeneration::OPENED);
        assert_eq!(
            serving
                .certification_physical_residency()
                .active_arena_generation(cancelled_range),
            reissued,
            "Path B drain must advance generation on exact cancelled range (arena 1, offset 0)"
        );
        serving.close();
        (b_record, reissued)
    };

    // Session 3 (post-reopen): floor must not rewind to OPENED, and copying `a`
    // (0x5a) out of arena 2 must reuse the exact cancelled `(arena 1, 0, len)`.
    let reopened = serving_from_open(&root);
    let reopened_gen = reopened
        .certification_physical_residency()
        .active_arena_generation(cancelled_range);
    assert!(
        reopened_gen >= reissued,
        "reopen must seed durable generation floor >= pre-close barrier ({reopened_gen:?} vs {reissued:?})"
    );
    copy_and_publish_extent(&reopened, a_persisted, 250);
    let reused_route = current_extent_route(&root, a_persisted);
    assert_eq!(
        (reused_route.arena, reused_route.offset, reused_route.length),
        (1, 0, expected_len),
        "reopened allocation must reuse the exact cancelled arena range"
    );
    assert_eq!(read_record(&reopened, a_record), payload_a);
    assert_eq!(read_record(&reopened, b_record), payload_b);
    reopened.close();
}
