#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::evacuation::*;
use worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::produced_retirement_payloads;
use {worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::completed, worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::manifest_fixture::current_extent_route, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::prepare, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::prepare_extent_rewrite, worth_store_testkit_store::physical_record_journeys::durability_admission::managed_mutation::extent_record_rewrite::request};
use {worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES, worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES};

/// Uses the public minimum arena geometry and ordinary append/rewrite/release.
/// The fill deliberately costs real I/O rather than inventing free-map state.
#[test]
#[ignore = "64 MiB production arena fill; run explicitly for C11 evacuation gate"]
fn sparse_arena_producer_moves_live_extent_under_compaction_queue_authority() {
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
    let durability = fill_durability(&media);
    let serving = worth_store_testkit_store::physical_record_journeys::success(
        media.initialize_record_store(
            worth_store::physical_runtime::PhysicalRecordInitialization::new(
                format, placement, access, durability,
            ),
        ),
    );
    // Rewrite only the first three ceiling-sized records to make arena 1 sparse.
    // Every remaining live source is beyond that ceiling and requires SourceCopy.
    let rewrite_payload = vec![77; 256 * 1024];
    let payload = vec![77; LIVE_PAYLOAD_BYTES];
    let page_bytes = format.declaration().page_size().bytes() as u64;
    let chunk_payload_bytes =
        page_bytes - (DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES) as u64;
    let expected_chunks = (payload.len() as u64).div_ceil(chunk_payload_bytes);
    let mut records = Vec::new();
    for ordinal in 1..=256_u64 {
        let bytes = if ordinal <= 3 {
            &rewrite_payload
        } else {
            &payload
        };
        let result = completed(prepare(&serving, placement, material(ordinal), bytes).execute());
        let identity = result.persisted_records()[0];
        let record = result.into_acknowledgment().record_ids().next().unwrap();
        records.push((identity, record));
        if current_extent_route(&root, identity).arena != 1 {
            break;
        }
    }
    assert_eq!(current_extent_route(&root, records[0].0).arena, 1);
    assert_eq!(
        current_extent_route(&root, records.last().unwrap().0).arena,
        2,
        "ordinary allocation fills one admitted arena before opening another"
    );
    let mut rewrite_count = 0_u64;
    for (identity, record) in records.iter().take(3) {
        for _ in 0..16 {
            if current_extent_route(&root, *identity).arena != 1 {
                break;
            }
            completed(
                prepare_extent_rewrite(&serving, placement, material(300 + rewrite_count), *record)
                    .execute(),
            );
            rewrite_count += 1;
        }
        assert_ne!(
            current_extent_route(&root, *identity).arena,
            1,
            "bounded ordinary rewrites must move every ceiling-sized source outside arena 1",
        );
    }
    for _ in 0..rewrite_count {
        serving.retire_displaced_segment().unwrap();
    }
    let sources = records
        .iter()
        .map(|(identity, record)| (*identity, *record, current_extent_route(&root, *identity)))
        .collect::<Vec<_>>();
    let before = serving.physical_work_counters();
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material(400)),
        )
        .unwrap();
    let mut outcome = submission
        .prepare_arena_evacuation(placement, request(key.clone()))
        .unwrap();
    for _ in 0..16 {
        if !matches!(
            outcome,
            worth_store::physical_runtime::PhysicalArenaEvacuationPreparationOutcome::ScanPending
        ) {
            break;
        }
        outcome = submission
            .prepare_arena_evacuation(placement, request(key.clone()))
            .unwrap();
    }
    let worth_store::physical_runtime::PhysicalArenaEvacuationPreparationOutcome::Copying(
        mut progress,
    ) = outcome
    else {
        panic!("sparse arena must produce a retained bounded copy session");
    };
    let mut foreground_progress = false;
    let mut copied_before_verification = 0;
    for _ in 0..10_000 {
        assert_eq!(progress.logical_bytes, payload.len() as u64);
        assert!(progress.completed_payload_bytes <= progress.logical_bytes);
        if progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::Copying {
            copied_before_verification =
                copied_before_verification.max(progress.completed_payload_bytes);
        }
        if progress.phase
            == worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption
        {
            break;
        }
        if progress.phase == worth_store::physical_runtime::PhysicalExtentCopyPhase::Copying
            && !foreground_progress
        {
            completed(prepare(&serving, placement, material(401), &[33; 4096]).execute());
            foreground_progress = true;
        }
        match submission.advance_extent_copy() {
            Ok(next) => {
                let reset = matches!(
                    (progress.phase, next.phase),
                    (
                        worth_store::physical_runtime::PhysicalExtentCopyPhase::DurableIntent,
                        worth_store::physical_runtime::PhysicalExtentCopyPhase::Copying
                    ) | (
                        worth_store::physical_runtime::PhysicalExtentCopyPhase::Synchronizing,
                        worth_store::physical_runtime::PhysicalExtentCopyPhase::Verifying
                    )
                );
                if reset {
                    assert_eq!(progress.completed_payload_bytes, payload.len() as u64);
                    assert_eq!(next.completed_payload_bytes, 0);
                } else {
                    assert!(
                        next.completed_payload_bytes >= progress.completed_payload_bytes,
                        "copy progress may reset only at its two phase boundaries",
                    );
                    assert!(
                        next.completed_payload_bytes - progress.completed_payload_bytes
                            <= chunk_payload_bytes,
                        "one executed copy advance may complete at most one payload frame",
                    );
                }
                progress = next;
            }
            Err(worth_store::physical_runtime::RecordAppendError::Denied(
                worth_store::physical_runtime::RecordAppendDenial::PhysicalPressure,
            )) => {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            Err(error) => panic!("bounded copy failed at {progress:?}: {error:?}"),
        }
    }
    assert!(
        foreground_progress,
        "ordinary foreground publication advances during copying"
    );
    assert_eq!(
        progress.phase,
        worth_store::physical_runtime::PhysicalExtentCopyPhase::ReadyForAdoption
    );
    assert_eq!(copied_before_verification, payload.len() as u64);
    let prepared = submission
        .prepare_completed_extent_copy()
        .expect("adopt completed copy using current root");
    let result = completed(prepared.execute());
    assert_eq!(result.persisted_records().len(), 1);
    let observation = result.observation();
    assert_eq!(observation.transfer_count(), expected_chunks);
    assert_eq!(observation.explicit_copy_count(), expected_chunks);
    assert_eq!(observation.copied_bytes(), payload.len() as u64);
    assert!(observation.peak_transfer_width() <= page_bytes);
    assert!(observation.peak_scratch_bytes() <= page_bytes);
    let selected = result.persisted_records()[0];
    let (_, record, source) = sources
        .iter()
        .find(|(identity, _, _)| *identity == selected)
        .unwrap();
    let destination = current_extent_route(&root, selected);
    assert_ne!(
        source.arena, destination.arena,
        "evacuation excludes its source arena"
    );
    assert_eq!(
        result
            .into_acknowledgment()
            .record_ids()
            .collect::<Vec<_>>(),
        vec![*record],
        "SourceCopy publication preserves the selected record id",
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
        expected_chunks + 1,
        "bounded chunk frames and manifest execute with compaction lease authority"
    );
    let reader = serving.records().unwrap();
    let session = reader
        .open(
            *record,
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
    let checkpoint = copy_checkpoint(&serving, 1);
    assert!(matches!(
        submission.finalize_extent_copy(&checkpoint).unwrap(),
        worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress::AwaitingCheckpoint { .. }
    ));
    let checkpoint = copy_checkpoint(&serving, 2);
    assert_eq!(
        submission.finalize_extent_copy(&checkpoint).unwrap(),
        worth_store::physical_runtime::PhysicalExtentCopyResolutionProgress::Resolved
    );
    serving.retire_displaced_segment().unwrap();
    serving.close();
    let retirements = produced_retirement_payloads(&root);
    for action in [
        worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Intent,
        worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetirementAction::Completion,
    ] {
        assert!(
            retirements.iter().any(|retirement| {
                retirement.kind == worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::retirement_redo::IndependentRetiredKind::Extent
                    && retirement.action == action
                    && retirement.arena_range == Some([source.arena, source.offset, source.length])
            }),
            "evacuated source range requires exact durable {action:?}",
        );
    }
    let mut weak_identity = record.allocation_epoch().to_vec();
    weak_identity.extend_from_slice(&record.ordinal().to_le_bytes());
    std::fs::write(parent.path().join("evacuated-record.id"), weak_identity).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            READER_CHILD,
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(REOPEN_DIRECTORY, parent.path())
        .output()
        .unwrap();
    assert!(
        output.status.success()
            && String::from_utf8_lossy(&output.stdout).contains("running 1 test"),
        "fresh-process evacuation reader failed: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    crate::durability_admission::managed_mutation::extent_record_rewrite::evacuation::observation::assert_evacuated_arena_offline(&root, parent.path());
}
#[path = "evacuation_reader/mod.rs"]
pub(crate) mod evacuation_reader;
