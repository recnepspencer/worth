#[allow(unused_imports)]
use crate::physical_record_journeys::durability_admission::checkpoint_pressure::*;

use {
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
};

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
        worth_store::physical_runtime::PhysicalDataSettlementOutcome::Settled(_)
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
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = start_checkpoint(&serving, 0x51);
    let frozen_source = handle.source();
    assert!(gate.await_arrival());

    let foreground_iterations = resident_dirty_frames * 32;
    let (completed_foreground, foreground_completion) = std::sync::mpsc::channel();
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
    let progress_deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let final_foreground_lsn = loop {
        match foreground_completion.try_recv() {
            Ok(final_lsn) => break final_lsn,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                panic!("foreground siege worker disconnected")
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }
        if gate.arrival_count() > next_foreground_arrival {
            let release = gate.release_arrival(next_foreground_arrival).unwrap();
            assert!(release.await_resumption());
            next_foreground_arrival += 1;
            continue;
        }
        assert!(
            std::time::Instant::now() < progress_deadline,
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
    assert_eq!(
        during.phase(),
        worth_store::physical_runtime::PhysicalCheckpointProgressPhase::Capture
    );
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
    let maintenance = worth_store::physical_runtime::PhysicalOperationAllocationScope::Maintenance;
    let residency_during_slice = serving.residency_observation().counters();
    assert_eq!(
        residency_during_slice.active_operation_bytes_for(maintenance)
            - residency_before_slice.active_operation_bytes_for(maintenance),
        during.current_capture_bytes()
    );

    gate.release();
    let completed = match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("bounded checkpoint siege failed: {other:?}"),
    };
    assert_eq!(completed.basis().source(), frozen_source);
    assert_eq!(completed.dirty_records(), resident_dirty_frames);
    let terminal_progress = serving
        .checkpoints()
        .start(checkpoint_request(0x51))
        .into_raw();
    let terminal_progress = match terminal_progress {
        worth_proof::TransitionOutcome::Success(handle) => handle.progress(),
        _ => panic!("same-key checkpoint observation must remain joinable"),
    };
    assert_eq!(
        terminal_progress.phase(),
        worth_store::physical_runtime::PhysicalCheckpointProgressPhase::Terminal
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

    let artifact = std::fs::read(store_root.join("families/checkpoint.current")).unwrap();
    let records = checkpoint_records(&artifact);
    assert_eq!(records.iter().filter(|record| record[9] == 7).count(), 1);
    assert_eq!(
        records.len() as u64,
        // Header, compaction header, positive NoRelease custody, and footer.
        resident_dirty_frames + completed.binding_compaction().binding_count() + 4
    );
    assert_checkpoint_io(media_before_checkpoint, serving.media_counters(), &records);

    for dirty in dirty {
        dirty.discard().unwrap();
    }
    serving.close();
}
