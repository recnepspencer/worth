//! Crash-consistency oracle for one branch-transitions publication published
//! after canonical commits.
//!
//! For the journal's first entry and a later one, the oracle counts the media
//! operations of every role the publication performs, then reruns it once per
//! (role, ordinal, directive) with a fault at exactly that operation, abandons
//! the runtime, recovers in the production path and requires: the transitions
//! journal holds exactly the old head or the new one (an acknowledged entry is
//! never lost, a proven-no-effect one never appears); an indeterminate
//! publication blocks its journal until reopen; the canonical commits are
//! untouched; the same publication retried after recovery is answered from
//! durable state, never by the dead runtime's attempt; the suffix publishes;
//! and a second recovery agrees.

pub(crate) use std::num::NonZeroU64;
pub(crate) use std::path::Path;

pub(crate) use {
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaOperationRole,
};
pub(crate) use {
    worth_store::physical_runtime::OrderedJournalPublicationFailure,
    worth_store::physical_runtime::OrderedJournalPublished,
    worth_store::physical_runtime::OrderedJournalResolution,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};

pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::open_with_faults, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};
pub(crate) use {crate::production_entry::ordered_journals::two_journals::assert_holds, crate::production_entry::ordered_journals::two_journals::digest, crate::production_entry::ordered_journals::two_journals::publish, crate::production_entry::ordered_journals::two_journals::Journal};
pub(crate) use {worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld};

/// Canonical commits published before the swept transition.
pub(crate) const COMMITS: u64 = 3;
pub(crate) const TRANSITIONS: u64 = 4;
/// The journal's first entry (no anchor yet) and a later one with a spine.
pub(crate) const TARGETS: [u64; 2] = [1, 3];

pub(crate) type Outcome = Result<OrderedJournalPublished, OrderedJournalPublicationFailure>;

/// One measured transition publication and exactly one injected fault.
pub(crate) struct FaultCase {
    pub(crate) target: u64,
    pub(crate) role: MediaOperationRole,
    pub(crate) ordinal: NonZeroU64,
    pub(crate) directive: MediaFaultDirective,
}

/// Preserve target, role, ordinal and directive order before independent cases
/// are placed on workers. Measuring creates no runtime shared with a case.
pub(crate) fn fault_cases() -> Vec<FaultCase> {
    let mut cases = Vec::new();
    for target in TARGETS {
        let counts = measure(prepared_store(target).path(), target);
        assert!(
            !counts.is_empty(),
            "the publication performed media operations"
        );
        for (role, count) in counts {
            for nth in 1..=count {
                let ordinal = NonZeroU64::new(nth).unwrap();
                for directive in [
                    MediaFaultDirective::FailBefore {
                        kind: std::io::ErrorKind::Other,
                        raw_os_error: None,
                    },
                    MediaFaultDirective::IndeterminateAfterEffect,
                ] {
                    cases.push(FaultCase {
                        target,
                        role,
                        ordinal,
                        directive,
                    });
                }
            }
        }
    }
    cases
}

/// A cleanly closed store holding canonical commits `1..=COMMITS` and
/// transitions `1..target`, each transition published after the commit at
/// its position.
pub(crate) fn prepared_store(target: u64) -> worth_store_test_support::TemporaryDirectory {
    let _ = worth_store_testkit_recovery_runtime::setup_observation::record(
        "branch-transition-prefix",
        &|inputs| {
            inputs.unsigned("target", target);
            inputs.unsigned("canonical-commits", COMMITS);
            for position in 1..=COMMITS.max(target - 1) {
                if position <= COMMITS {
                    inputs.unsigned("commit-position", position);
                    inputs.bytes("commit-payload", &Journal::Commits.payload(position));
                }
                if position < target {
                    inputs.unsigned("transition-position", position);
                    inputs.bytes(
                        "transition-payload",
                        &Journal::Transitions.payload(position),
                    );
                }
            }
        },
    );
    let world = worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld::initialize_for_recovery("transitions-oracle")
        .expect("oracle store");
    for position in 1..=COMMITS.max(target - 1) {
        if position <= COMMITS {
            publish(
                world.serving(),
                world.placement(),
                crate::production_entry::ordered_journals::two_journals::Journal::Commits,
                position,
            )
            .expect("commit publication");
        }
        if position < target {
            publish(
                world.serving(),
                world.placement(),
                crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
                position,
            )
            .expect("transition publication");
        }
    }
    let root = world.retained_root();
    let _ = world.close();
    root
}

/// Counts each role's identified operations while transition `target`
/// publishes in a store prepared for it.
pub(crate) fn measure(root: &Path, target: u64) -> Vec<(MediaOperationRole, u64)> {
    let never = worth_store::physical_runtime::certification::MediaFaultDirective::FailBefore {
        kind: std::io::ErrorKind::Other,
        raw_os_error: None,
    };
    let faults: Vec<_> = worth_store::physical_runtime::certification::MediaOperationRole::ALL
        .iter()
        .map(|role| (*role, std::num::NonZeroU64::MAX, never.clone()))
        .collect();
    let (serving, activations) =
        open_with_faults(root, &faults).expect("fail-before is admissible for every role");
    for activation in &activations {
        activation.arm().unwrap();
    }
    publish(
        &serving,
        placement(),
        crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
        target,
    )
    .expect("the measured publication completes");
    let counts = worth_store::physical_runtime::certification::MediaOperationRole::ALL
        .iter()
        .zip(&activations)
        .map(|(role, activation)| (*role, activation.matching_operation_count()))
        .filter(|(_, count)| *count > 0)
        .collect();
    let _ = serving.close();
    counts
}

