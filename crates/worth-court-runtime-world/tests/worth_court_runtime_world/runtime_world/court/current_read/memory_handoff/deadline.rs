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

#[derive(Clone)]
struct AtomicWorldClock(std::sync::Arc<std::sync::atomic::AtomicU64>);

impl AtomicWorldClock {
    fn new(ticks: u64) -> Self {
        Self(std::sync::Arc::new(std::sync::atomic::AtomicU64::new(
            ticks,
        )))
    }

    fn set(&self, ticks: u64) {
        self.0.store(ticks, std::sync::atomic::Ordering::Release);
    }
}

impl RuntimeWorldClockSource for AtomicWorldClock {
    fn now(&self) -> RuntimeWorldInstant {
        RuntimeWorldInstant::from_ticks(self.0.load(std::sync::atomic::Ordering::Acquire))
    }
}

fn history_marker(court: &CompositeSupplyChainCourt) -> (usize, usize, u64, u64) {
    let snap = court.world.inspection_port().history_snapshot().unwrap();
    (
        snap.installed_commits(),
        snap.reserved_commits(),
        snap.costs().metadata_promotions(),
        snap.costs().reachability_rows_installed(),
    )
}

fn prepare_with_deadline(
    court: &CompositeSupplyChainCourt,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    target: &ProductBranchObservation,
    current: &ProductBranchObservation,
    token: &RuntimeWorldCancellationToken,
    deadline: RuntimeWorldInstant,
) -> PreparedCompositePublicationWithoutSignal {
    let candidate =
        court
            .records
            .candidate(execution, target.basis().relational_basis(), "grain", "5");
    court
        .world
        .publication_port()
        .prepare_without_signal(
            target.clone(),
            intent(
                CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                    .with_prepared_relational_candidate(candidate),
                current,
            ),
            token,
            Some(deadline),
        )
        .unwrap()
}

#[test]
fn world_cancellation_and_deadline_during_native_transition_wait_stop_before_native_effect() {
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(8_000_000)),
    );
    for trigger_cancel in [true, false] {
        let clock = AtomicWorldClock::new(0);
        let cancel = std::sync::Arc::new(RuntimeWorldCancellationSource::new());
        let court = CompositeSupplyChainCourt::compile_with_clock(RuntimeWorldClock::from_source(
            clock.clone(),
        ));
        let host_request = crate::test_execution_policy::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host_request);
        let current = court.bootstrap();
        let target = fork(&court, &execution, &current);
        let before_history = history_marker(&court);
        assert_eq!(before_history.1, 0, "no reserved commits before prepare");
        let prepared = prepare_with_deadline(
            &court,
            &execution,
            &target,
            &current,
            &cancel.token(),
            RuntimeWorldInstant::from_ticks(10),
        );
        assert_eq!(
            history_marker(&court).1,
            1,
            "prepare reserves one provisional history slot"
        );
        let runtime = &court.records.runtime;
        let positions = runtime.patch_position_reservation_counters().assignments;
        let commits = runtime.history().immutable_commit_count();
        let before_target_basis = direct_basis(&court, &target);
        let before_current_basis = direct_basis(&court, &current);
        assert_eq!(
            court
                .world
                .inspection_port()
                .recovery_snapshot()
                .unwrap()
                .installed(),
            0
        );
        let mem = worth_execution::SerialMemoryBudget::from_policy(&policy);
        let (ready, issued) = std::sync::mpsc::sync_channel(1);
        let ((arrived, queued, locked), (caller, outcome)) = std::thread::scope(|threads| {
            let mut worker = None;
            let schedule = runtime
                .with_transition_reservation_for_test(&execution, || {
                    let port = court.world.publication_port();
                    let worker_cancel = std::sync::Arc::clone(&cancel);
                    let worker_mem = mem.clone();
                    worker = Some(threads.spawn(move || {
                        let req = worth_execution::SerialRequest::from_memory(
                            worker_mem,
                            worth_execution::CancellationToken::new(),
                            None,
                        );
                        port.execute_without_signal_with_memory_cutover(
                            worth_execution::ExecutionRequest::serial(&req),
                            prepared,
                            &worker_cancel.token(),
                            |cutover| {
                                ready.send(()).unwrap();
                                cutover.perform();
                                "callback returned"
                            },
                        )
                    }));
                    let arrived = issued.recv_timeout(Duration::from_secs(5));
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
                    if trigger_cancel {
                        cancel.cancel();
                    } else {
                        clock.set(10);
                    }
                    (arrived, queued, locked)
                })
                .unwrap();
            (schedule, worker.unwrap().join().unwrap())
        });
        assert!(arrived.is_ok());
        assert_eq!(queued, 1);
        assert!(locked);
        assert_eq!(caller, Some("callback returned"));
        let RuntimeWorldPublicationOutcome::NoEffect(no) = outcome else {
            panic!("World stop while queued on Native transition reservation must refuse")
        };
        let expected_cause = if trigger_cancel {
            NoEffectCause::CancelledBeforeEffect
        } else {
            NoEffectCause::DeadlineBeforeEffect
        };
        assert_eq!(no.cause(), expected_cause);
        assert_eq!(
            runtime.patch_position_reservation_counters().assignments,
            positions
        );
        assert_eq!(runtime.history().immutable_commit_count(), commits);
        assert_eq!(direct_basis(&court, &target), before_target_basis);
        assert_eq!(direct_basis(&court, &current), before_current_basis);
        assert_eq!(history_marker(&court), before_history);
        assert_eq!(court.observe(&current), current);
        assert_eq!(court.observe(&target), target);
        assert_eq!(
            court
                .world
                .inspection_port()
                .recovery_snapshot()
                .unwrap()
                .installed(),
            0
        );
        court.assert_no_pending_attempts();
        assert_eq!(mem.charged_memory_bytes(), 0);

        let live_token = RuntimeWorldCancellationSource::new().token();
        let (_, outcome) = court
            .world
            .publication_port()
            .execute_without_signal_with_memory_cutover(
                execution,
                prepare_with_deadline(
                    &court,
                    &execution,
                    &target,
                    &current,
                    &live_token,
                    RuntimeWorldInstant::from_ticks(20),
                ),
                &live_token,
                |cutover| cutover.perform(),
            );
        let RuntimeWorldPublicationOutcome::Performed(done) = outcome else {
            panic!("subsequent live write must still perform")
        };
        assert_eq!(grain(&court, &execution, &court.observe(&target)), "5");
        drop(done.consume());
        court.assert_no_pending_attempts();
    }
}
