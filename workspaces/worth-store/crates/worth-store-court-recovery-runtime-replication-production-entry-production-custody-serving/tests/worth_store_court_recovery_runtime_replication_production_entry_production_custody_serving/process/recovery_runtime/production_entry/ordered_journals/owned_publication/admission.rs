use super::cutover::{start, view};
use super::*;
#[test]
fn duplicate_controls_and_foreign_receiver_do_not_replace_the_actual_journal_step() {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("journal-admission").unwrap();
    let journal = world.serving().canonical_commits().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let original = CancellationSource::new();
    let (operation, recovery) = start(&world, &memory, &original, None, BYTES);
    let gate = world
        .serving()
        .pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeWalAppend);
    let OrderedJournalPublicationStep::Pending(operation) = operation.step(&journal) else {
        panic!("held original step");
    };
    assert!(gate.await_arrival());
    let identity = operation.observe().mutation_identity().unwrap();
    let other = CancellationSource::new();
    let duplicate = journal.start_publication(
        OrderedJournalPublication::new(
            1,
            BYTES.to_vec(),
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ),
        view(&memory, &other),
    );
    assert!(matches!(
        duplicate,
        OrderedJournalPublicationStart::Refused(OrderedJournalOperationFailure::Busy)
    ));
    other.cancel();
    let foreign = world.serving().branch_transitions().unwrap();
    let OrderedJournalPublicationStep::Failed(failure) = operation.step(&foreign) else {
        panic!("foreign receiver must refuse before advancement");
    };
    assert!(matches!(
        failure.cause(),
        OrderedJournalOperationFailure::ForeignReceiver
    ));
    let stopped = failure.into_stopped();
    let recovery = match recovery.stop().resume() {
        OrderedJournalPublicationResume::Busy(same) => same,
        _ => panic!("live stopped endpoint cannot be duplicated"),
    };
    gate.release();
    let actual = settle(stopped, &journal);
    assert_eq!(
        actual.terminal().unwrap().mutation_identity(),
        Some(identity)
    );
    assert!(actual.terminal().unwrap().published().is_some());
    assert!(foreign.head().unwrap().is_none());
    assert_eq!(memory.charged_memory_bytes(), 0);
    let before = world.serving().physical_mutation_observation();
    let retry = journal.start_publication(
        OrderedJournalPublication::new(
            1,
            BYTES.to_vec(),
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ),
        view(&memory, &original),
    );
    let OrderedJournalPublicationStart::AlreadyCommitted(done) = retry else {
        panic!("actual durable duplicate must be sealed completion");
    };
    assert_eq!(
        done.entry(),
        actual.terminal().unwrap().published().unwrap().entry()
    );
    assert_eq!(done.source().journal(), journal.name());
    assert_eq!(world.serving().physical_mutation_observation(), before);
    assert!(matches!(
        journal.publish(OrderedJournalPublication::new(
            1,
            b"conflict".to_vec(),
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap()
        )),
        Err(OrderedJournalPublicationFailure::PositionConflict(_))
    ));
    assert!(matches!(
        journal.publish(OrderedJournalPublication::new(
            3,
            b"gap".to_vec(),
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap()
        )),
        Err(OrderedJournalPublicationFailure::PositionGap {
            head: 1,
            requested: 3
        })
    ));
    journal
        .publish(OrderedJournalPublication::new(
            2,
            b"second".to_vec(),
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
        .unwrap();
    assert_eq!(
        journal
            .range_descending(2, 10)
            .unwrap()
            .iter()
            .map(|entry| entry.position())
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert_eq!(journal.read(1, 1024).unwrap().bytes(), BYTES);
    assert_eq!(journal.read(2, 1024).unwrap().bytes(), b"second");
    drop(recovery);
}

#[test]
fn stopped_before_submission_never_lends_another_endpoint_or_fabricates_a_terminal() {
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("journal-stopped-before-submit")
            .unwrap();
    let journal = world.serving().canonical_commits().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let (operation, recovery) = start(&world, &memory, &cancellation, None, BYTES);
    let recovery = match recovery.stop().resume() {
        OrderedJournalPublicationResume::Busy(same) => same,
        _ => panic!("original live endpoint was duplicated"),
    };
    let OrderedJournalPublicationStep::Failed(failure) = operation.step(&journal) else {
        panic!("stopped endpoint advanced");
    };
    assert!(matches!(
        failure.cause(),
        OrderedJournalOperationFailure::Stopped
    ));
    assert!(failure.observe().mutation_identity().is_none());
    assert!(failure.observe().terminal().is_none());
    let stopped = failure.into_stopped();
    let actual = settle(stopped, &journal);
    assert!(actual.terminal().is_none());
    assert!(actual.stopped());
    assert_eq!(memory.charged_memory_bytes(), 0);
    assert_eq!(world.serving().physical_mutation_observation().started(), 0);
    assert!(journal.head().unwrap().is_none());
    let OrderedJournalPublicationResume::Stopped {
        operation,
        recovery,
    } = recovery.resume()
    else {
        panic!("stop regained advancing authority");
    };
    drop(operation);
    drop(recovery);
}
