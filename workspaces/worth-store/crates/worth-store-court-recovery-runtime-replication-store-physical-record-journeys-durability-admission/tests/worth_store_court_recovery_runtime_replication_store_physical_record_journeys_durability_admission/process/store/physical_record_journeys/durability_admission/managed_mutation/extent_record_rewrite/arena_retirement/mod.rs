#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement::*;
use {worth_store_testkit_store::durability_admission::independent_wal_oracle::copy_intent::produced_copy_intents, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind, worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction};
use {std::path::Path, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::ServingPhysicalRuntime, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement_crash, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::manifest_fixture::current_extent_route, crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::read_record, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request};
use {worth_store::physical_runtime::ArenaEvacuationThreshold, worth_store::physical_runtime::ExtentArenaCapacity, worth_store::physical_runtime::PhysicalArenaEvacuationPreparationOutcome, worth_store::physical_runtime::PhysicalExtentCopyPhase, worth_store::physical_runtime::PhysicalRecordInitialization, worth_store::physical_runtime::PhysicalRecordOpen, worth_store::physical_runtime::PhysicalRecordPlacementPolicy};
use {worth_store_physical_format::RecordArtifactFile};

#[test]
fn selected_copy_releases_its_entire_source_arena() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = configuration();
    let placement = worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .arena_capacity(
            worth_store::physical_runtime::ExtentArenaCapacity::bytes(64 << 20).unwrap(),
        )
        .arena_evacuation(
            worth_store::physical_runtime::ArenaEvacuationThreshold::percent(99).unwrap(),
        )
        .admit(format)
        .unwrap();
    let media = worth_store_testkit_store::physical_record_journeys::media(&root);
    let durability = crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation::fill_durability(&media);
    let serving = worth_store_testkit_store::physical_record_journeys::success(
        media.initialize_record_store(
            worth_store::physical_runtime::PhysicalRecordInitialization::new(
                format, placement, access, durability,
            ),
        ),
    );
    let arena = root.join("families/records/arenas").join(
        worth_store_physical_format::RecordArtifactFile::ExtentArena { arena: 1 }.file_name(),
    );
    let payload = vec![91; 40_000];
    let append = completed(prepare(&serving, placement, [21; 32], &payload).execute());
    let identity = append.persisted_records()[0];
    let record = append.into_acknowledgment().record_ids().next().unwrap();
    let source_route = current_extent_route(&root, identity);
    assert_eq!(source_route.arena, 1);
    let source_arena_bytes = std::fs::metadata(&arena).unwrap().len();
    let occupied_before = arena_disk_bytes(&root);
    assert_eq!(occupied_before, source_arena_bytes);
    let submission = serving.record_submission();
    let before = serving.physical_work_counters();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([22; 32]),
        )
        .unwrap();
    let Ok(mut progress) = submission
        .certification_begin_selected_extent_copy(placement, request(key), identity)
        .unwrap()
    else {
        panic!("the selected source must enter a bounded copy session");
    };
    let mut foreground_progress = false;
    for _ in 0..1000 {
        if progress.phase
            == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption
        {
            break;
        }
        if progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::Copying
            && !foreground_progress
        {
            completed(prepare(&serving, placement, [24; 32], &[17; 4096]).execute());
            foreground_progress = true;
        }
        progress = submission.advance_extent_copy().unwrap();
    }
    assert_eq!(
        progress.phase,
        worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption
    );
    assert!(
        foreground_progress,
        "ordinary foreground publication advances while copying"
    );
    let after = serving.physical_work_counters();
    let compaction = |counts: worth_store::physical_runtime::PhysicalWorkCounterSnapshot| {
        counts.count_under_pressure(
            worth_store::physical_runtime::PhysicalWorkOperationFamily::ArtifactPublication,
            worth_store::physical_runtime::PhysicalWorkPressureClass::BackgroundCompaction,
            worth_store::physical_runtime::PhysicalWorkCounterStage::Terminal,
        )
    };
    assert_eq!(
        compaction(after) - compaction(before),
        4,
        "three source chunks and one manifest settle through the compaction queue"
    );
    let prepared = submission.prepare_completed_extent_copy().unwrap();
    completed(prepared.execute());
    let occupied_at_copy_peak = arena_disk_bytes(&root);
    assert!(
        occupied_at_copy_peak <= occupied_before + (64 << 20),
        "one admitted filling arena bounds physical space while the source remains held"
    );
    let destination_route = current_extent_route(&root, identity);
    assert_eq!(destination_route.arena, 2);
    let reader = serving.records().unwrap();
    let session = reader
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(payload.len() as u32).unwrap(),
            ),
        )
        .unwrap();
    assert_eq!(
        worth_store_testkit_store::physical_record_journeys::read_record(session, payload.len()).0,
        payload
    );
    drop(reader);
    assert!(arena.exists());
    let checkpoint = crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation::copy_checkpoint(&serving, 1);
    assert!(matches!(
        submission.finalize_extent_copy(&checkpoint).unwrap(),
        worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress::AwaitingCheckpoint { .. }
    ));
    let checkpoint = crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation::copy_checkpoint(&serving, 2);
    assert_eq!(
        submission.finalize_extent_copy(&checkpoint).unwrap(),
        worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress::Resolved,
    );
    serving.retire_displaced_segment().unwrap();
    assert!(
        arena.exists(),
        "range release keeps the shared source arena"
    );
    drop(submission);
    serving.close();
    let copies = produced_copy_intents(&root);
    assert_eq!(
        copies.len(),
        1,
        "the source hold has one original copy intent"
    );
    assert_eq!(copies[0].source_root, 2);
    assert_eq!(
        copies[0].source,
        [source_route.arena, source_route.offset, source_route.length]
    );
    assert_eq!(
        copies[0].destination,
        [
            destination_route.arena,
            destination_route.offset,
            destination_route.length,
        ]
    );
    let releases = produced_retirement_payloads(&root)
        .into_iter()
        .filter(|record| {
            record.kind == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Extent
                && record.action == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent
        })
        .collect::<Vec<_>>();
    assert_eq!(
        releases.len(),
        1,
        "one exact source range release is durable"
    );
    assert_eq!(releases[0].release_roots, Some([4, 5]));
    assert_eq!(
        releases[0].arena_range,
        Some([source_route.arena, source_route.offset, source_route.length])
    );
    crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::arena_retirement_crash::assert_offline_arena_integrity(
        &root,
        parent.path(),
        0,
        "foreground-rebased-release",
    );
    let serving = reopen_with_copy_policy(&root);
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([23; 32]),
        )
        .unwrap();
    let outcome = submission
        .prepare_arena_evacuation(placement, request(key))
        .unwrap();
    assert!(matches!(
        outcome,
        worth_store::physical_runtime::PhysicalArenaEvacuationPreparationOutcome::EmptyAwaitingRetirement { arena: 1, .. }
    ));
    serving.retire_displaced_segment().unwrap();
    assert!(!arena.exists(), "whole-arena retirement removes the file");
    assert_eq!(
        arena_disk_bytes(&root),
        occupied_at_copy_peak - source_arena_bytes,
        "retirement reclaims the exact source-arena byte footprint"
    );
    serving.close();
    let reopened = reopen_with_copy_policy(&root);
    assert!(!arena.exists());
    let reader = reopened.records().unwrap();
    let session = reader
        .open(
            record,
            worth_store::physical_runtime::RecordReadLimits::new(
                worth_store::physical_runtime::RecordByteLimit::new(payload.len() as u32).unwrap(),
            ),
        )
        .unwrap();
    assert_eq!(
        worth_store_testkit_store::physical_record_journeys::read_record(session, payload.len()).0,
        payload
    );
    reopened.close();
}
