use super::*;
use std::time::{Duration, Instant};
#[test]
fn original_cancellation_after_before_wal_append_refuses_the_actual_group_seal() {
    pre_seal_stop(false);
}
#[test]
fn original_absolute_deadline_after_before_wal_append_refuses_the_actual_group_seal() {
    pre_seal_stop(true);
}
fn pre_seal_stop(deadline_stop: bool) {
    let world = world("owned-original-cutover");
    let records = world.serving().anchored_records().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let members = if deadline_stop { 0 } else { 3 };
    let port = Port::new(&memory, members, Delivery::Normal);
    let expires = deadline_stop.then(|| Instant::now() + Duration::from_secs(2));
    let (operation, recovery) = start(&world, members, &memory, &cancellation, expires);
    let gate = pause(world.serving(), PhysicalMutationCheckpoint::BeforeWalAppend);
    let operation = pending(operation, &records, &port);
    assert!(gate.0.await_arrival());
    let identity = operation.observe().accepted_mutation().unwrap();
    if let Some(expires) = expires {
        while Instant::now() < expires {
            std::thread::sleep(Duration::from_millis(1));
        }
    } else {
        cancellation.cancel();
    }
    let stopped = operation.stop();
    gate.0.release();
    let failure = match settle(stopped, &records) {
        AnchoredRecordStoppedPoll::Failed(f) => f,
        _ => panic!("pre-seal stop must return its actual refusal"),
    };
    let observed = failure.observe();
    let terminal = observed.step_terminal().unwrap();
    assert_eq!(terminal.mutation_identity(), identity);
    let PhysicalMutationOutcome::ProvenNoEffect(fate) = terminal.outcome() else {
        panic!("original controls were not checked at seal")
    };
    assert_eq!(
        fate.cause(),
        if deadline_stop {
            PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal
        } else {
            PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal
        }
    );
    assert!(records.read(name(), 1024).unwrap().is_none());
    assert!(matches!(
        recovery.resume(),
        AnchoredRecordPublicationResume::Stopped(_)
    ));
}
#[test]
fn duplicate_admission_cannot_replace_the_original_request_controls() {
    let world = world("owned-duplicate-controls");
    let records = world.serving().anchored_records().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let original = CancellationSource::new();
    let duplicate = CancellationSource::new();
    let port = Port::new(&memory, 0, Delivery::Normal);
    let (operation, _recovery) = start(&world, 0, &memory, &original, None);
    let gate = pause(world.serving(), PhysicalMutationCheckpoint::BeforeWalAppend);
    let operation = pending(operation, &records, &port);
    assert!(gate.0.await_arrival());
    assert!(matches!(
        records.start_publication(publication(&world, 0), view(&memory, &duplicate, None)),
        AnchoredRecordPublicationStart::Refused(AnchoredRecordOperationFailure::Busy)
    ));
    duplicate.cancel();
    gate.0.release();
    let receipt = match next(operation, &records, &port) {
        AnchoredRecordPublicationStep::Completed(receipt) => receipt,
        _ => panic!("duplicate observer replaced original controls"),
    };
    let quiet = world.serving().physical_mutation_observation();
    let retry = CancellationSource::new();
    let already =
        match records.start_publication(publication(&world, 0), view(&memory, &retry, None)) {
            AnchoredRecordPublicationStart::AlreadyPublished(receipt) => receipt,
            _ => panic!("actual terminal selection must answer without a worker"),
        };
    assert_eq!(already.membership(), receipt.membership());
    assert_eq!(world.serving().physical_mutation_observation(), quiet);
}
