pub use std::thread;
pub use {std::sync::atomic::AtomicBool, std::sync::atomic::Ordering};
pub use {std::time::Duration, std::time::Instant};

pub use worth_proof::TransitionOutcome;
pub use worth_store_physical_backend::MediaOperationRole;
pub use {
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress,
    worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest,
    worth_store::physical_runtime::PhysicalIntegrityScrubTarget,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalRetirementDenial,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadDenial,
    worth_store::physical_runtime::RecordReadLimits,
};
pub use {
    worth_store_physical_format::PhysicalArtifactReadTarget,
    worth_store_physical_format::PhysicalRecordFormatDeclaration,
    worth_store_physical_format::RecordArtifactFile,
};
pub use {
    worth_store_physical_integrity::PhysicalArtifactScope,
    worth_store_physical_integrity::PhysicalByteRange,
};

pub use worth_store::physical_runtime::PhysicalWorkCapacity;

pub use {
    crate::physical_record_journeys::maintenance_interference::append,
    crate::physical_record_journeys::maintenance_interference::checkpoint,
    crate::physical_record_journeys::maintenance_interference::limits,
    crate::physical_record_journeys::maintenance_interference::placement,
    crate::physical_record_journeys::maintenance_interference::segment_files,
};

pub fn run_cold_extent(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    gate: worth_store_physical_backend::MediaPauseGate,
    activation: worth_store_physical_backend::CertificationMediaFaultActivation,
    root: &std::path::Path,
    request: worth_store::physical_runtime::PhysicalReadWorkRequest,
    ordinal_base: u64,
) {
    let policy = placement();
    let inline_id = append(serving, policy, ordinal_base + 1, b"held-inline");
    let extent_bytes = vec![0x5A; 20_000];
    let extent_id = append(serving, policy, ordinal_base + 2, &extent_bytes);
    let old = serving.records().unwrap();
    let selected = old.protected_root();
    let second = serving.records().unwrap();
    let mut inline = second.open(inline_id, limits()).unwrap();
    let borrowed = inline.next_chunk().unwrap().unwrap();
    assert_eq!(borrowed.bytes(), b"held-inline");

    let checkpoint_finished = std::sync::atomic::AtomicBool::new(false);
    let foreground_finished = std::sync::atomic::AtomicBool::new(false);
    let floor_finished = std::sync::atomic::AtomicBool::new(false);
    std::thread::scope(|scope| {
        // Release before the scope joins workers if any observation panics.
        let _release_sync = crate::physical_record_journeys::maintenance_interference::sync::FileSyncPauseRelease::new(&gate);
        let mut cold = old
            .open(
                extent_id,
                worth_store::physical_runtime::RecordReadLimits::new(
                    worth_store::physical_runtime::RecordByteLimit::new(20_000).unwrap(),
                ),
            )
            .unwrap();
        let pause = cold.certification_pause_next_read_call();
        let expected = extent_bytes.clone();
        let reader = scope.spawn(move || {
            let mut payload = Vec::new();
            while let Some(chunk) = cold.next_chunk().unwrap() {
                payload.extend_from_slice(chunk.bytes());
            }
            payload
        });
        assert!(pause.await_arrival());
        match rewrite(serving, policy, ordinal_base + 20).execute() {
            worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {}
            worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
                panic!("rewrite had no effect: {:?}", fate.cause())
            }
            worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
                panic!("rewrite became indeterminate at {:?}", fate.stage())
            }
        }
        let mut newest = extent_id;
        for step in 0..4 {
            let mut record = vec![0x44; 20_000];
            record[0] = step as u8;
            newest = append(serving, policy, ordinal_base + 10 + step, &record);
        }
        let current = serving.records().unwrap();
        assert_ne!(current.protected_root(), selected);
        let mut seen = Vec::new();
        let mut session = current
            .open(
                newest,
                worth_store::physical_runtime::RecordReadLimits::new(
                    worth_store::physical_runtime::RecordByteLimit::new(20_000).unwrap(),
                ),
            )
            .unwrap();
        while let Some(chunk) = session.next_chunk().unwrap() {
            seen.extend_from_slice(chunk.bytes());
        }
        assert_eq!(seen[0], 3);
        assert_eq!(seen.len(), 20_000);
        assert!(matches!(
            old.open(
                newest,
                worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(20_000).unwrap()),
            ),
            Err(error) if error.denial() == worth_store::physical_runtime::RecordReadDenial::RecordNotFound
        ));
        let syncs_before_checkpoint = serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::SynchronizeFileState);
        activation.arm().unwrap();
        let checkpoint_thread = scope.spawn(|| {
            checkpoint(serving, ordinal_base + 30);
            checkpoint_finished.store(true, std::sync::atomic::Ordering::Release);
        });
        // The scaled reopened world may capture many frames before its first
        // sync. Match the checkpoint's own deadline; the gate proves ordering.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        while gate.reached_context().is_none() && std::time::Instant::now() < deadline {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        // Only the checkpoint runs after arming; the cold reader is parked
        // before its read call, so the held sync was issued by the checkpoint.
        let reached = gate
            .reached_context()
            .expect("checkpoint must reach a real file-state sync");
        assert_eq!(
            reached.role(),
            worth_store_physical_backend::MediaOperationRole::SynchronizeFileState
        );
        assert!(reached.role_ordinal() > syncs_before_checkpoint);
        assert_eq!(
            reached.role_ordinal(),
            serving.media_counters().attempts_for(
                worth_store_physical_backend::MediaOperationRole::SynchronizeFileState
            )
        );
        assert!(
            !checkpoint_finished.load(std::sync::atomic::Ordering::Acquire),
            "checkpoint returned while its file sync was paused"
        );

        // Fill every ready slot and both background dispatch permits while
        // the checkpoint sync is still held, then prove the protected floor.
        // The held sync occupies one command slot and one background permit.
        let paused_slots =
            crate::physical_record_journeys::maintenance_interference::scheduler::hold_ready_slots(
                serving,
                request.clone(),
                1,
            );
        let mut background = Vec::new();
        while let Ok(permit) = serving.reserve_physical_scheduler_background(
            crate::physical_record_journeys::maintenance_interference::scheduler::dispatch_lane(),
        ) {
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
                        .reserve_physical_scheduler_foreground(crate::physical_record_journeys::maintenance_interference::scheduler::dispatch_lane())
                        .expect("the foreground floor stays open beside the paused sync"),
                );
            }
            floor_finished.store(true, std::sync::atomic::Ordering::Release);
            drop(held);
        });
        let floor_deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while !floor_finished.load(std::sync::atomic::Ordering::Acquire)
            && std::time::Instant::now() < floor_deadline
        {
            std::thread::yield_now();
        }
        assert!(
            floor_finished.load(std::sync::atomic::Ordering::Acquire),
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
            append(serving, policy, ordinal_base + 40, b"during-sync");
            foreground_finished.store(true, std::sync::atomic::Ordering::Release);
        });
        // The gate stays closed until this wait ends, so completion here
        // proves the foreground did not queue behind the paused sync. The
        // bound only turns a hang into a failure.
        let hang_guard = std::time::Instant::now() + std::time::Duration::from_secs(120);
        while !foreground_finished.load(std::sync::atomic::Ordering::Acquire)
            && std::time::Instant::now() < hang_guard
        {
            std::thread::yield_now();
        }
        let foreground_during_pause =
            foreground_finished.load(std::sync::atomic::Ordering::Acquire);
        let still_paused = !checkpoint_finished.load(std::sync::atomic::Ordering::Acquire);
        let attempts = serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::SynchronizeFileState);
        let completed = serving.media_counters().completed_operations_for(
            worth_store_physical_backend::MediaOperationRole::SynchronizeFileState,
        );
        gate.release();
        foreground.join().unwrap();
        checkpoint_thread.join().unwrap();
        drop(background);
        // The finished checkpoint returned its command slot: all eight admit.
        let mut held_slots =
            crate::physical_record_journeys::maintenance_interference::scheduler::hold_ready_slots(
                serving, request, 0,
            );
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
        inspect_selector(serving, root);
        let deletions = serving.media_counters().deletions();
        assert_eq!(
            serving.retire_displaced_segment(),
            Err(worth_store::physical_runtime::PhysicalRetirementDenial::Protected)
        );
        assert_eq!(serving.media_counters().deletions(), deletions);
        drop(held_slots);
        let reads_before_resume = serving.media_counters().completed_operations_for(
            worth_store_physical_backend::MediaOperationRole::PositionedRead,
        );
        pause.release();
        let payload = reader.join().unwrap();
        assert_eq!(payload, expected);
        assert!(
            serving.media_counters().completed_operations_for(
                worth_store_physical_backend::MediaOperationRole::PositionedRead
            ) > reads_before_resume
        );
        drop(current);
    });

    let deletions = serving.media_counters().deletions();
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Protected)
    );
    assert_eq!(serving.media_counters().deletions(), deletions);
    drop(inline);
    drop(second);
    assert_eq!(
        serving.retire_displaced_segment(),
        Err(worth_store::physical_runtime::PhysicalRetirementDenial::Protected)
    );
    drop(old);
    let occupied = segment_files(root);
    serving
        .retire_displaced_segment()
        .expect("the last protector admits retirement");
    assert!(occupied.difference(&segment_files(root)).next().is_some());
}

