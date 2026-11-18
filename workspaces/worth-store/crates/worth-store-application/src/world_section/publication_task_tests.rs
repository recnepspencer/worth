//! An original World publication cannot be displaced before a proven fate.
use super::{Denial, Endpoint, TaskState};

#[test]
fn unresolved_original_without_physical_handles_refuses_replacement() {
    let state = TaskState {
        endpoint: Some(Endpoint::Unresolved),
        ..TaskState::default()
    };
    assert!(matches!(
        state.replacement_refusal(false),
        Some(Denial::PublicationUnresolved)
    ));
}

#[test]
fn initial_and_driving_originals_remain_owned_until_proven_no_effect() {
    let mut state = TaskState::default();
    assert!(matches!(
        state.replacement_refusal(false),
        Some(Denial::PublicationBusy)
    ));
    state.endpoint = Some(Endpoint::NoEffect);
    assert!(matches!(
        state.replacement_refusal(true),
        Some(Denial::PublicationBusy)
    ));
    assert!(state.replacement_refusal(false).is_none());
    state.invalid = true;
    assert!(matches!(
        state.replacement_refusal(false),
        Some(Denial::PublicationUnresolved)
    ));
}

#[test]
fn submitted_cancelled_publication_releases_world_replacement_guard() {
    use worth_execution::{
        CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest,
    };
    use worth_store::physical_runtime::*;

    let directory = tempfile::tempdir().unwrap();
    let opened = crate::open::physical_admission::open(
        directory.path(),
        crate::open::physical_admission::PhysicalOpening::production(),
    )
    .unwrap();
    let serving = opened.serving;
    let gate = serving.pause_physical_mutation_at(
        worth_store::physical_runtime::production::PhysicalMutationCheckpoint::BeforeEffectCutover,
    );
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&original);
    let view = request
        .run_native_with_checkpoint(|context| context.retain_interruption_view(None))
        .unwrap()
        .0
        .unwrap();
    let journal = serving
        .ordered_journal(OrderedJournalName::new("world.branches").unwrap())
        .unwrap();
    let publication = OrderedJournalPublication::new(
        1,
        b"cancelled-world-original".to_vec(),
        crate::open::physical_admission::placement(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    );
    let OrderedJournalPublicationStart::Ready {
        operation,
        recovery,
    } = journal.start_publication(publication, view)
    else {
        panic!("original must be admitted");
    };
    let step = operation.step(&journal);
    let arrived = gate.await_arrival();
    let OrderedJournalPublicationStep::Pending(operation) = step else {
        gate.release();
        panic!("submitted original must remain pending at the effect gate");
    };
    let mut stopped = operation.stop();
    assert!(stopped.observe().mutation_identity().is_some());
    stopped.request_cancellation();
    gate.release();
    assert!(arrived, "worker must reach the fixed pre-effect seam");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let observation = loop {
        assert!(
            std::time::Instant::now() < deadline,
            "cancelled original must terminate"
        );
        request.consult().unwrap();
        match stopped.settle(&journal) {
            OrderedJournalStoppedPoll::Pending(same) => {
                stopped = same;
                std::thread::yield_now();
            }
            OrderedJournalStoppedPoll::Settled(observation) => break observation,
            OrderedJournalStoppedPoll::Failed(failure) => {
                let observed = failure.observe();
                assert!(
                    observed.terminal().is_some(),
                    "cancelled original must settle: {failure:?}"
                );
                break observed;
            }
        }
    };
    let terminal = observation
        .terminal()
        .expect("actual submitted terminal")
        .clone();
    assert!(matches!(
        terminal.physical_outcome(),
        Some(PhysicalMutationOutcome::ProvenNoEffect(_))
    ));
    let mut state = TaskState {
        endpoint: Some(Endpoint::Terminal(terminal.clone())),
        ..TaskState::default()
    };
    assert!(matches!(
        state.replacement_refusal(false),
        Some(Denial::PublicationBusy)
    ));
    let result = Some(super::super::publication_fate::terminal(&terminal));
    state.endpoint = state
        .endpoint
        .take()
        .map(|endpoint| endpoint.after_observation(&result));
    assert!(state.replacement_refusal(false).is_none());
    drop(recovery);
}
