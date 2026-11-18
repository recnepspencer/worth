//! Fixed interleaving: one original holds journal admission while another arrives.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[test]
fn managed_journal_admission_retains_the_second_original_until_the_first_finishes() {
    let directory = tempfile::tempdir().unwrap();
    let opened = crate::open::physical_admission::open(
        directory.path(),
        crate::open::physical_admission::PhysicalOpening::production(),
    )
    .unwrap();
    let serving = Arc::new(opened.serving);
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    let first = journal(&serving, "query.corrections").unwrap();
    let view = request
        .run_native_with_checkpoint(|context| context.retain_interruption_view(None))
        .unwrap()
        .0
        .unwrap();
    let publication = OrderedJournalPublication::new(
        1,
        b"first-original".to_vec(),
        crate::open::physical_admission::placement(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    );
    let OrderedJournalPublicationStart::Ready {
        mut operation,
        recovery,
    } = first.start_publication(publication, view)
    else {
        panic!("first admission");
    };
    // Caller A owns an admitted original but has not performed its first step.
    let second = ApplicationQueryJournal::new(&serving, "query.obligations");
    assert!(
        matches!(
            second.advance(&request, 1, b"second-original"),
            QueryJournalPublicationOutcome::Pending
        ),
        "contention must retain the second original, never manufacture no-effect"
    );
    loop {
        request.consult().unwrap();
        match operation.step(&first) {
            OrderedJournalPublicationStep::Pending(same) => operation = same,
            OrderedJournalPublicationStep::Completed(_) => break,
            OrderedJournalPublicationStep::Failed(_) => panic!("first original failed"),
        }
    }
    drop(recovery);
    loop {
        request.consult().unwrap();
        match second.advance(&request, 1, b"second-original") {
            QueryJournalPublicationOutcome::Pending => std::thread::yield_now(),
            QueryJournalPublicationOutcome::Committed { position: 1, .. } => break,
            _ => panic!("the retained second original must commit after admission releases"),
        }
    }
    assert_eq!(
        second.read_frame(&request, 1, 1024).unwrap().unwrap(),
        b"second-original"
    );
}

#[test]
fn pending_journal_admission_keeps_original_cancellation_under_a_live_receiver() {
    let directory = tempfile::tempdir().unwrap();
    let opened = crate::open::physical_admission::open(
        directory.path(),
        crate::open::physical_admission::PhysicalOpening::production(),
    )
    .unwrap();
    let serving = Arc::new(opened.serving);
    let live = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&live);
    let first = journal(&serving, "query.corrections").unwrap();
    let view = request
        .run_native_with_checkpoint(|context| context.retain_interruption_view(None))
        .unwrap()
        .0
        .unwrap();
    let publication = OrderedJournalPublication::new(
        1,
        b"first-original".to_vec(),
        crate::open::physical_admission::placement(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    );
    let OrderedJournalPublicationStart::Ready {
        operation,
        recovery,
    } = first.start_publication(publication, view)
    else {
        panic!("first admission");
    };
    let cancellation = CancellationSource::new();
    let original =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let second = ApplicationQueryJournal::new(&serving, "query.obligations");
    assert!(matches!(
        second.advance(&ExecutionRequest::serial(&original), 1, b"stopped-original"),
        QueryJournalPublicationOutcome::Pending
    ));
    cancellation.cancel();
    // A new live receiver must settle cancellation of B's retained original,
    // even while A still owns admission; it must never publish B's payload.
    assert!(matches!(
        second.advance(&request, 1, b"stopped-original"),
        QueryJournalPublicationOutcome::ProvenNoEffect { position: 1, .. }
    ));
    assert!(second.read_frame(&request, 1, 1024).unwrap().is_none());
    drop(operation);
    drop(recovery);
}