/// Whether a publication's fate is left to recovery.
pub(crate) fn leaves_fate_unresolved(outcome: &Outcome) -> bool {
    matches!(
        outcome,
        Err(worth_store::physical_runtime::OrderedJournalPublicationFailure::Indeterminate(_)
            | worth_store::physical_runtime::OrderedJournalPublicationFailure::CompletedWithoutRecord)
    )
}

/// What one faulted case exercised.
#[derive(Default)]
pub(crate) struct Exercised {
    /// A serving runtime refused the unresolved journal until reopen.
    pub(crate) unresolved_refusal: bool,
    /// An indeterminate publication that recovery found absent was retried
    /// at the same position with the same payload.
    pub(crate) retry_after_unresolved: bool,
}

/// One faulted publication of transition `target`, then production recovery.
pub(crate) fn run_case(
    target: u64,
    role: MediaOperationRole,
    nth: NonZeroU64,
    directive: MediaFaultDirective,
) -> Exercised {
    let store = prepared_store(target);
    let Some((serving, activations)) =
        open_with_faults(store.path(), &[(role, nth, directive.clone())])
    else {
        return crate::production_entry::ordered_journals::branch_transitions_oracle::Exercised::default();
    };
    activations[0].arm().unwrap();
    let outcome = publish(
        &serving,
        placement(),
        crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
        target,
    );
    assert!(activations[0].is_consumed(), "{role:?}#{nth} did not fire");
    let case = format!("target {target} {role:?}#{nth} {directive:?}");
    let unresolved_refusal = unresolved_journal_refuses(&serving, &outcome, target, &case);
    let _ = serving.abort();

    let recovered = serve(store.path(), &case);
    let head = crate::production_entry::ordered_journals::two_journals::Journal::Transitions
        .of(&recovered)
        .head()
        .unwrap()
        .map_or(0, |entry| entry.position());
    match &outcome {
        Ok(_) => assert_eq!(head, target, "{case}: acknowledged entry lost"),
        Err(worth_store::physical_runtime::OrderedJournalPublicationFailure::ProvenNoEffect(_)) => {
            assert_eq!(head, target - 1, "{case}: a no-effect entry appeared")
        }
        Err(_) => assert!(head == target - 1 || head == target, "{case}: head {head}"),
    }
    assert_holds(
        &recovered,
        crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
        head,
    );
    assert_holds(
        &recovered,
        crate::production_entry::ordered_journals::two_journals::Journal::Commits,
        COMMITS,
    );
    // The same publication again: durable state answers it.
    let retry = publish(
        &recovered,
        placement(),
        crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
        target,
    )
    .unwrap_or_else(|failure| panic!("{case}: retry: {failure:?}"));
    assert_eq!(
        matches!(
            retry,
            worth_store::physical_runtime::OrderedJournalPublished::AlreadyCommitted(_)
        ),
        head == target,
        "{case}: retry answer"
    );
    for position in target + 1..=TRANSITIONS {
        publish(
            &recovered,
            placement(),
            crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
            position,
        )
        .unwrap_or_else(|failure| panic!("{case}: suffix {position}: {failure:?}"));
    }
    assert_holds(
        &recovered,
        crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
        TRANSITIONS,
    );
    assert_holds(
        &recovered,
        crate::production_entry::ordered_journals::two_journals::Journal::Commits,
        COMMITS,
    );
    let _ = recovered.close();

    let again = serve(store.path(), &format!("{case} second recovery"));
    assert_holds(
        &again,
        crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
        TRANSITIONS,
    );
    assert_holds(
        &again,
        crate::production_entry::ordered_journals::two_journals::Journal::Commits,
        COMMITS,
    );
    let _ = again.close();
    Exercised {
        unresolved_refusal,
        retry_after_unresolved: leaves_fate_unresolved(&outcome) && head == target - 1,
    }
}

/// After a publication whose fate may be durable, its journal refuses with
/// `Unresolved` and resolves the position as unresolved for as long as this
/// incarnation serves. Returns whether that was observed.
pub(crate) fn unresolved_journal_refuses(
    serving: &ServingPhysicalRuntime,
    outcome: &Outcome,
    target: u64,
    case: &str,
) -> bool {
    if !leaves_fate_unresolved(outcome) || serving.branch_transitions().is_err() {
        return false;
    }
    let journal = crate::production_entry::ordered_journals::two_journals::Journal::Transitions;
    let retry = publish(serving, placement(), journal, target);
    assert!(
        matches!(
            retry,
            Err(worth_store::physical_runtime::OrderedJournalPublicationFailure::Unresolved { position }) if position == target
        ),
        "{case}: an unresolved journal admitted {retry:?}"
    );
    let resolution = journal
        .of(serving)
        .resolve(target, digest(&journal.payload(target)))
        .unwrap();
    assert_eq!(
        resolution,
        worth_store::physical_runtime::OrderedJournalResolution::Unresolved,
        "{case}: resolve"
    );
    true
}
