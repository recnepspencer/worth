//! Interruption after physical drain returns the one running close and lease.
use worth_execution::{
    CancellationSource, ExecutionRequest, MapKernelStop, SerialMemoryBudget, SerialRequest,
    WorkCeilingDenial,
};
use worth_relational::facade::runtime::RelationalRuntimeApi;
use worth_store::physical_runtime::PhysicalStoreClosePhase;
use worth_store_application::{
    ApplicationHomeCloseDenial, ApplicationHomeCloseOutcome, ApplicationHomeClosePosture,
};
use worth_store_testkit_application_oracle::facade::home_opening;

#[test]
fn cancellation_after_drain_retains_the_running_close_and_resume_releases_its_lease() {
    interrupted_close(false);
}

#[test]
fn deadline_after_drain_retains_the_running_close_and_resume_releases_its_lease() {
    interrupted_close(true);
}

fn interrupted_close(deadline: bool) {
    let directory = tempfile::tempdir().unwrap();
    let request = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    let home = home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &execution,
    )
    .unwrap();
    let identity = home.home_identity();
    let cancellation = CancellationSource::new();
    let closing = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        cancellation.token(),
        deadline.then(|| std::time::Instant::now() + std::time::Duration::from_secs(5)),
    );
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let (outcome, gate) = std::thread::scope(|scope| {
        let cancellation = &cancellation;
        let controller = scope.spawn(move || {
            let gate: worth_store::physical_runtime::certification::CertificationPhysicalClosePauseGate =
                receiver.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
            assert!(gate.await_arrival(), "the actual Store drain started before interruption");
            if !deadline { cancellation.cancel(); }
            gate
        });
        let outcome = home.close_with_physical_observer_for_test(
            &ExecutionRequest::serial(&closing),
            move |plan| {
                let gate = plan.certification_pause_at(PhysicalStoreClosePhase::SignalDisposed);
                sender.send(gate).unwrap();
            },
        );
        (outcome, controller.join().unwrap())
    });
    let custody = match outcome {
        ApplicationHomeCloseOutcome::CloseRefused {
            reason: ApplicationHomeCloseDenial::Execution(WorkCeilingDenial::Stopped(cause)),
            posture: ApplicationHomeClosePosture::RecoveryRequired(custody),
        } => {
            assert_eq!(
                cause,
                if deadline {
                    MapKernelStop::DeadlineElapsed
                } else {
                    MapKernelStop::Cancelled
                }
            );
            custody
        }
        ApplicationHomeCloseOutcome::Closed { .. }
        | ApplicationHomeCloseOutcome::CloseRefused { .. } => {
            panic!("interruption must return the running close, never a serving owner")
        }
    };
    assert_eq!(custody.home_identity(), identity);
    assert_eq!(custody.location(), directory.path());
    let progress = custody.physical_close_observation().unwrap();
    assert!(progress.reached(PhysicalStoreClosePhase::SignalDisposed));
    assert!(!progress.reached(PhysicalStoreClosePhase::MediaReleased));
    crate::incarnation::assert_contended(directory.path());
    gate.release();
    assert!(matches!(
        custody.try_release(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
    assert!(
        progress.reached(PhysicalStoreClosePhase::MediaReleased),
        "resume completed the observed original task"
    );
    let reopened = home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &execution,
    )
    .unwrap();
    assert_eq!(reopened.home_identity(), identity);
    assert!(matches!(
        reopened.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}

#[test]
fn close_deadline_retains_the_stopped_native_owner_until_its_accepted_checkpoint_settles() {
    use worth_store::physical_runtime::{
        certification::CertificationPhysicalMutationCheckpoint, AnchoredRecordStepKind,
    };
    use worth_store_application::ApplicationHomeCheckpointDenial;
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &execution,
    )
    .unwrap();
    let identity = home.home_identity();
    let cancellation = CancellationSource::new();
    let interrupted =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let (checkpoint, gate) = std::thread::scope(|scope| {
        let controller = scope.spawn(move || {
            let gate: worth_store::physical_runtime::certification::CertificationPhysicalMutationPauseGate = receiver.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
            assert!(gate.await_arrival());
            cancellation.cancel();
            gate
        });
        let checkpoint = home.checkpoint_with_step_observer_for_test(
            &ExecutionRequest::serial(&interrupted),
            |registration| {
                if registration.kind() == AnchoredRecordStepKind::Anchor {
                    sender
                        .send(
                            home.physical_serving_for_test()
                                .certification_pause_physical_mutation_at(
                                    CertificationPhysicalMutationCheckpoint::AfterRootReplacement,
                                ),
                        )
                        .unwrap();
                }
            },
        );
        (checkpoint, controller.join().unwrap())
    });
    assert!(matches!(
        checkpoint,
        Err(ApplicationHomeCheckpointDenial::RecoveryRequired(_))
    ));
    let closing = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        Some(std::time::Instant::now() + std::time::Duration::from_millis(25)),
    );
    let custody = match home.close(&ExecutionRequest::serial(&closing)) {
        ApplicationHomeCloseOutcome::CloseRefused {
            reason:
                ApplicationHomeCloseDenial::Execution(WorkCeilingDenial::Stopped(
                    MapKernelStop::DeadlineElapsed,
                )),
            posture: ApplicationHomeClosePosture::RecoveryRequired(custody),
        } => custody,
        _ => panic!("unsettled accepted checkpoint retains the stopped Native owner in recovery"),
    };
    assert_eq!(custody.home_identity(), identity);
    assert!(
        custody.physical_close_observation().is_none(),
        "physical release has not started"
    );
    crate::incarnation::assert_contended(directory.path());
    gate.release();
    assert!(matches!(
        custody.try_release(&execution),
        ApplicationHomeCloseOutcome::Closed {
            checkpoint: Ok(_),
            ..
        }
    ));
    let home = home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &execution,
    )
    .unwrap();
    assert_eq!(home.home_identity(), identity);
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}

