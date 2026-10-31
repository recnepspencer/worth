//! Actual journal task outlives the original disposable request result.
use std::time::{Duration, Instant};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_store::physical_runtime::production::PhysicalMutationCheckpoint;
use worth_store::physical_runtime::*;
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;
const BYTES: &[u8] = b"opaque owner artifact: actual ordered publication";
#[test]
fn original_request_loss_after_root_replacement_retains_the_same_journal_entry() {
    let world =
        PhysicalResidencyStoreWorld::initialize_for_recovery("owned-ordered-journal").unwrap();
    let journal = world.serving().canonical_commits().unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    let request = SerialRequest::from_memory(memory.clone(), cancellation.token(), None);
    let gate = world
        .serving()
        .pause_physical_mutation_at(PhysicalMutationCheckpoint::AfterRootReplacement);
    let mut operation = None;
    let mut recovery = None;
    ExecutionRequest::serial(&request)
        .run_native_with_checkpoint(|context| {
            let input = memory.reserve(BYTES.len() as u64).unwrap();
            let publication = OrderedJournalPublication::new(
                1,
                BYTES.to_vec(),
                world.placement(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            )
            .with_input_memory(input);
            let view = context.retain_interruption_view(None).unwrap();
            let OrderedJournalPublicationStart::Ready {
                operation: actual,
                recovery: retained,
            } = journal.start_publication(publication, view)
            else {
                panic!("actual original journal admission");
            };
            recovery = Some(retained); // This custody is OUTSIDE disposable R before step.
            operation = Some(actual);
            let actual = operation.take().unwrap();
            match actual.step(&journal) {
                OrderedJournalPublicationStep::Pending(same) => operation = Some(same),
                _ => panic!("held actual step must be pending"),
            }
            let source = operation.as_ref().unwrap().observe().source();
            drop(operation.take());
            let OrderedJournalPublicationResume::Ready {
                operation: resumed,
                recovery: retained,
            } = recovery.take().unwrap().resume()
            else {
                panic!("same dropped endpoint must resume");
            };
            assert_eq!(resumed.observe().source(), source);
            recovery = Some(retained); // Reinstall BEFORE the resumed endpoint is disposable.
            drop(resumed); // Observer loss after resume leaves the SAME strong task outside R.
        })
        .unwrap();
    drop(request);
    assert!(gate.await_arrival());
    let recovery = recovery.unwrap();
    let identity = recovery.observe().mutation_identity().unwrap();
    assert!(memory.charged_memory_bytes() > 0);
    cancellation.cancel();
    let OrderedJournalPublicationResume::Stopped {
        operation: stopped,
        recovery,
    } = recovery.stop().resume()
    else {
        panic!("stopped recovery must never regain advancement");
    };
    assert!(
        stopped.observe().terminal().is_none(),
        "root replacement gate precedes settled completion"
    );
    gate.release();
    let visibility_deadline = Instant::now() + Duration::from_secs(5);
    while recovery.observe().terminal().is_none() {
        assert!(
            Instant::now() < visibility_deadline,
            "actual journal terminal was not delivered"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        memory.charged_memory_bytes(),
        0,
        "visible terminal preceded original control release"
    );
    // Another admission can replace the weak slot after actual settlement;
    // the earlier stopped endpoint still owns its original actual terminal.
    journal
        .publish(OrderedJournalPublication::new(
            2,
            b"next".to_vec(),
            world.placement(),
            PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
        ))
        .unwrap();
    let observed = settle(stopped, &journal);
    let terminal = observed.terminal().unwrap();
    assert_eq!(terminal.mutation_identity(), Some(identity));
    assert!(matches!(
        terminal.physical_outcome(),
        Some(PhysicalMutationOutcome::Completed(_))
    ));
    assert_eq!(terminal.failure(), None);
    let actual = terminal.published().unwrap().entry();
    assert_eq!(actual.position(), 1);
    assert_eq!(journal.read(1, 1024).unwrap().bytes(), BYTES);
    assert_eq!(
        memory.charged_memory_bytes(),
        0,
        "descriptive recovery/terminal observers pinned original backing"
    );
    assert_eq!(
        recovery
            .observe()
            .terminal()
            .unwrap()
            .published()
            .unwrap()
            .entry(),
        actual
    );
    let OrderedJournalPublicationResume::Committed(sealed) = recovery.resume() else {
        panic!("same actual task must readmit owner-sealed committed fact after R loss");
    };
    assert_eq!(sealed.entry(), actual);
    assert_eq!(sealed.source(), observed.source());
    assert_eq!(sealed.terminal().mutation_identity(), Some(identity));
    drop(journal);
    let root = world.retained_root();
    let _closed = world.close();
    let cold=worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve(root.path(),"owned journal cold read");
    let read = cold.canonical_commits().unwrap().read(1, 1024).unwrap();
    assert_eq!(read.entry(), actual);
    assert_eq!(read.bytes(), BYTES);
    assert_eq!(
        cold.canonical_commits()
            .unwrap()
            .read(2, 1024)
            .unwrap()
            .bytes(),
        b"next"
    );
    cold.close();
}
fn settle(
    mut stopped: OrderedJournalStoppedPublication,
    journal: &OrderedJournal<'_>,
) -> OrderedJournalPublicationObservation {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        match stopped.settle(journal) {
            OrderedJournalStoppedPoll::Pending(same) => stopped = same,
            OrderedJournalStoppedPoll::Settled(actual) => return actual,
            OrderedJournalStoppedPoll::Failed(failure) => panic!(
                "actual accepted journal settlement failed: {:?}",
                failure.cause()
            ),
        }
        assert!(
            Instant::now() < deadline,
            "actual accepted journal step did not settle"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

mod admission;
mod cutover;
mod loss;
