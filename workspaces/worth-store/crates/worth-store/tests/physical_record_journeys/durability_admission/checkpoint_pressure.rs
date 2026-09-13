use std::{
    fs,
    sync::mpsc::{self, TryRecvError},
    time::{Duration, Instant},
};

use worth_proof::{NonEmpty, TransitionOutcome};
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint;
use worth_store::physical_runtime::{
    DataDispatchedPhysicalMutation, PhysicalCheckpointDeadline, PhysicalCheckpointIdempotencyKey,
    PhysicalCheckpointOutcome, PhysicalCheckpointProgressPhase, PhysicalCheckpointRequest,
    PhysicalDataDispatchOutcome, PhysicalDataSettlementOutcome,
    PhysicalMutationIdempotencyMaterial, PhysicalOperationAllocationScope,
    PhysicalWalGroupAppendOutcome, PhysicalWalGroupBarrierOutcome, ServingPhysicalRuntime,
    WalDurablePhysicalMutation,
};
use worth_store_physical_backend::MediaOperationRole;

use super::super::{configuration, serving_from_initialization};

#[test]
fn whole_store_checkpoint_stays_bounded_during_32x_foreground_mutation() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let serving = serving_from_initialization(&store_root);
    let (format, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let payload = vec![0x6c; format.declaration().page_size().bytes() as usize * 3];
    let dispatched = dispatch_dirty_data(&serving, placement, payload);
    let coordinates = dispatched
        .effects()
        .iter()
        .map(|effect| effect.coordinate())
        .collect::<Vec<_>>();
    assert!(matches!(
        dispatched.settle_exact_effects(),
        PhysicalDataSettlementOutcome::Settled(_)
    ));
    let residency = serving.certification_physical_residency();
    let dirty = coordinates
        .into_iter()
        .enumerate()
        .map(|(index, coordinate)| {
            let resident = residency.pin_exact(coordinate).unwrap();
            residency
                .admit_dirty_frame(resident, |source, target| {
                    target.copy_from_slice(source);
                    let last = target.len() - 1;
                    target[last] ^= (index as u8).wrapping_add(1);
                })
                .unwrap()
        })
        .collect::<Vec<_>>();
    let resident_dirty_frames = dirty.len() as u64;
    assert!(
        resident_dirty_frames > 0,
        "siege requires dirty source frames"
    );
    assert_eq!(
        serving.residency_observation().counters().dirty_frames() as u64,
        resident_dirty_frames
    );

    let gate = serving.certification_pause_physical_execution_at(
        CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = start_checkpoint(&serving, 0x51);
    let frozen_source = handle.source();
    assert!(gate.await_arrival());

    let foreground_iterations = resident_dirty_frames * 32;
    let (completed_foreground, foreground_completion) = mpsc::channel();
    let foreground = std::thread::spawn(move || {
        let mut final_lsn = frozen_source.wal().covered_end_lsn_exclusive();
        for index in 0..foreground_iterations {
            let (_durable, end_lsn) = durable_wal(
                &submission,
                placement,
                mutation_material(index),
                &index.to_le_bytes(),
            );
            final_lsn = end_lsn;
        }
        completed_foreground.send(final_lsn).unwrap();
    });
    let mut next_foreground_arrival = 1;
    let progress_deadline = Instant::now() + Duration::from_secs(30);
    let final_foreground_lsn = loop {
        match foreground_completion.try_recv() {
            Ok(final_lsn) => break final_lsn,
            Err(TryRecvError::Disconnected) => panic!("foreground siege worker disconnected"),
            Err(TryRecvError::Empty) => {}
        }
        if gate.arrival_count() > next_foreground_arrival {
            let release = gate.release_arrival(next_foreground_arrival).unwrap();
            assert!(release.await_resumption());
            next_foreground_arrival += 1;
            continue;
        }
        assert!(
            Instant::now() < progress_deadline,
            "foreground mutation stopped behind checkpoint capture"
        );
        std::thread::yield_now();
    };
    foreground.join().unwrap();
    assert!(final_foreground_lsn > frozen_source.wal().covered_end_lsn_exclusive());

    let media_before_checkpoint = serving.media_counters();
    let residency_before_slice = serving.residency_observation().counters();
    let checkpoint_append_arrival = gate.arrival_count();
    let first_action = gate.release_arrival(0).unwrap();
    assert!(first_action.await_resumption());
    assert!(gate.await_arrivals(checkpoint_append_arrival + 1));

    let during = handle.progress();
    assert_eq!(during.phase(), PhysicalCheckpointProgressPhase::Capture);
    assert!(during.current_capture_bytes() > 0);
    assert_eq!(during.peak_capture_bytes(), during.current_capture_bytes());
    assert!(
        during.current_capture_bytes()
            <= serving
                .durability_observation()
                .checkpoint_policy()
                .memory_limit()
                .get()
                .get()
    );
    let maintenance = PhysicalOperationAllocationScope::Maintenance;
    let residency_during_slice = serving.residency_observation().counters();
    assert_eq!(
        residency_during_slice.active_operation_bytes_for(maintenance)
            - residency_before_slice.active_operation_bytes_for(maintenance),
        during.current_capture_bytes()
    );

    gate.release();
    let completed = match handle.wait() {
        PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("bounded checkpoint siege failed: {other:?}"),
    };
    assert_eq!(completed.basis().source(), frozen_source);
    assert_eq!(completed.dirty_records(), resident_dirty_frames);
    let terminal_progress = serving
        .checkpoints()
        .start(checkpoint_request(0x51))
        .into_raw();
    let terminal_progress = match terminal_progress {
        TransitionOutcome::Success(handle) => handle.progress(),
        _ => panic!("same-key checkpoint observation must remain joinable"),
    };
    assert_eq!(
        terminal_progress.phase(),
        PhysicalCheckpointProgressPhase::Terminal
    );
    assert_eq!(terminal_progress.current_capture_bytes(), 0);
    assert_eq!(
        terminal_progress.peak_capture_bytes(),
        during.peak_capture_bytes()
    );
    assert_eq!(
        serving
            .residency_observation()
            .counters()
            .active_operation_bytes_for(maintenance),
        residency_before_slice.active_operation_bytes_for(maintenance)
    );

    let artifact = fs::read(store_root.join("families/checkpoint.current")).unwrap();
    let records = checkpoint_records(&artifact);
    assert_eq!(
        records.len() as u64,
        resident_dirty_frames + completed.binding_compaction().binding_count() + 3
    );
    assert_checkpoint_io(media_before_checkpoint, serving.media_counters(), &records);

    for dirty in dirty {
        dirty.discard().unwrap();
    }
    serving.close();
}

fn dispatch_dirty_data(
    serving: &ServingPhysicalRuntime,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    payload: Vec<u8>,
) -> DataDispatchedPhysicalMutation {
    let submission = serving.certification_record_submission();
    let (durable, _end_lsn) = durable_wal(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([0x41; 32]),
        &payload,
    );
    match submission.dispatch_wal_durable_data(durable) {
        PhysicalDataDispatchOutcome::Dispatched(dispatched) => dispatched,
        _ => panic!("siege setup requires real dirty data effects"),
    }
}

fn durable_wal(
    submission: &worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission,
    placement: worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    material: PhysicalMutationIdempotencyMaterial,
    payload: &[u8],
) -> (WalDurablePhysicalMutation, u64) {
    let prepared = super::wal_append::prepared(submission, placement, material, payload);
    let appended = match submission.append_prepared_wal_group(NonEmpty::new(prepared, Vec::new())) {
        PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("foreground WAL mutation must append while checkpoint is paused"),
    };
    let end_lsn = appended.members()[0]
        .mutation()
        .reserved()
        .declaration()
        .lsn_range()
        .end_exclusive()
        .get();
    let durable = match submission.synchronize_appended_wal_group(appended) {
        PhysicalWalGroupBarrierOutcome::Durable(durable) => durable
            .into_members()
            .into_vec()
            .pop()
            .expect("singleton WAL group derives one member"),
        _ => panic!("foreground WAL mutation must cross its barrier"),
    };
    (durable, end_lsn)
}

fn start_checkpoint(
    serving: &ServingPhysicalRuntime,
    key: u8,
) -> worth_store::physical_runtime::PhysicalCheckpointHandle {
    match serving
        .checkpoints()
        .start(checkpoint_request(key))
        .into_raw()
    {
        TransitionOutcome::Success(handle) => handle,
        _ => panic!("checkpoint siege requires an admitted handle"),
    }
}

fn checkpoint_request(key: u8) -> PhysicalCheckpointRequest {
    PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([key; 32]),
        PhysicalCheckpointDeadline::at(TemporalDuration::temporal_duration(10_000).unwrap()),
    )
}