#[test]
fn last_home_loss_keeps_original_checkpoint_input_tickets_until_actual_terminal() {
    use worth_store::physical_runtime::{
        certification::CertificationPhysicalMutationCheckpoint, AnchoredRecordStepKind,
    };
    use worth_store_application::ApplicationHomeCheckpointDenial;
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &execution,
    )
    .unwrap();
    let identity = home.home_identity();
    let budget = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let interrupted = SerialRequest::from_memory(budget.clone(), cancellation.token(), None);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let (checkpoint, gate) = std::thread::scope(|scope| {
        let controller = scope.spawn(move || {
            let gate: worth_store::physical_runtime::certification::CertificationPhysicalMutationPauseGate = receiver.recv_timeout(std::time::Duration::from_secs(5)).unwrap();
            assert!(gate.await_arrival());
            cancellation.cancel();
            gate
        });
        let checkpoint = home.checkpoint_with_step_observer_for_test(
            &ExecutionRequest::serial(&interrupted),
            |registration| {
                if registration.kind() == AnchoredRecordStepKind::Anchor {
                    sender
                        .send(
                            home.physical_serving_for_test()
                                .certification_pause_physical_mutation_at(
                                    CertificationPhysicalMutationCheckpoint::AfterRootReplacement,
                                ),
                        )
                        .unwrap();
                }
            },
        );
        (checkpoint, controller.join().unwrap())
    });
    let input_bytes = home
        .checkpoint_input_memory_bytes_for_test()
        .expect("original accounted input tickets");
    assert!(input_bytes > 0);
    assert!(matches!(
        &checkpoint,
        Err(ApplicationHomeCheckpointDenial::RecoveryRequired(_))
    ));
    drop(checkpoint);
    drop(home);
    assert!(
        budget.charged_memory_bytes() >= input_bytes,
        "actual accepted bytes must retain their original tickets after the last Home disappears"
    );
    gate.release();
    let terminal_deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while budget.charged_memory_bytes() != 0 {
        assert!(
            std::time::Instant::now() < terminal_deadline,
            "terminal cleanup retained original input tickets"
        );
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    let home = home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &execution,
    )
    .unwrap();
    assert_eq!(home.home_identity(), identity);
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}

#[test]
fn overlapping_recovery_observes_the_same_attempt_without_replacing_its_driver() {
    overlapping_recovery(OverlapPoint::PhysicalStepRegistered);
}

#[test]
fn overlapping_recovery_after_native_acknowledgement_cannot_clear_before_wal_start() {
    overlapping_recovery(OverlapPoint::NativeAcknowledged);
}

#[derive(Clone, Copy)]
enum OverlapPoint {
    PhysicalStepRegistered,
    NativeAcknowledged,
}

fn overlapping_recovery(point: OverlapPoint) {
    use worth_store::physical_runtime::AnchoredRecordStepKind;
    use worth_store_application::ApplicationHomeCheckpointDenial;
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = home_opening::open(
        home_opening::request(directory.path(), RelationalRuntimeApi::builder()),
        &execution,
    )
    .unwrap();
    let (entered, await_entry) = std::sync::mpsc::sync_channel(1);
    let (release, await_release) = std::sync::mpsc::sync_channel(1);
    let await_release = std::sync::Mutex::new(await_release);
    let pause = |accepted| {
        entered.send(accepted).unwrap();
        await_release
            .lock()
            .unwrap()
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
    };
    let result = std::thread::scope(|scope| {
        let driver =
            scope.spawn(|| match point {
                OverlapPoint::PhysicalStepRegistered => home
                    .checkpoint_with_step_observer_for_test(&execution, |registration| {
                        if registration.kind() == AnchoredRecordStepKind::Anchor {
                            pause(Some(registration.mutation_identity()));
                        }
                    }),
                OverlapPoint::NativeAcknowledged => home
                    .checkpoint_with_acknowledgement_observer_for_test(&execution, || pause(None)),
            });
        let accepted = await_entry
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap();
        let competing = crate::workload::serial_request();
        let observer = match home.recover_checkpoint(&ExecutionRequest::serial(&competing)) {
            Err(ApplicationHomeCheckpointDenial::RecoveryRequired(observer)) => observer,
            other => {
                panic!("a competing driver only observes the actual pending attempt: {other:?}")
            }
        };
        assert_eq!(observer.accepted_mutation(), accepted);
        assert_eq!(
            home.checkpoint_observation().unwrap().accepted_mutation(),
            accepted
        );
        match point {
            OverlapPoint::PhysicalStepRegistered => assert!(!observer.acknowledgement_complete),
            OverlapPoint::NativeAcknowledged => {
                assert!(observer.acknowledgement_complete);
                assert_eq!(observer.selected_generation, Some(1));
                assert_eq!(crate::selected_checkpoint::generation(&home), 1);
            }
        }
        drop(observer);
        release.send(()).unwrap();
        driver.join().unwrap().unwrap()
    });
    assert_eq!(result.generation(), 1);
    assert_eq!(
        crate::selected_checkpoint::generation(&home),
        result.generation()
    );
    assert!(matches!(
        home.recover_checkpoint(&execution),
        Err(ApplicationHomeCheckpointDenial::NoPendingCheckpoint)
    ));
    assert_eq!(
        crate::selected_checkpoint::generation(&home),
        result.generation()
    );
    assert!(matches!(
        home.close(&execution),
        ApplicationHomeCloseOutcome::Closed { .. }
    ));
}
