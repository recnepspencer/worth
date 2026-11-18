//! Fixed journal interleaving: terminal DATA cannot release admission early.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

#[test]
fn a_second_original_waits_until_the_first_terminal_is_visible() {
    let directory = tempfile::tempdir().unwrap();
    let opened = crate::open::physical_admission::open(
        directory.path(),
        crate::open::physical_admission::PhysicalOpening::production(),
    )
    .unwrap();
    let serving = Arc::new(opened.serving);
    let gate = serving.pause_physical_mutation_at(
        worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeJournalTerminalVisibility,
    );
    let serial = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            let first = journal(&serving, "query.corrections").unwrap();
            first
                .publish(OrderedJournalPublication::new(
                    1,
                    b"first-original".to_vec(),
                    crate::open::physical_admission::placement(),
                    PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
                ))
                .unwrap()
        });
        let arrived = gate.await_arrival();
        if !arrived {
            gate.release();
            first.join().unwrap();
            panic!("first original must reach its terminal visibility seam");
        }
        let second = journal(&serving, "query.obligations").unwrap();
        let view = request
            .run_native_with_checkpoint(|context| context.retain_interruption_view(None))
            .unwrap()
            .0
            .unwrap();
        let admission = second.prepare_publication(
            OrderedJournalPublication::new(
                1,
                b"second-original".to_vec(),
                crate::open::physical_admission::placement(),
                PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
            ),
            view,
        );
        let progress = admission.advance(&second);
        // Release even on a failed assertion, so the actual worker cannot hang.
        gate.release();
        first.join().unwrap();
        let OrderedJournalPublicationAdmissionProgress::Pending(original) = progress else {
            panic!("a retained but invisible terminal must still own journal admission");
        };
        let OrderedJournalPublicationAdmissionProgress::Started(
            OrderedJournalPublicationStart::Ready {
                operation,
                recovery,
            },
        ) = original.advance(&second)
        else {
            panic!("the same waiting original must be admitted after visibility");
        };
        assert!(operation.observe().mutation_identity().is_none());
        drop(operation.stop());
        drop(recovery);
    });
}