fn mutation_material(index: u64) -> PhysicalMutationIdempotencyMaterial {
    let mut material = [0_u8; 32];
    material[..8].copy_from_slice(&index.to_le_bytes());
    material[8] = 0x9d;
    PhysicalMutationIdempotencyMaterial::new(material)
}

fn assert_checkpoint_io(
    before: worth_store_physical_backend::MediaCounterSnapshot,
    after: worth_store_physical_backend::MediaCounterSnapshot,
    records: &[&[u8]],
) {
    const PHYSICAL_EFFECT_RECOVERY_RECORD_BYTES: u64 = 160;

    let delta = |role| {
        after
            .attempts_for(role)
            .saturating_sub(before.attempts_for(role))
    };
    let checkpoint_actions = records.len() as u64 + 3;
    assert_eq!(delta(MediaOperationRole::CreateNew), checkpoint_actions + 1);
    assert_eq!(
        delta(MediaOperationRole::PositionedWrite),
        records.len() as u64
    );
    assert_eq!(delta(MediaOperationRole::Append), checkpoint_actions);
    assert_eq!(
        delta(MediaOperationRole::SynchronizeFileState),
        checkpoint_actions + 1
    );
    assert_eq!(delta(MediaOperationRole::Delete), checkpoint_actions);
    assert_eq!(delta(MediaOperationRole::AtomicReplace), 1);
    assert_eq!(
        delta(MediaOperationRole::SynchronizeDirectoryPublication),
        checkpoint_actions * 2 + 1
    );
    assert_eq!(
        after.completed_bytes_for(MediaOperationRole::PositionedWrite)
            - before.completed_bytes_for(MediaOperationRole::PositionedWrite),
        records
            .iter()
            .map(|record| record.len() as u64)
            .sum::<u64>()
    );
    assert_eq!(
        after.completed_bytes_for(MediaOperationRole::Append)
            - before.completed_bytes_for(MediaOperationRole::Append),
        checkpoint_actions * PHYSICAL_EFFECT_RECOVERY_RECORD_BYTES
    );
}

fn checkpoint_records(bytes: &[u8]) -> Vec<&[u8]> {
    let mut records = Vec::new();
    let mut offset = 0;
    while offset < bytes.len() {
        assert!(bytes.len() - offset >= 20);
        let payload = u32::from_le_bytes(bytes[offset + 12..offset + 16].try_into().unwrap());
        let end = offset + 16 + payload as usize + 4;
        assert!(end <= bytes.len());
        records.push(&bytes[offset..end]);
        offset = end;
    }
    records
}
