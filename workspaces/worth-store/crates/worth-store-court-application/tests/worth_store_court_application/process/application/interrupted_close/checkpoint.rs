//! Interrupted and overlapping work uses the actual Native/World operation.
use super::*;
fn coupled_fixture(
    path: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
) -> worth_store_testkit_application_oracle::world_home::WorldHomeFixture {
    let fixture = worth_store_testkit_application_oracle::world_home::open(
        home_opening::request(
            path,
            crate::workload::runtime_builder(crate::schema_transition::SOURCE),
        ),
        execution,
    )
    .unwrap();
    drop(fixture.bootstrap_root(execution));
    fixture
}
#[test]
fn close_deadline_retains_the_original_coupled_checkpoint_and_retry_closes_the_same_home() {
    use worth_store::physical_runtime::{
        certification::CertificationPhysicalMutationCheckpoint, AnchoredRecordStepKind,
    };
    use worth_store_application::ApplicationHomeCheckpointDenial;
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = coupled_fixture(directory.path(), &execution);
    let home = fixture.home;
    let operation = home.checkpoint_operation(Default::default()).unwrap();
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
        let checkpoint = operation.checkpoint_with_step_observer_for_test(
            &ExecutionRequest::serial(&interrupted),
            |registration| {
                if registration.kind() == AnchoredRecordStepKind::Anchor {
                    sender
                        .send(
                            operation
                                .physical_serving_for_test()
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
    drop(operation);
    let refused = home
        .close(&ExecutionRequest::serial(&closing))
        .err()
        .expect("the original checkpoint remains pending under the deadline");
    assert!(
        matches!(
            refused.reason(),
            ApplicationHomeCloseDenial::Execution(WorkCeilingDenial::Stopped(
                MapKernelStop::DeadlineElapsed
            ))
        ),
        "{:?}",
        refused.reason()
    );
    assert!(matches!(
        refused.posture(),
        worth_store_application::WorldHomeClosePosture::RecoveryRequired(
            worth_runtime_world::facade::RuntimeWorldOwnerLifecycleObservation::Closing
                | worth_runtime_world::facade::RuntimeWorldOwnerLifecycleObservation::Closed
        )
    ));
    crate::incarnation::assert_contended(directory.path());
    gate.release();
    assert!(matches!(
        refused
            .try_close(&execution)
            .unwrap_or_else(|refused| panic!("same World/Home retry: {:?}", refused.reason())),
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
        Ok(ApplicationHomeCloseOutcome::Closed { .. })
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
    let fixture = coupled_fixture(directory.path(), &execution);
    let home = fixture.home;
    let operation = home.checkpoint_operation(Default::default()).unwrap();
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
        let checkpoint = operation.checkpoint_with_step_observer_for_test(
            &ExecutionRequest::serial(&interrupted),
            |registration| {
                if registration.kind() == AnchoredRecordStepKind::Anchor {
                    sender
                        .send(
                            operation
                                .physical_serving_for_test()
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
    let input_bytes = operation
        .checkpoint_input_memory_bytes_for_test()
        .expect("original accounted input tickets");
    assert!(input_bytes > 0);
    assert!(matches!(
        &checkpoint,
        Err(ApplicationHomeCheckpointDenial::RecoveryRequired(_))
    ));
    drop(checkpoint);
    drop(operation);
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
        Ok(ApplicationHomeCloseOutcome::Closed { .. })
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
    let fixture = coupled_fixture(directory.path(), &execution);
    let home = fixture.home;
    let operation = home.checkpoint_operation(Default::default()).unwrap();
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
        let driver = scope.spawn(|| match point {
            OverlapPoint::PhysicalStepRegistered => operation
                .checkpoint_with_step_observer_for_test(&execution, |registration| {
                    if registration.kind() == AnchoredRecordStepKind::Anchor {
                        pause(Some(registration.mutation_identity()));
                    }
                }),
            OverlapPoint::NativeAcknowledged => operation
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
            operation
                .checkpoint_observation_for_test()
                .unwrap()
                .accepted_mutation(),
            accepted
        );
        match point {
            OverlapPoint::PhysicalStepRegistered => assert!(!observer.acknowledgement_complete),
            OverlapPoint::NativeAcknowledged => {
                assert!(observer.acknowledgement_complete);
                assert_eq!(observer.selected_generation, Some(1));
                assert_eq!(
                    crate::selected_checkpoint::generation_from_serving(
                        operation.physical_serving_for_test()
                    ),
                    1
                );
            }
        }
        drop(observer);
        release.send(()).unwrap();
        driver.join().unwrap().unwrap()
    });
    assert_eq!(result.generation(), 1);
    assert_eq!(
        crate::selected_checkpoint::generation_from_serving(operation.physical_serving_for_test()),
        result.generation()
    );
    assert!(matches!(
        home.recover_checkpoint(&execution),
        Err(ApplicationHomeCheckpointDenial::NoPendingCheckpoint)
    ));
    assert_eq!(
        crate::selected_checkpoint::generation_from_serving(operation.physical_serving_for_test()),
        result.generation()
    );
    drop(operation);
    assert!(matches!(
        home.close(&execution),
        Ok(ApplicationHomeCloseOutcome::Closed { .. })
    ));
}
