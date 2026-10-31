//! Original controls at the real WAL seal, without manufacturing a new grant.
use super::*;
#[test]
fn original_cancel_before_journal_wal_seal_preserves_noeffect_and_retry_identity() {
    preseal(false);
}
#[test]
fn original_absolute_deadline_before_journal_wal_seal_preserves_noeffect() {
    preseal(true);
}
pub(super) fn start(
    world: &PhysicalResidencyStoreWorld,
    memory: &SerialMemoryBudget,
    cancellation: &CancellationSource,
    deadline: Option<Instant>,
    bytes: &[u8],
) -> (
    OrderedJournalPublicationOperation,
    OrderedJournalPublicationRecovery,
) {
    let request = SerialRequest::from_memory(memory.clone(), cancellation.token(), deadline);
    let mut retained = None;
    ExecutionRequest::serial(&request)
        .run_native_with_checkpoint(|context| {
            let input = memory.reserve(bytes.len() as u64).unwrap();
            let publication = OrderedJournalPublication::new(
                1,
                bytes.to_vec(),
                world.placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_input_memory(input);
            retained = Some(
                world
                    .serving()
                    .canonical_commits()
                    .unwrap()
                    .start_publication(
                        publication,
                        context.retain_interruption_view(None).unwrap(),
                    ),
            );
        })
        .unwrap();
    let OrderedJournalPublicationStart::Ready {
        operation,
        recovery,
    } = retained.unwrap()
    else {
        panic!("actual journal admission");
    };
    (operation, recovery)
}
pub(super) fn view(
    memory: &SerialMemoryBudget,
    cancellation: &CancellationSource,
) -> worth_execution::ExecutionInterruptionView {
    let request = SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
    let mut retained = None;
    ExecutionRequest::serial(&request)
        .run_native_with_checkpoint(|context| {
            retained = Some(context.retain_interruption_view(None).unwrap())
        })
        .unwrap();
    retained.unwrap()
}
fn preseal(deadline_stop: bool) {
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("journal-preseal-control").unwrap();
    let journal = world.serving().canonical_commits().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let expires = deadline_stop.then(|| Instant::now() + Duration::from_secs(2));
    let (operation, recovery) = start(&world, &memory, &cancellation, expires, BYTES);
    let gate = world
        .serving()
        .pause_physical_mutation_at(PhysicalMutationCheckpoint::BeforeWalAppend);
    let OrderedJournalPublicationStep::Pending(operation) = operation.step(&journal) else {
        panic!("held pre-seal step");
    };
    assert!(gate.await_arrival());
    let identity = operation.observe().mutation_identity().unwrap();
    if let Some(expires) = expires {
        while Instant::now() < expires {
            std::thread::sleep(Duration::from_millis(1));
        }
    } else {
        cancellation.cancel();
    }
    let stopped = operation.stop();
    gate.release();
    let failure = failed(stopped, &journal);
    let observed = failure.observe();
    let terminal = observed.terminal().unwrap();
    assert_eq!(terminal.mutation_identity(), Some(identity));
    let Some(PhysicalMutationOutcome::ProvenNoEffect(actual)) = terminal.physical_outcome() else {
        panic!("actual original control did not establish NoEffect");
    };
    let expected = if deadline_stop {
        PhysicalMutationProvenNoEffectCause::DeadlineElapsedBeforeGroupSeal
    } else {
        PhysicalMutationProvenNoEffectCause::CancelledBeforeGroupSeal
    };
    assert_eq!(actual.cause(), expected);
    assert!(terminal.published().is_none());
    assert!(journal.head().unwrap().is_none());
    assert_eq!(memory.charged_memory_bytes(), 0);
    let before = world.serving().physical_mutation_observation();
    let retry = journal.publish(OrderedJournalPublication::new(
        1,
        BYTES.to_vec(),
        world.placement(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    ));
    let Err(OrderedJournalPublicationFailure::ProvenNoEffect(same)) = retry else {
        panic!("exact retry cannot erase original attempt fate");
    };
    assert_eq!(same.mutation_identity(), identity);
    assert_eq!(world.serving().physical_mutation_observation(), before);
    // A genuinely different admission at the still-empty position is allowed.
    let next = journal
        .publish(OrderedJournalPublication::new(
            1,
            b"fresh request".to_vec(),
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
        .unwrap();
    assert_eq!(next.entry().position(), 1);
    assert_eq!(journal.read(1, 1024).unwrap().bytes(), b"fresh request");
    drop(recovery);
}
pub(super) fn failed(
    mut stopped: OrderedJournalStoppedPublication,
    journal: &OrderedJournal<'_>,
) -> OrderedJournalPublicationFailureCustody {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match stopped.settle(journal) {
            OrderedJournalStoppedPoll::Pending(same) => stopped = same,
            OrderedJournalStoppedPoll::Failed(f) => return f,
            _ => panic!("expected actual owning failure"),
        };
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
