use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    ManagedPhysicalIntegrityScrubProgress, ManagedPhysicalIntegrityScrubRequest,
    PhysicalIntegrityScrubTarget, PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationOutcome, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalRetirementDenial, RecordByteLimit, RecordReadDenial, RecordReadLimits,
};
use worth_store_physical_backend::MediaOperationRole;
use worth_store_physical_format::{
    PhysicalArtifactReadTarget, PhysicalRecordFormatDeclaration, RecordArtifactFile,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

use worth_store::physical_runtime::PhysicalWorkCapacity;

use super::{append, checkpoint, limits, placement, segment_files};

#[test]
fn cold_extent_survives_four_publications_scrub_and_blocked_retirement() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (profile, request, _) = super::super::physical_work::work_fixture();
    let capacity = PhysicalWorkCapacity::new(8, 256, 32_768, 1024 * 1024, 64 * 1024 * 1024)
        .unwrap()
        .with_dispatch_permits(4)
        .unwrap();
    let (serving, gate, activation) =
        super::sync::open_with_file_sync_pause(&root, profile.with_capacity(capacity));
    run_cold_extent(&serving, gate, activation, &root, request, 0);
    serving.close();
}

pub(super) fn run_cold_extent(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    gate: worth_store_physical_backend::MediaPauseGate,
    activation: worth_store_physical_backend::CertificationMediaFaultActivation,
    root: &std::path::Path,
    request: worth_store::physical_runtime::PhysicalReadWorkRequest,
    ordinal_base: u64,
) {
    let policy = placement();
    let inline_id = append(&serving, policy, ordinal_base + 1, b"held-inline");
    let extent_bytes = vec![0x5A; 20_000];
    let extent_id = append(&serving, policy, ordinal_base + 2, &extent_bytes);
    let old = serving.records().unwrap();
    let selected = old.protected_root();
    let second = serving.records().unwrap();
    let mut inline = second.open(inline_id, limits()).unwrap();
    let borrowed = inline.next_chunk().unwrap().unwrap();
    assert_eq!(borrowed.bytes(), b"held-inline");

    let checkpoint_finished = AtomicBool::new(false);
    let foreground_finished = AtomicBool::new(false);
    let floor_finished = AtomicBool::new(false);
    thread::scope(|scope| {
        let mut cold = old
            .open(
                extent_id,
                RecordReadLimits::new(RecordByteLimit::new(20_000).unwrap()),
            )
            .unwrap();
        let pause = cold.certification_pause_next_read_call();
        let expected = extent_bytes.clone();
        let reader = scope.spawn(move || {
            let mut payload = Vec::new();
            loop {
                match cold.next_chunk().unwrap() {
                    Some(chunk) => payload.extend_from_slice(chunk.bytes()),
                    None => break,
                }
            }
            payload
        });
        assert!(pause.await_arrival());
        match rewrite(&serving, policy, ordinal_base + 20).execute() {
            PhysicalMutationOutcome::Completed(_) => {}
            PhysicalMutationOutcome::ProvenNoEffect(fate) => {
                panic!("rewrite had no effect: {:?}", fate.cause())
            }
            PhysicalMutationOutcome::Indeterminate(fate) => {
                panic!("rewrite became indeterminate at {:?}", fate.stage())
            }
        }
        let mut newest = extent_id;
        for step in 0..4 {
            let mut record = vec![0x44; 20_000];
            record[0] = step as u8;
            newest = append(&serving, policy, ordinal_base + 10 + step, &record);
        }
        let current = serving.records().unwrap();
        assert_ne!(current.protected_root(), selected);
        let mut seen = Vec::new();
        let mut session = current
            .open(
                newest,
                RecordReadLimits::new(RecordByteLimit::new(20_000).unwrap()),
            )
            .unwrap();
        loop {
            match session.next_chunk().unwrap() {
                Some(chunk) => seen.extend_from_slice(chunk.bytes()),
                None => break,
            }
        }
        assert_eq!(seen[0], 3);
        assert_eq!(seen.len(), 20_000);
        assert!(matches!(
            old.open(
                newest,
                RecordReadLimits::new(RecordByteLimit::new(20_000).unwrap()),
            ),
            Err(error) if error.denial() == RecordReadDenial::RecordNotFound
        ));
        let syncs_before_checkpoint = serving
            .media_counters()
            .attempts_for(MediaOperationRole::SynchronizeFileState);
        activation.arm().unwrap();
        let checkpoint_thread = scope.spawn(|| {
            checkpoint(&serving, ordinal_base + 30);
            checkpoint_finished.store(true, Ordering::Release);
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        while gate.reached_context().is_none() && Instant::now() < deadline {
            thread::yield_now();
        }
        // Only the checkpoint runs after arming; the cold reader is parked
        // before its read call, so the held sync was issued by the checkpoint.
        let reached = gate
            .reached_context()
            .expect("checkpoint must reach a real file-state sync");
        assert_eq!(reached.role(), MediaOperationRole::SynchronizeFileState);
        assert!(reached.role_ordinal() > syncs_before_checkpoint);
        assert_eq!(
            reached.role_ordinal(),
            serving
                .media_counters()
                .attempts_for(MediaOperationRole::SynchronizeFileState)
        );
        assert!(
            !checkpoint_finished.load(Ordering::Acquire),
            "checkpoint returned while its file sync was paused"
        );

        // Fill every ready slot and both background dispatch permits while
        // the checkpoint sync is still held, then prove the protected floor.
        // The held sync occupies one command slot and one background permit.
        let paused_slots = super::scheduler::hold_ready_slots(&serving, request.clone(), 1);
        let mut background = Vec::new();
        while let Ok(permit) =
            serving.reserve_physical_scheduler_background(super::scheduler::dispatch_lane())
        {
            background.push(permit);
            assert!(background.len() <= 4);
        }
        assert_eq!(
            background.len(),
            1,
            "the paused checkpoint and one more dispatch fill the background share"
        );
        let floor = scope.spawn(|| {
            let mut held = Vec::new();
            for _ in 0..2 {
                held.push(
                    serving
                        .reserve_physical_scheduler_foreground(super::scheduler::dispatch_lane())
                        .expect("the foreground floor stays open beside the paused sync"),
                );
            }
            floor_finished.store(true, Ordering::Release);
            drop(held);
        });
        let floor_deadline = Instant::now() + Duration::from_secs(2);
        while !floor_finished.load(Ordering::Acquire) && Instant::now() < floor_deadline {
            thread::yield_now();
        }
        assert!(
            floor_finished.load(Ordering::Acquire),
            "the foreground floor did not admit while the checkpoint sync was paused"
        );
        floor.join().unwrap();
        // Foreground reads share the ready queue, so free it; the background
        // share and the paused sync stay held.
        drop(paused_slots);

        let foreground = scope.spawn(|| {
            let mut session = serving
                .records()
                .unwrap()
                .open(inline_id, limits())
                .unwrap();
            assert_eq!(
                session.next_chunk().unwrap().unwrap().bytes(),
                b"held-inline"
            );
            append(&serving, policy, ordinal_base + 40, b"during-sync");
            foreground_finished.store(true, Ordering::Release);
        });
        // The gate stays closed until this wait ends, so completion here
        // proves the foreground did not queue behind the paused sync. The
        // bound only turns a hang into a failure.
        let hang_guard = Instant::now() + Duration::from_secs(120);
        while !foreground_finished.load(Ordering::Acquire) && Instant::now() < hang_guard {
            thread::yield_now();
        }
        let foreground_during_pause = foreground_finished.load(Ordering::Acquire);
        let still_paused = !checkpoint_finished.load(Ordering::Acquire);
        let attempts = serving
            .media_counters()
            .attempts_for(MediaOperationRole::SynchronizeFileState);
        let completed = serving
            .media_counters()
            .completed_operations_for(MediaOperationRole::SynchronizeFileState);
        gate.release();
        foreground.join().unwrap();
        checkpoint_thread.join().unwrap();
        drop(background);
        // The finished checkpoint returned its command slot: all eight admit.
        let mut held_slots = super::scheduler::hold_ready_slots(&serving, request, 0);
        assert!(
            foreground_during_pause,
            "foreground read and append did not finish while the checkpoint sync was paused"
        );
        assert!(
            still_paused,
            "checkpoint returned while its file sync was paused"
        );
        assert_eq!(
            completed + 1,
            attempts,
            "the paused file-state sync is still in flight"
        );
        held_slots.release_one();
        inspect_selector(&serving, &root);
        let deletions = serving.media_counters().deletions();
        assert_eq!(
            serving.retire_displaced_segment(),
            Err(PhysicalRetirementDenial::Protected)
        );
        assert_eq!(serving.media_counters().deletions(), deletions);
        drop(held_slots);
        let reads_before_resume = serving
            .media_counters()
            .completed_operations_for(MediaOperationRole::PositionedRead);
        pause.release();
        let payload = reader.join().unwrap();
        assert_eq!(payload, expected);
        assert!(
            serving
                .media_counters()
                .completed_operations_for(MediaOperationRole::PositionedRead)
                > reads_before_resume
        );
        drop(current);
    });

    let deletions = serving.media_counters().deletions();
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Protected)
    );
    assert_eq!(serving.media_counters().deletions(), deletions);
    drop(borrowed);
    drop(inline);
    drop(second);
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(PhysicalRetirementDenial::Protected)
    );
    drop(old);
    let occupied = segment_files(&root);
    serving
        .retire_displaced_segment()
        .expect("the last protector admits retirement");
    assert!(occupied.difference(&segment_files(&root)).next().is_some());
}

