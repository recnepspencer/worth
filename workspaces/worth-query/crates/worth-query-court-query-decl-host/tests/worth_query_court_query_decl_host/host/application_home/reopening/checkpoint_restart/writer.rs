//! Park only after observing actual selected image/tail or interrupted ACK facts.
use super::*;
use worth_query_host::facade::WorthQueryApplicationHome;
type Home = WorthQueryApplicationHome<fixture::schema::HomeIdentitySchema, fixture::HomeProgram>;

pub(super) fn run(case: Case, path: std::path::PathBuf) -> ! {
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = Arc::new(AtomicUsize::new(0));
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let limit = if matches!(case, Case::BoundedTail) {
        8
    } else {
        4096
    };
    let request = fixture::request(path.clone(), build, initialized.clone(), contacts.clone())
        .with_checkpoint_limits(WorthQueryApplicationHomeCheckpointLimits::new(
            std::num::NonZeroU64::new(limit).unwrap(),
            std::num::NonZeroU64::new(64 << 20).unwrap(),
        ));
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| opening_failed("restart writer", &refused));
    let (_, live) = scope();
    let mut value = 8;
    assert!(matches!(
        set(home.runtime(), &live, 0x435000, value).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let generation = match case {
        Case::JournalOnly => 0,
        Case::InterruptedAck => {
            let original = interrupt(&home, &execution);
            let resumed = home.recover_checkpoint(&execution).unwrap();
            assert_eq!(resumed.generation(), original);
            assert!(home
                .inspect_checkpoint(&execution)
                .unwrap()
                .pending
                .is_none());
            original
        }
        Case::Image | Case::ImageAndTail | Case::BoundedTail => {
            home.checkpoint(&execution).unwrap().generation()
        }
    };
    let first_covered = home.inspect_checkpoint(&execution).unwrap().journals[0].covered;
    let additional = match case {
        Case::Image | Case::InterruptedAck => 0,
        Case::JournalOnly => 2,
        Case::ImageAndTail => 3,
        Case::BoundedTail => 37,
    };
    for offset in 0..additional {
        // Each mutation is a distinct authenticated request. Checkpoint work
        // from earlier requests must not consume a later request's deadline.
        let (_, live) = scope();
        value += 1;
        assert!(matches!(
            set(home.runtime(), &live, 0x435100 + offset, value).unwrap(),
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
    }
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    assert_eq!(contacts.load(Ordering::SeqCst), 1 + additional as usize);
    let (_, live) = scope();
    assert_eq!(balance(home.runtime(), &live), value);
    // An ordinary Query read resumes pending Home maintenance. Park the second
    // interruption after that read so the original unacked attempt reaches kill.
    let generation = if matches!(case, Case::InterruptedAck) {
        let next = interrupt(&home, &execution);
        assert_eq!(next, generation + 1);
        next
    } else {
        generation
    };
    let observed = home.inspect_checkpoint(&execution).unwrap();
    let tail = observed
        .journals
        .iter()
        .map(|journal| journal.head - journal.covered)
        .max()
        .unwrap();
    match case {
        Case::Image => assert!(observed.pending.is_none()),
        Case::JournalOnly => {
            assert!(observed.pending.is_none());
            assert_eq!(observed.journals[0].covered, 0);
            assert!(observed.journals[0].head > 0);
        }
        Case::ImageAndTail => assert!(observed.journals[0].head > observed.journals[0].covered),
        Case::InterruptedAck => {
            let pending = observed.pending.as_ref().unwrap();
            assert_eq!(pending.selected_generation, Some(generation));
            assert!(!pending.acknowledgement_complete);
        }
        Case::BoundedTail => {
            assert!(
                !observed.due && observed.pending.is_none(),
                "automatic maintenance: {observed:?}",
            );
            assert!(
                observed.journals[0].covered > first_covered,
                "automatic maintenance must select a later image"
            );
            assert!(tail <= limit);
        }
    }
    // Interrupted ACK has no new acknowledged scheduling seed; the selected
    // image nevertheless covers the actual current Native head.
    let covered = match case {
        Case::InterruptedAck => observed.journals[0].head,
        Case::Image | Case::JournalOnly | Case::ImageAndTail | Case::BoundedTail => {
            observed.journals[0].covered
        }
    };
    let ready = path.with_extension("complete-home-ready");
    let pending = ready.with_extension("pending");
    std::fs::write(
        &pending,
        format!(
            "{generation} {covered} {} {value} {tail}",
            observed.journals[0].head
        ),
    )
    .unwrap();
    std::fs::rename(pending, ready).unwrap();
    loop {
        std::hint::black_box(&home);
        std::thread::sleep(Duration::from_secs(1));
    }
}

fn interrupt(home: &Home, original: &worth_execution::ExecutionRequest<'_, '_>) -> u64 {
    let cancellation = worth_execution::CancellationSource::new();
    let stopping = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = calls.clone();
    assert!(home
        .checkpoint_with_native_ack_observer_for_test(
            &worth_execution::ExecutionRequest::serial(&stopping),
            Box::new(move || {
                observed.fetch_add(1, Ordering::SeqCst);
                cancellation.cancel();
            })
        )
        .is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let pending = home
        .inspect_checkpoint(original)
        .unwrap()
        .pending
        .expect("the actual selected attempt stays owned");
    assert!(!pending.acknowledgement_complete);
    pending
        .selected_generation
        .expect("physical selection must precede interruption")
}
