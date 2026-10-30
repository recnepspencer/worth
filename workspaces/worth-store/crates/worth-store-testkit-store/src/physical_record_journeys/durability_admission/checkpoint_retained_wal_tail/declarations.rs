pub use {std::sync::mpsc, std::sync::mpsc::TryRecvError};
pub use {std::time::Duration, std::time::Instant};

pub use worth_proof::TransitionOutcome;
pub use worth_signal::facade::TemporalDuration;
pub use worth_store::physical_runtime::certification::CertificationPhysicalRecordSubmission;
pub use worth_store_physical_format::PhysicalCheckpointSource;
pub use {
    worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint,
    worth_store::physical_runtime::certification::CertificationPhysicalExecutionPauseGate,
};
pub use {
    worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    worth_store::physical_runtime::CompletedPhysicalCheckpoint,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointHandle,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointProgressPhase,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome,
    worth_store::physical_runtime::PhysicalWalReclamationObservation,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub use {crate::physical_record_journeys::configuration, crate::physical_record_journeys::durability_with_wal_policy, crate::physical_record_journeys::media, crate::physical_record_journeys::success};
pub use {crate::physical_record_journeys::durability_admission::checkpoint_lifecycle::pause_checkpoint_at_phase};
pub use {crate::physical_record_journeys::durability_admission::independent_wal_oracle::segment_inventory::inspect_wal_inventory};
pub use {crate::physical_record_journeys::durability_admission::wal_rotation::append_group, crate::physical_record_journeys::durability_admission::wal_rotation::prepared, crate::physical_record_journeys::durability_admission::wal_rotation::wal_policy};

pub fn rotation_world(
    store_root: &std::path::Path,
) -> (ServingPhysicalRuntime, AdmittedRecordPlacementPolicy) {
    let media_owner = media(store_root);
    let durability = durability_with_wal_policy(&media_owner, wal_policy(6));
    let (format, placement, access) = configuration();
    let serving = success(media_owner.initialize_record_store(
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        ),
    ));
    (serving, placement)
}

pub fn durable_group(
    submission: &CertificationPhysicalRecordSubmission,
    placement: AdmittedRecordPlacementPolicy,
    members: &[(u8, &[u8])],
) -> u64 {
    let group = append_group(
        submission,
        members
            .iter()
            .map(|(identity, payload)| prepared(submission, placement, *identity, payload))
            .collect(),
    );
    let end_lsn = group
        .members()
        .last()
        .unwrap()
        .mutation()
        .reserved()
        .declaration()
        .lsn_range()
        .end_exclusive()
        .get();
    assert!(matches!(
        submission.synchronize_appended_wal_group(group),
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
    end_lsn
}

#[expect(
    clippy::disallowed_methods,
    reason = "Drives bounded foreground rotations while the fixture probes retained-WAL checkpoint races."
)]
pub fn publish_after_foreground_rotations(
    serving: &ServingPhysicalRuntime,
    submission: CertificationPhysicalRecordSubmission,
    placement: AdmittedRecordPlacementPolicy,
) -> (CompletedPhysicalCheckpoint, PhysicalCheckpointSource) {
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = start_checkpoint(serving);
    let frozen_source = handle.source();
    assert!(gate.await_arrival());
    let (finished, completion) = std::sync::mpsc::channel();
    let foreground = std::thread::spawn(move || {
        durable_group(
            &submission,
            placement,
            &[(2, b"second-a"), (3, b"second-b")],
        );
        durable_group(&submission, placement, &[(4, b"third-a"), (5, b"third-b")]);
        finished.send(()).unwrap();
    });
    release_foreground_arrivals(&gate, &completion);
    foreground.join().unwrap();
    let checkpoint_start = gate.release_arrival(0).unwrap();
    assert!(checkpoint_start.await_resumption());
    gate.release();
    (completed_checkpoint(handle), frozen_source)
}

pub fn release_foreground_arrivals(
    gate: &CertificationPhysicalExecutionPauseGate,
    completion: &std::sync::mpsc::Receiver<()>,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut next_arrival = 1;
    loop {
        match completion.try_recv() {
            Ok(()) => return,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                panic!("retained-tail worker disconnected")
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }
        if gate.arrival_count() > next_arrival {
            let release = gate.release_arrival(next_arrival).unwrap();
            assert!(release.await_resumption());
            next_arrival += 1;
        } else {
            assert!(
                std::time::Instant::now() < deadline,
                "foreground rotations stalled behind checkpoint capture"
            );
            std::thread::yield_now();
        }
    }
}