pub(super) fn rewrite(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
    let mut material = [0x6B; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    match submission
        .rewrite_selected_inline_segment(
            policy,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Success(_) => panic!("rewrite {ordinal} did not stay prepared"),
        TransitionOutcome::Denied(denial) => panic!("rewrite {ordinal} denied: {denial:?}"),
        TransitionOutcome::Deferred(deferred) => panic!("rewrite {ordinal} deferred: {deferred:?}"),
        TransitionOutcome::Stale(stale) => panic!("rewrite {ordinal} stale: {stale:?}"),
        TransitionOutcome::RebindRequired(rebind) => panic!("rewrite {ordinal} rebind: {rebind:?}"),
        TransitionOutcome::Failed(failure) => panic!("rewrite {ordinal} failed: {failure:?}"),
    }
}

fn inspect_selector(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    root: &std::path::Path,
) {
    let length = std::fs::metadata(root.join("families/records/root-current.selector"))
        .unwrap()
        .len();
    let target = PhysicalIntegrityScrubTarget::new(
        PhysicalArtifactReadTarget::Record(RecordArtifactFile::CurrentRootSelector),
        PhysicalArtifactScope::current_root_selector(
            serving.store_identity(),
            PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
            PhysicalByteRange::new(0, length).unwrap(),
        ),
    )
    .unwrap();
    let request = ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        65_536,
        65_536,
        Duration::from_secs(30),
    )
    .unwrap();
    let mut handle = serving.start_physical_integrity_scrub(request).unwrap();
    match handle.next_window() {
        ManagedPhysicalIntegrityScrubProgress::WindowInspected(observation) => {
            assert!(observation.counters.acquired_bytes > 0);
        }
        other => panic!("scrub window did not inspect the selector: {other:?}"),
    }
}
