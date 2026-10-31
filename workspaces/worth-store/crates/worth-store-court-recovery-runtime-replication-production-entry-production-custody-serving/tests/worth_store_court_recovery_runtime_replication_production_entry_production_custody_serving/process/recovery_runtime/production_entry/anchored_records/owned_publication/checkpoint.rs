//! Original immutable controls on the actual fuzzy checkpoint publication owner.
use super::fixture::*;
use std::time::{Duration, Instant};
use worth_execution::{CancellationSource, SerialMemoryBudget};
use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::production::PhysicalCheckpointStep;
use worth_store::physical_runtime::*;

#[test]
fn fuzzy_checkpoint_original_cancel_after_candidate_sync_removes_the_real_candidate() {
    checkpoint_cutover(PhysicalCheckpointStep::CandidateSynchronization, false);
}
#[test]
fn fuzzy_checkpoint_original_cancel_after_publication_preserves_actual_completion() {
    checkpoint_cutover(PhysicalCheckpointStep::CandidatePublication, true);
}
fn checkpoint_cutover(seam: PhysicalCheckpointStep, published: bool) {
    let world = world(if published {
        "owned-cp-after-publication"
    } else {
        "owned-cp-before-publication"
    });
    let records = world.serving().anchored_records().unwrap();
    let selected = records.publish(publication(&world, 0)).unwrap();
    let memory = SerialMemoryBudget::new(None);
    let cancellation = CancellationSource::new();
    // The arm belongs to this actual Serving receiver; no reconstructed owner.
    let gate = world.serving().pause_physical_checkpoint_at(seam);
    let request = PhysicalCheckpointRequest::fuzzy(
        PhysicalCheckpointIdempotencyKey::new([0xc7; 32]),
        PhysicalCheckpointDeadline::after_milliseconds(30_000).unwrap(),
    );
    let TransitionOutcome::Success(handle) = world
        .serving()
        .checkpoints()
        .start_with_interruption_view(request, view(&memory, &cancellation, None))
        .into_raw()
    else {
        panic!("actual fuzzy checkpoint admission");
    };
    let identity = handle.identity();
    assert!(
        gate.await_arrival(),
        "actual checkpoint seam was not reached"
    );
    let candidates = || {
        std::fs::read_dir(world.root().join("staging"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|e| e == "candidate"))
            .collect::<Vec<_>>()
    };
    if !published {
        assert_eq!(
            candidates().len(),
            1,
            "real durable candidate before cleanup"
        );
    }
    cancellation.cancel();
    gate.release();
    let deadline = Instant::now() + Duration::from_secs(5);
    let outcome = loop {
        match handle.poll() {
            PhysicalCheckpointPoll::Terminal(outcome) => break outcome,
            PhysicalCheckpointPoll::Pending(_) => {
                assert!(
                    Instant::now() < deadline,
                    "actual checkpoint did not settle"
                );
                std::thread::sleep(Duration::from_millis(1));
            }
        }
    };
    if published {
        let PhysicalCheckpointOutcome::Completed(actual) = outcome else {
            panic!("late cancellation replaced actual completion: {outcome:?}");
        };
        assert_eq!(actual.basis().identity(), identity);
    } else {
        let PhysicalCheckpointOutcome::ProvenNoEffect(actual) = outcome else {
            panic!("original cancellation was not proven pre-publication: {outcome:?}");
        };
        assert_eq!(actual.identity(), identity);
        assert_eq!(
            actual.cause(),
            PhysicalCheckpointProvenNoEffectCause::CancelledAndCandidateRemoved
        );
    }
    assert!(
        candidates().is_empty(),
        "candidate residue remains after actual settlement"
    );
    let current = records.read(name(), 1024).unwrap().unwrap();
    assert_eq!(current.version(), selected.membership().version());
    assert_eq!(current.bytes(), PAYLOAD);
    assert_eq!(memory.charged_memory_bytes(), 0);
}