pub fn assert_independent_tail(
    store_root: &std::path::Path,
    frozen_source: PhysicalCheckpointSource,
    completed: &CompletedPhysicalCheckpoint,
) {
    let independent = inspect_wal_inventory(store_root).unwrap();
    assert_eq!(independent.segments(), &[(2, 1), (3, 1)]);
    let boundary = frozen_source.wal().covered_end_lsn_exclusive();
    let expected = independent
        .segment_facts()
        .iter()
        .copied()
        .filter(|segment| segment.lsn_range().1 > boundary)
        .collect::<Vec<_>>();
    let tail = completed.retained_wal_tail();
    assert_eq!(tail.checkpoint_source(), frozen_source);
    assert_eq!(tail.checkpoint_boundary_lsn().get(), boundary);
    assert_eq!(tail.segments().len(), expected.len());
    for (actual, expected) in tail.segments().iter().zip(&expected) {
        assert_eq!(
            (
                actual.artifact().segment().get(),
                actual.artifact().generation().get()
            ),
            expected.identity()
        );
        assert_eq!(
            (
                actual.observed_lsn_range().start().get(),
                actual.observed_lsn_range().end_exclusive().get()
            ),
            expected.lsn_range()
        );
        assert_eq!(actual.physical_bytes(), expected.byte_count());
    }
    assert_eq!(
        tail.retained_physical_bytes(),
        expected
            .iter()
            .map(|segment| segment.byte_count())
            .sum::<u64>()
    );
    assert_eq!(
        tail.durable_tail_end_lsn_exclusive().get(),
        independent.lsn_range().unwrap().1
    );
}

pub fn assert_reclaimed_prefix(completed: &CompletedPhysicalCheckpoint, expected_segments: u32) {
    match completed.wal_reclamation() {
        worth_store::physical_runtime::PhysicalWalReclamationObservation::Reclaimed(report) => {
            assert_eq!(report.planned_segments(), expected_segments);
            assert_eq!(report.reclaimed_segments(), expected_segments);
            assert!(report.reclaimed_bytes() > 0);
            assert_eq!(report.first_unreclaimed(), None);
        }
        other => panic!("checkpoint did not reclaim its obsolete WAL prefix: {other:?}"),
    }
}

pub fn checkpoint_at_publication_replacement(
    serving: &ServingPhysicalRuntime,
) -> (
    CertificationPhysicalExecutionPauseGate,
    PhysicalCheckpointHandle,
    usize,
) {
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let handle = start_checkpoint(serving);
    let publication_arrival = pause_checkpoint_at_phase(
        &gate,
        &handle,
        worth_store::physical_runtime::PhysicalCheckpointProgressPhase::PublicationReplacement,
    );
    (gate, handle, publication_arrival)
}

#[expect(
    clippy::disallowed_methods,
    reason = "Drives the foreground writer while the fixture controls the checkpoint cutover seam."
)]
pub fn start_cutover_foreground(
    submission: CertificationPhysicalRecordSubmission,
    placement: AdmittedRecordPlacementPolicy,
) -> (std::thread::JoinHandle<()>, std::sync::mpsc::Receiver<u64>) {
    let (started, start_observation) = std::sync::mpsc::channel();
    let (finished, completion) = std::sync::mpsc::channel();
    let foreground = std::thread::spawn(move || {
        started.send(()).unwrap();
        let end_lsn = durable_group(&submission, placement, &[(32, b"must-follow-cutover")]);
        finished.send(end_lsn).unwrap();
    });
    start_observation.recv().unwrap();
    (foreground, completion)
}

pub fn finish_checkpoint_cutover(
    gate: &CertificationPhysicalExecutionPauseGate,
    handle: PhysicalCheckpointHandle,
    publication_arrival: usize,
    foreground: &std::sync::mpsc::Receiver<u64>,
) -> CompletedPhysicalCheckpoint {
    let replace = gate.release_arrival(publication_arrival).unwrap();
    assert!(replace.await_resumption());
    assert!(gate.await_arrivals(publication_arrival + 2));
    assert!(matches!(
        foreground.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Empty)
    ));
    let namespace = gate.release_arrival(publication_arrival + 1).unwrap();
    assert!(namespace.await_resumption());
    completed_checkpoint(handle)
}

pub fn start_checkpoint(serving: &ServingPhysicalRuntime) -> PhysicalCheckpointHandle {
    match serving.checkpoints().start(checkpoint_request()).into_raw() {
        worth_proof::TransitionOutcome::Success(handle) => handle,
        _ => panic!("retained-tail checkpoint was not admitted"),
    }
}

pub fn completed_checkpoint(handle: PhysicalCheckpointHandle) -> CompletedPhysicalCheckpoint {
    match handle.wait() {
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(completed) => completed,
        other => panic!("retained-tail checkpoint did not complete: {other:?}"),
    }
}

pub fn checkpoint_request() -> PhysicalCheckpointRequest {
    worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x73; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::at(
            worth_signal::facade::TemporalDuration::temporal_duration(10_000).unwrap(),
        ),
    )
}
