//! Refused or unwound candidates release their named reservation without moving the head.
use super::{medium_fixture as media, owner_fixture as fixture};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_foundational::facade::DurablePublicationFate;
use worth_signal::facade::durability::*;

#[test]
fn candidate_unwind_and_proven_absence_preserve_the_head_and_release_the_original_reservation() {
    let directory = tempfile::tempdir().unwrap();
    let funding = media::serial_request();
    let execution = ExecutionRequest::serial(&funding);
    let (serving, medium) = media::open(directory.path(), true, &execution);
    let reference = fixture::seed(medium.as_ref(), &execution);
    let owner = fixture::readmit(medium.clone(), &execution);
    let (&branch, descriptor) = reference.heads.first_key_value().unwrap();
    let basis = owner.source_port().readmit_descriptor(descriptor).unwrap();
    let unwound = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = owner
            .mutation_port()
            .begin_advance(execution, &basis, &mut (), |_| {
                panic!("caller unwind while only the isolated candidate is mutable")
            });
    }));
    assert!(unwound.is_err());
    assert_eq!(&owner.source_port().observe(branch).unwrap(), descriptor);
    let task = owner
        .mutation_port()
        .begin_advance(execution, &basis, &mut (), |_| Ok(()))
        .unwrap();
    let cancellation = CancellationSource::new();
    let stopped =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    cancellation.cancel();
    let mut task = task;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    loop {
        match task
            .drive(&ExecutionRequest::serial(&stopped))
            .unwrap_or_else(|refusal| panic!("{:?}", refusal.denial()))
        {
            SignalDurableAdvanceProgress::Pending(next) => {
                assert!(std::time::Instant::now() < deadline);
                task = next;
                std::thread::yield_now();
            }
            SignalDurableAdvanceProgress::Finished(done) => {
                assert!(matches!(
                    done.fate(),
                    DurablePublicationFate::ProvenNoEffect {
                        detail: SignalDurableAdvanceDenial::Medium(
                            SignalDurableMediumDenial::Request(_)
                        )
                    }
                ));
                break;
            }
        }
    }
    assert_eq!(&owner.source_port().observe(branch).unwrap(), descriptor);
    drop(fixture::finish(
        owner
            .mutation_port()
            .begin_advance(execution, &basis, &mut (), |_| Ok(()))
            .unwrap(),
        &execution,
    ));
    drop((basis, owner));
    media::close(serving, medium, &execution);
}
