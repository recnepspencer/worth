use super::*;
use std::time::{Duration, Instant};

#[test]
fn narrowing_cutoff_survives_world_handoff_and_the_actual_native_transition_wait() {
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let court = CompositeSupplyChainCourt::compile();
    let current = court.bootstrap();
    let target = fork(&court, &execution, &current);
    let prepared = prepare(&court, &execution, &target, &current);
    let runtime = &court.records.runtime;
    let positions = runtime.patch_position_reservation_counters().assignments;
    let commits = runtime.history().immutable_commit_count();
    let (ready, issued) = std::sync::mpsc::sync_channel(1);
    let (send, receive) = std::sync::mpsc::sync_channel(1);
    let ((arrived, sent, queued, locked), (caller, outcome)) = std::thread::scope(|threads| {
        let mut worker = None;
        let schedule = runtime
            .with_transition_reservation_for_test(&execution, || {
                let port = court.world.publication_port();
                worker = Some(threads.spawn(move || {
                    let request = crate::test_execution_policy::serial_request();
                    let execution = worth_execution::ExecutionRequest::serial(&request);
                    port.execute_without_signal_with_memory_cutover(
                        execution,
                        prepared,
                        &RuntimeWorldCancellationSource::new().token(),
                        |cutover| {
                            ready.send(()).unwrap();
                            cutover.perform_before(
                                receive.recv_timeout(Duration::from_secs(5)).unwrap(),
                            );
                            "callback returned"
                        },
                    )
                }));
                let arrived = issued.recv_timeout(Duration::from_secs(5));
                let expires = Instant::now() + Duration::from_millis(500);
                let sent = send.send(expires);
                let backstop = Instant::now() + Duration::from_secs(5);
                while runtime.waiting_patch_position_publishers_for_test() == 0
                    && Instant::now() < backstop
                {
                    std::thread::yield_now();
                }
                let queued = runtime.waiting_patch_position_publishers_for_test();
                let locked = court
                    .world
                    .observation_port()
                    .product_branch_writers_are_locked_out_for_test(&current)
                    .unwrap();
                std::thread::sleep(
                    expires.saturating_duration_since(Instant::now()) + Duration::from_millis(1),
                );
                (arrived, sent, queued, locked)
            })
            .unwrap();
        // Release the actual reservation before assertions and worker join.
        (schedule, worker.unwrap().join().unwrap())
    });
    assert!(arrived.is_ok() && sent.is_ok());
    assert_eq!(queued, 1);
    assert!(locked);
    assert_eq!(caller, Some("callback returned"));
    let RuntimeWorldPublicationOutcome::NoEffect(no) = outcome else {
        panic!("expired original wait must refuse")
    };
    assert_eq!(
        no.cause(),
        NoEffectCause::CurrentReadDependency(
            CurrentProductReadDependencyDenial::CutoverDeadlineElapsed
        )
    );
    assert_eq!(
        runtime.patch_position_reservation_counters().assignments,
        positions
    );
    assert_eq!(runtime.history().immutable_commit_count(), commits);
    assert_eq!(court.observe(&current), current);
    assert_eq!(court.observe(&target), target);
    assert_eq!(
        direct_basis(&court, &target),
        *target.basis().relational_basis()
    );
    let (_, outcome) = court
        .world
        .publication_port()
        .execute_without_signal_with_memory_cutover(
            execution,
            prepare(&court, &execution, &target, &current),
            &RuntimeWorldCancellationSource::new().token(),
            |cutover| cutover.perform_before(Instant::now() + Duration::from_secs(30)),
        );
    let RuntimeWorldPublicationOutcome::Performed(done) = outcome else {
        panic!("same write and live cutoff performs")
    };
    assert_eq!(grain(&court, &execution, &court.observe(&target)), "5");
    drop(done.consume());
}