pub fn rewrite(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
) -> worth_store::physical_runtime::PreparedPhysicalMutation {
    let mut material = [0x6B; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    match submission
        .rewrite_selected_inline_segment(
            policy,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        worth_proof::TransitionOutcome::Success(_) => {
            panic!("rewrite {ordinal} did not stay prepared")
        }
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("rewrite {ordinal} denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(deferred) => {
            panic!("rewrite {ordinal} deferred: {deferred:?}")
        }
        worth_proof::TransitionOutcome::Stale(stale) => {
            panic!("rewrite {ordinal} stale: {stale:?}")
        }
        worth_proof::TransitionOutcome::RebindRequired(rebind) => {
            panic!("rewrite {ordinal} rebind: {rebind:?}")
        }
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!("rewrite {ordinal} failed: {failure:?}")
        }
    }
}

pub fn inspect_selector(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    root: &std::path::Path,
) {
    let length = std::fs::metadata(root.join("families/records/root-current.selector"))
        .unwrap()
        .len();
    let target = worth_store::physical_runtime::PhysicalIntegrityScrubTarget::new(
        worth_store_physical_format::PhysicalArtifactReadTarget::Record(
            worth_store_physical_format::RecordArtifactFile::CurrentRootSelector,
        ),
        worth_store_physical_integrity::PhysicalArtifactScope::current_root_selector(
            serving.store_identity(),
            worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
                .admit()
                .unwrap(),
            worth_store_physical_integrity::PhysicalByteRange::new(0, length).unwrap(),
        ),
    )
    .unwrap();
    let request = worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest::new(
        serving.store_identity(),
        [target],
        65_536,
        65_536,
        std::time::Duration::from_secs(30),
    )
    .unwrap();
    let mut handle = serving.start_physical_integrity_scrub(request).unwrap();
    match handle.next_window() {
        worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress::WindowInspected(
            observation,
        ) => {
            assert!(observation.counters.acquired_bytes > 0);
        }
        other => panic!("scrub window did not inspect the selector: {other:?}"),
    }
}
