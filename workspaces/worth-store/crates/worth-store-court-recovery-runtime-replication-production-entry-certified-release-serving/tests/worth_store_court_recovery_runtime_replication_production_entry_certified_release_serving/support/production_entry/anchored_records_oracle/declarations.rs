//! Crash-consistency oracle for one anchored-record publication.
//!
//! For the first value of an anchor and for a replacement, the oracle counts
//! the media operations of every role the publication performs, then reruns it
//! once per (role, ordinal, directive) with a fault at exactly that operation,
//! abandons the runtime, recovers in the production path and requires: the
//! anchor holds exactly the old or the new value (an acknowledged value is
//! never lost, a proven-no-effect one never appears); an indeterminate
//! publication's name refuses as unresolved until reopen; the journal beside it is
//! unchanged; the new value then publishes; and a second recovery agrees.

pub(crate) use std::num::NonZeroU64;

pub(crate) use {
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaOperationRole,
};
pub(crate) use {
    worth_store::physical_runtime::AnchoredRecordPublicationFailure,
    worth_store::physical_runtime::AnchoredRecordPublished,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::anchored_records::assert_journal, worth_store_testkit_recovery_runtime::production_entry::anchored_records::current, worth_store_testkit_recovery_runtime::production_entry::anchored_records::name, worth_store_testkit_recovery_runtime::production_entry::anchored_records::publish, worth_store_testkit_recovery_runtime::production_entry::anchored_records::publish_journal_entry, worth_store_testkit_recovery_runtime::production_entry::anchored_records::value, worth_store_testkit_recovery_runtime::production_entry::anchored_records::CHECKPOINT};
pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::open_with_faults, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};
pub(crate) use {worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld, worth_store_testkit_recovery_runtime::production_entry::anchored_records};

pub(crate) const JOURNAL_HEAD: u64 = 2;

/// The value `generation` of the swept anchor.
pub(crate) fn generation_value(generation: u64) -> Vec<u8> {
    value(0x10 + generation as u8, 3_000 + generation as usize * 4_500)
}

/// What the anchor must hold before the target publication.
pub(crate) fn before(target: u64) -> Option<(u64, Vec<u8>)> {
    (target > 1).then(|| (target - 1, generation_value(target - 1)))
}

pub(crate) fn publish_target(
    serving: &ServingPhysicalRuntime,
    target: u64,
) -> Result<AnchoredRecordPublished, AnchoredRecordPublicationFailure> {
    publish(
        serving,
        placement(),
        name(CHECKPOINT),
        target - 1,
        generation_value(target),
    )
}

/// A cleanly closed store with journal entries and generations `1..target`.
pub(crate) fn prepared_store(target: u64) -> worth_store_test_support::TemporaryDirectory {
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("anchored-oracle")
        .expect("oracle store");
    for position in 1..=JOURNAL_HEAD {
        publish_journal_entry(world.serving(), world.placement(), position);
    }
    for generation in 1..target {
        publish_target(world.serving(), generation).expect("prior generation");
    }
    let root = world.retained_root();
    let _ = world.close();
    root
}

/// Counts each role's identified operations while `target` publishes.
pub(crate) fn measure(target: u64) -> Vec<(MediaOperationRole, u64)> {
    let store = prepared_store(target);
    let never = worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
        kind: std::io::ErrorKind::Other,
        raw_os_error: None,
    };
    let faults: Vec<_> = worth_store::physical_runtime::certification::MediaOperationRole::ALL
        .iter()
        .map(|role| (*role, std::num::NonZeroU64::MAX, never.clone()))
        .collect();
    let (serving, activations) =
        open_with_faults(store.path(), &faults).expect("fail-before is admissible for every role");
    for activation in &activations {
        activation.arm().unwrap();
    }
    publish_target(&serving, target).expect("the measured publication completes");
    let counts = worth_store::physical_runtime::certification::MediaOperationRole::ALL
        .iter()
        .zip(&activations)
        .map(|(role, activation)| (*role, activation.matching_operation_count()))
        .filter(|(_, count)| *count > 0)
        .collect();
    let _ = serving.close();
    counts
}

/// One faulted publication of `target`, then production recovery.
pub(crate) fn run_case(
    target: u64,
    role: MediaOperationRole,
    nth: NonZeroU64,
    directive: MediaFaultDirective,
) -> bool {
    let store = prepared_store(target);
    let Some((serving, activations)) =
        open_with_faults(store.path(), &[(role, nth, directive.clone())])
    else {
        return false;
    };
    activations[0].arm().unwrap();
    let outcome = publish_target(&serving, target);
    assert!(activations[0].is_consumed(), "{role:?}#{nth} did not fire");
    let case = format!("target {target} {role:?}#{nth} {directive:?}");
    let refused_unresolved = indeterminate_name_refuses_until_reopen(&serving, &outcome, &case);
    let _ = serving.abort();
    let recovered = serve(store.path(), &case);
    let after = Some((target, generation_value(target)));
    let held = current(&recovered, name(CHECKPOINT));
    match &outcome {
        Ok(_) => assert!(held == after, "{case}: acknowledged value lost"),
        Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::ProvenNoEffect(_)) => {
            assert!(held == before(target), "{case}: a no-effect value appeared")
        }
        Err(_) => assert!(
            held == before(target) || held == after,
            "{case}: neither the old nor the new value"
        ),
    }
    assert_journal(&recovered, JOURNAL_HEAD);
    let republished = publish_target(&recovered, target)
        .unwrap_or_else(|failure| panic!("{case}: republication: {failure:?}"));
    assert_eq!(
        matches!(
            republished,
            worth_store::physical_runtime::AnchoredRecordPublished::AlreadyPublished(_)
        ),
        held == after,
        "{case}: republication answer"
    );
    assert!(
        current(&recovered, name(CHECKPOINT)) == after,
        "{case}: republished value"
    );
    let _ = recovered.close();
    let again = serve(store.path(), &format!("{case} second recovery"));
    assert!(
        current(&again, name(CHECKPOINT)) == after,
        "{case}: second recovery"
    );
    assert_journal(&again, JOURNAL_HEAD);
    let _ = again.close();
    refused_unresolved
}

/// After a publication whose fate may be durable, its name refuses with
/// `Unresolved` for as long as this incarnation still serves. Returns whether
/// that refusal was observed.
pub(crate) fn indeterminate_name_refuses_until_reopen(
    serving: &ServingPhysicalRuntime,
    outcome: &Result<AnchoredRecordPublished, AnchoredRecordPublicationFailure>,
    case: &str,
) -> bool {
    let fate_unresolved = matches!(
        outcome,
        Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::Indeterminate(_)
            | worth_store::physical_runtime::AnchoredRecordPublicationFailure::CompletedWithoutRecord)
    );
    if !fate_unresolved || serving.anchored_records().is_err() {
        return false;
    }
    // Any request to the name refuses; the unresolved fate is checked first.
    let retry = publish(serving, placement(), name(CHECKPOINT), 0, value(0xee, 10));
    assert!(
        matches!(
            retry,
            Err(worth_store::physical_runtime::AnchoredRecordPublicationFailure::Unresolved)
        ),
        "{case}: an unresolved name admitted {retry:?}"
    );
    true
}
