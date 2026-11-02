use super::*;
use std::time::{Duration, Instant};
#[test]
fn lost_controllers_keep_only_the_accepted_member_and_its_original_memory_until_terminal() {
    let world = world("owned-controller-loss");
    let records = world.serving().anchored_records().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let port = Port::new(&memory, 3, Delivery::Normal);
    let (operation, recovery) = start(&world, 3, &memory, &cancellation, None);
    let gate = pause(
        world.serving(),
        PhysicalMutationCheckpoint::BeforeEffectCutover,
    );
    let operation = pending(operation, &records, &port);
    assert!(gate.0.await_arrival());
    let registration = port.take_registration();
    let identity = registration.mutation_identity();
    drop(operation);
    drop(recovery);
    drop(port);
    assert!(
        memory.charged_memory_bytes() >= (3 * MEMBER_BYTES + PAYLOAD.len()) as u64,
        "accepted bytes lost original reservation"
    );
    assert!(matches!(
        records.publish(publication(&world, 0)),
        Err(AnchoredRecordPublicationFailure::PublicationBusy)
    ));
    gate.0.release();
    let actual = wait_terminal(&registration);
    assert_eq!(actual.mutation_identity(), identity);
    assert!(matches!(
        actual.outcome(),
        PhysicalMutationOutcome::Completed(_)
    ));
    let deadline = Instant::now() + Duration::from_secs(5);
    while memory.charged_memory_bytes() != 0 {
        assert!(
            Instant::now() < deadline,
            "terminal observer pinned input/view backing"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert!(
        records.read(name(), 1024).unwrap().is_none(),
        "worker advanced a later batch/anchor"
    );
    // Retrying the exact value reuses the ACTUAL completed first batch,
    // without spawning another worker or inventing its member locator.
    let port = Port::new(&memory, 3, Delivery::Normal);
    let (operation, _recovery) = start(&world, 3, &memory, &cancellation, None);
    let quiet = world.serving().physical_mutation_observation();
    let operation = match next(operation, &records, &port) {
        AnchoredRecordPublicationStep::Ready(same) => same,
        AnchoredRecordPublicationStep::Failed(failure) => {
            panic!("completed batch reuse refused: {:?}", failure.cause())
        }
        _ => panic!("completed batch must return its real prefix"),
    };
    assert_eq!(port.registration(0).0, identity);
    assert_eq!(operation.observe().completed_prefix().len(), 1);
    assert_eq!(world.serving().physical_mutation_observation(), quiet);
    assert!(matches!(
        settle(operation.stop(), &records),
        AnchoredRecordStoppedPoll::Settled(_)
    ));
    assert!(records.read(name(), 1024).unwrap().is_none());
    assert!(matches!(
        records.publish(publication(&world, 0)),
        Ok(AnchoredRecordPublished::Published(_))
    ));
}
#[test]
fn registration_panic_returns_the_same_stopped_attempt_with_real_unstarted_settlement() {
    let world = world("owned-registration-panic");
    let records = world.serving().anchored_records().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let port = Port::new(&memory, 0, Delivery::PanicRegistration);
    let (operation, recovery) = start(&world, 0, &memory, &cancellation, None);
    let failure = match operation.step(&records, &port) {
        AnchoredRecordPublicationStep::Failed(f) => f,
        _ => panic!("registration panic lost owning failure"),
    };
    assert!(matches!(
        failure.cause(),
        AnchoredRecordOperationFailure::Panicked
    ));
    let observed = failure.observe();
    assert!(matches!(
        observed.step_terminal().unwrap().outcome(),
        PhysicalMutationOutcome::ProvenNoEffect(_)
    ));
    let registration = port.take_registration();
    assert_eq!(
        observed.accepted_mutation(),
        Some(registration.mutation_identity())
    );
    assert!(matches!(
        wait_terminal(&registration).outcome(),
        PhysicalMutationOutcome::ProvenNoEffect(_)
    ));
    assert!(records.read(name(), 1024).unwrap().is_none());
    assert!(matches!(
        recovery.resume(),
        AnchoredRecordPublicationResume::Stopped(_)
    ));
}
#[test]
fn domain_delivery_panic_preserves_the_actual_selected_receipt() {
    settlement_panic(
        Delivery::PanicTerminal,
        AnchoredRecordStepSettlementFailure::TerminalDeliveryPanicked,
    );
}
#[test]
fn domain_protection_release_panic_preserves_the_actual_selected_receipt() {
    settlement_panic(
        Delivery::PanicRelease,
        AnchoredRecordStepSettlementFailure::ProtectionReleasePanicked,
    );
}
fn settlement_panic(behavior: Delivery, cause: AnchoredRecordStepSettlementFailure) {
    let world = world("owned-delivery-panic");
    let records = world.serving().anchored_records().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let port = Port::new(&memory, 0, behavior);
    let (operation, recovery) = start(&world, 0, &memory, &cancellation, None);
    let failure = match next(operation, &records, &port) {
        AnchoredRecordPublicationStep::Failed(f) => f,
        _ => panic!("domain delivery failure lost completion"),
    };
    assert!(
        matches!(failure.cause(), AnchoredRecordOperationFailure::StepSettlement(actual) if *actual == cause)
    );
    let observed = failure.observe();
    assert!(matches!(
        observed.step_terminal().unwrap().outcome(),
        PhysicalMutationOutcome::Completed(_)
    ));
    let current = records.read(name(), 1024).unwrap().unwrap();
    assert_eq!(current.version(), observed.terminal().unwrap().version());
    assert_eq!(current.bytes(), PAYLOAD);
    assert!(matches!(
        recovery.resume(),
        AnchoredRecordPublicationResume::Terminal(_)
    ));
}
#[test]
fn an_owned_operation_refuses_a_different_live_store_receiver_before_effect() {
    let world = world("owned-original-receiver");
    let other = fixture::world("owned-foreign-receiver");
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let port = Port::new(&memory, 0, Delivery::Normal);
    let (operation, recovery) = start(&world, 0, &memory, &cancellation, None);
    let failure = match operation.step(&other.serving().anchored_records().unwrap(), &port) {
        AnchoredRecordPublicationStep::Failed(f) => f,
        _ => panic!("foreign receiver advanced owned operation"),
    };
    assert!(matches!(
        failure.cause(),
        AnchoredRecordOperationFailure::ForeignReceiver
    ));
    assert!(failure.observe().accepted_mutation().is_none());
    assert!(world
        .serving()
        .anchored_records()
        .unwrap()
        .read(name(), 1024)
        .unwrap()
        .is_none());
    assert!(other
        .serving()
        .anchored_records()
        .unwrap()
        .read(name(), 1024)
        .unwrap()
        .is_none());
    assert!(matches!(
        recovery.resume(),
        AnchoredRecordPublicationResume::Stopped(_)
    ));
}

#[test]
fn caller_unwind_after_acceptance_permanently_stops_advancement_and_retains_its_real_step() {
    let world = world("owned-caller-unwind");
    let records = world.serving().anchored_records().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let port = Port::new(&memory, 3, Delivery::Normal);
    let (operation, recovery) = start(&world, 3, &memory, &cancellation, None);
    let gate = pause(
        world.serving(),
        PhysicalMutationCheckpoint::BeforeEffectCutover,
    );
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _operation = pending(operation, &records, &port);
        assert!(gate.0.await_arrival());
        panic!("original caller failed before delivering its returned endpoint");
    }));
    assert!(unwound.is_err());
    let registration = port.take_registration();
    let stopped = match recovery.resume() {
        AnchoredRecordPublicationResume::Stopped(stopped) => stopped,
        _ => panic!("caller unwind must not recreate an advancing endpoint"),
    };
    assert_eq!(
        stopped.observe().accepted_mutation(),
        Some(registration.mutation_identity())
    );
    gate.0.release();
    let actual = match settle(stopped, &records) {
        AnchoredRecordStoppedPoll::Settled(actual) => actual,
        _ => panic!("accepted member must settle even after caller unwind"),
    };
    assert_eq!(actual.completed_prefix().len(), 1);
    assert!(matches!(
        actual.step_terminal().unwrap().outcome(),
        PhysicalMutationOutcome::Completed(_)
    ));
    assert!(records.read(name(), 1024).unwrap().is_none());
}
