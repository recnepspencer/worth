use super::cutover::start;
use super::*;
#[test]
fn last_journal_controller_loss_keeps_actual_work_and_backing_until_terminal() {
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("journal-controller-loss").unwrap();
    let journal = world.serving().canonical_commits().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let (operation, recovery) = start(&world, &memory, &cancellation, None, BYTES);
    let gate = world
        .serving()
        .pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeEffectCutover);
    let OrderedJournalPublicationStep::Pending(operation) = operation.step(&journal) else {
        panic!("held actual worker");
    };
    assert!(gate.await_arrival());
    drop(operation);
    drop(recovery);
    assert!(memory.charged_memory_bytes() >= BYTES.len() as u64);
    assert!(matches!(
        journal.publish(OrderedJournalPublication::new(
            1,
            BYTES.to_vec(),
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap()
        )),
        Err(OrderedJournalPublicationFailure::OwnedStart(_))
    ));
    gate.release();
    let deadline = Instant::now() + Duration::from_secs(5);
    while memory.charged_memory_bytes() != 0 {
        assert!(
            Instant::now() < deadline,
            "lost observers pinned accepted backing"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(journal.read(1, 1024).unwrap().bytes(), BYTES);
    assert_eq!(world.serving().physical_mutation_observation().started(), 1);
}
#[test]
fn caller_unwind_after_resume_keeps_external_recovery_and_permanently_stops_advancement() {
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("journal-resumed-unwind").unwrap();
    let journal = world.serving().canonical_commits().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let gate = world
        .serving()
        .pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeEffectCutover);
    let (operation, recovery) = start(&world, &memory, &cancellation, None, BYTES);
    let OrderedJournalPublicationStep::Pending(operation) = operation.step(&journal) else {
        panic!("held actual worker");
    };
    assert!(gate.await_arrival());
    drop(operation);
    let OrderedJournalPublicationResume::Ready {
        operation,
        recovery,
    } = recovery.resume()
    else {
        panic!("same endpoint resume");
    };
    let source = recovery.observe().source();
    let retained = recovery;
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _loaned = operation;
            panic!("original caller unwound");
        }))
        .is_err()
    );
    let OrderedJournalPublicationResume::Stopped {
        operation,
        recovery,
    } = retained.resume()
    else {
        panic!("unwind must never restore advancement");
    };
    assert_eq!(recovery.observe().source(), source);
    gate.release();
    let actual = settle(operation, &journal);
    assert!(actual.stopped());
    assert!(actual.terminal().unwrap().published().is_some());
    assert_eq!(actual.source(), source);
    assert_eq!(memory.charged_memory_bytes(), 0);
    drop(recovery);
}
