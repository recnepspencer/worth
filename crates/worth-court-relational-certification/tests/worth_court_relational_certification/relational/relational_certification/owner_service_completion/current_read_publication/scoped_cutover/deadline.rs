//! A borrowed cutover can only narrow the original operation's time limits.
use super::*;
use std::time::{Duration, Instant};

#[test]
fn cutoff_expiring_in_the_real_transition_queue_assigns_nothing_and_preserves_the_target() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(&execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let prepared = prepare(&execution, &runtime, &source);
    let positions = runtime.patch_position_reservation_counters().assignments;
    let commits = runtime.history().immutable_commit_count();
    let (ready, issued) = std::sync::mpsc::sync_channel(1);
    let (cutoff, deadline) = std::sync::mpsc::sync_channel(1);
    let ((arrived, sent, queued), outcome) = std::thread::scope(|threads| {
        let mut worker = None;
        let schedule = runtime
            .with_transition_reservation_for_test(&execution, || {
                let runtime = &runtime;
                worker = Some(threads.spawn(move || {
                    let host = serial_request();
                    let execution = ExecutionRequest::serial(&host);
                    runtime
                        .publication_port()
                        .with_prepared_memory_cutover(&execution, prepared, |cutover| {
                            ready.send(()).unwrap();
                            cutover.perform_before(
                                deadline.recv_timeout(Duration::from_secs(5)).unwrap(),
                            );
                        })
                        .unwrap()
                        .into_parts()
                        .1
                        .unwrap()
                }));
                let arrived = issued.recv_timeout(Duration::from_secs(5));
                let expires = Instant::now() + Duration::from_millis(500);
                let sent = cutoff.send(expires);
                let backstop = Instant::now() + Duration::from_secs(5);
                while runtime.waiting_patch_position_publishers_for_test() == 0
                    && Instant::now() < backstop
                {
                    std::thread::yield_now();
                }
                let queued = runtime.waiting_patch_position_publishers_for_test();
                // Expiry is the semantic input. The exact original queued ticket,
                // not elapsed time or a lack of completion, proves the schedule.
                std::thread::sleep(
                    expires.saturating_duration_since(Instant::now()) + Duration::from_millis(1),
                );
                (arrived, sent, queued)
            })
            .unwrap();
        // The original transition reservation is released before joining or
        // making assertions, including an unsuccessful scheduling handshake.
        (schedule, worker.unwrap().join().unwrap())
    });
    assert!(arrived.is_ok() && sent.is_ok());
    assert_eq!(queued, 1);
    assert!(matches!(
        outcome,
        RelationalPublicationOutcome::Denied(RelationalPublicationDenial::CutoverDeadlineElapsed)
    ));
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        positions
    );
    assert_eq!(runtime.history().immutable_commit_count(), commits);
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap(),
        source
    );
    let prepared = prepare(&execution, &runtime, &source);
    let (_, outcome) = runtime
        .publication_port()
        .with_prepared_memory_cutover(&execution, prepared, |cutover| {
            cutover.perform_before(Instant::now() + Duration::from_secs(30));
        })
        .unwrap()
        .into_parts();
    let Some(RelationalPublicationOutcome::Performed(performed)) = outcome else {
        panic!("the same original target and a live cutoff must publish");
    };
    let settled = runtime.settle_performed_publication(performed).unwrap();
    assert_eq!(changed_entities(&settled).len(), 1);
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        positions + 1
    );
    release_test_commit_snapshot(&runtime, &settled);
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn a_looser_cutoff_cannot_mask_the_original_candidates_earlier_expiry() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let policy = worth_relational::facade::config::PublicationConfig {
        coherent_publication_required: true,
        max_patch_records_per_commit: None,
        max_published_snapshot_handles: None,
        max_active_snapshot_handles: None,
        max_transaction_savepoints: 1,
        max_prepared_candidates: None,
        candidate_max_lifetime_millis: 300,
    };
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .schema_registry(fixture().build_registry())
        .publication(policy)
        .build()
        .unwrap();
    let original = create(&execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let prepared = prepare(&execution, &runtime, &source);
    let before = runtime.patch_position_reservation_counters().assignments;
    let scoped = runtime
        .publication_port()
        .with_prepared_memory_cutover(&execution, prepared, |cutover| {
            let additional = Instant::now() + Duration::from_millis(350);
            std::thread::sleep(Duration::from_millis(400));
            cutover.perform_before(additional);
        })
        .unwrap();
    assert!(matches!(scoped.into_parts().1, Some(RelationalPublicationOutcome::Deferred(
        worth_relational::facade::mvcc::RelationalPublicationDeferred::CandidateLifetimeExpired { maximum_lifetime_millis: 300 },
    ))));
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        before
    );
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap(),
        source
    );
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn expired_cutover_deadline_assigns_nothing_and_drops_prepared_payload_after_callback() {
    let host = serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(&execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let probe = Arc::new(Probe::default());
    let _registration = register(&runtime, Arc::clone(&probe));
    let before = runtime.patch_position_reservation_counters().assignments;
    let commits = runtime.history().immutable_commit_count();
    let candidate = prepare(&execution, &runtime, &source);
    let scoped = runtime
        .publication_port()
        .with_prepared_memory_cutover(&execution, candidate, |cutover| {
            let _active = probe.enter();
            cutover.perform_before(Instant::now() - Duration::from_secs(1));
            assert!(probe.events().is_empty());
        })
        .unwrap();
    assert!(matches!(
        scoped.into_parts().1,
        Some(RelationalPublicationOutcome::Denied(
            RelationalPublicationDenial::CutoverDeadlineElapsed
        ))
    ));
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        before
    );
    assert_eq!(runtime.history().immutable_commit_count(), commits);
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap(),
        source
    );
    assert!(probe.events().contains(&(1, false)));
    let candidate = prepare(&execution, &runtime, &source);
    let scoped = runtime
        .publication_port()
        .with_prepared_memory_cutover(&execution, candidate, |cutover| {
            let _active = probe.enter();
            cutover.perform_before(Instant::now() + Duration::from_secs(30));
        })
        .unwrap();
    let Some(RelationalPublicationOutcome::Performed(performed)) = scoped.into_parts().1 else {
        panic!("the same write with a live cutoff must publish");
    };
    let settled = runtime.settle_performed_publication(performed).unwrap();
    assert_eq!(changed_entities(&settled).len(), 1);
    assert!(probe.events().iter().all(|(_, active)| !active));
    release_test_commit_snapshot(&runtime, &settled);
    release_test_commit_snapshot(&runtime, &original);
}

#[test]
fn original_request_stop_precedes_an_expired_additional_cutoff() {
    let cancellation = worth_execution::CancellationSource::new();
    let host = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    let execution = ExecutionRequest::serial(&host);
    let runtime = fixture().build_runtime();
    let original = create(&execution, &runtime, "control", "allowed");
    let source = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .unwrap();
    let candidate = prepare(&execution, &runtime, &source);
    let before = runtime.patch_position_reservation_counters().assignments;
    let scoped = runtime
        .publication_port()
        .with_prepared_memory_cutover(&execution, candidate, |cutover| {
            cancellation.cancel();
            cutover.perform_before(Instant::now() - Duration::from_secs(1));
        })
        .unwrap();
    assert!(matches!(
        scoped.into_parts().1,
        Some(RelationalPublicationOutcome::RequestStopped(
            worth_execution::WorkCeilingDenial::Stopped(worth_execution::MapKernelStop::Cancelled),
        ))
    ));
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        before
    );
    assert_eq!(
        runtime
            .admit_branch_basis(&runtime.main_branch_identity())
            .unwrap(),
        source
    );
    release_test_commit_snapshot(&runtime, &original);
}
