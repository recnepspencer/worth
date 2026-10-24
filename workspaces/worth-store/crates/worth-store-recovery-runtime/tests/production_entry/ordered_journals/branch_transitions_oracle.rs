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

use std::num::NonZeroU64;
use std::path::Path;

use worth_store::physical_runtime::certification::{MediaFaultDirective, MediaOperationRole};
use worth_store::physical_runtime::{
    OrderedJournalPublicationFailure, OrderedJournalPublished, OrderedJournalResolution,
    ServingPhysicalRuntime,
};

use super::canonical_journal_oracle::{open_with_faults, placement, serve};
use super::two_journals::{assert_holds, digest, publish, Journal};
use super::*;

/// Canonical commits published before the swept transition.
pub(crate) const COMMITS: u64 = 3;
const TRANSITIONS: u64 = 4;
/// The journal's first entry (no anchor yet) and a later one with a spine.
const TARGETS: [u64; 2] = [1, 3];

type Outcome = Result<OrderedJournalPublished, OrderedJournalPublicationFailure>;

/// A cleanly closed store holding canonical commits `1..=COMMITS` and
/// transitions `1..target`, each transition published after the commit at
/// its position.
pub(crate) fn prepared_store(target: u64) -> worth_store_test_support::TemporaryDirectory {
    let world = PhysicalResidencyStoreWorld::initialize_for_recovery("transitions-oracle")
        .expect("oracle store");
    for position in 1..=COMMITS.max(target - 1) {
        if position <= COMMITS {
            publish(
                world.serving(),
                world.placement(),
                Journal::Commits,
                position,
            )
            .expect("commit publication");
        }
        if position < target {
            publish(
                world.serving(),
                world.placement(),
                Journal::Transitions,
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
    let never = MediaFaultDirective::FailBefore {
        kind: std::io::ErrorKind::Other,
        raw_os_error: None,
    };
    let faults: Vec<_> = MediaOperationRole::ALL
        .iter()
        .map(|role| (*role, NonZeroU64::MAX, never.clone()))
        .collect();
    let (serving, activations) =
        open_with_faults(root, &faults).expect("fail-before is admissible for every role");
    for activation in &activations {
        activation.arm().unwrap();
    }
    publish(&serving, placement(), Journal::Transitions, target)
        .expect("the measured publication completes");
    let counts = MediaOperationRole::ALL
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
        Err(OrderedJournalPublicationFailure::Indeterminate(_)
            | OrderedJournalPublicationFailure::CompletedWithoutRecord)
    )
}

/// What one faulted case exercised.
#[derive(Default)]
struct Exercised {
    /// A serving runtime refused the unresolved journal until reopen.
    unresolved_refusal: bool,
    /// An indeterminate publication that recovery found absent was retried
    /// at the same position with the same payload.
    retry_after_unresolved: bool,
}

/// One faulted publication of transition `target`, then production recovery.
fn run_case(
    target: u64,
    role: MediaOperationRole,
    nth: NonZeroU64,
    directive: MediaFaultDirective,
) -> Exercised {
    let store = prepared_store(target);
    let Some((serving, activations)) =
        open_with_faults(store.path(), &[(role, nth, directive.clone())])
    else {
        return Exercised::default();
    };
    activations[0].arm().unwrap();
    let outcome = publish(&serving, placement(), Journal::Transitions, target);
    assert!(activations[0].is_consumed(), "{role:?}#{nth} did not fire");
    let case = format!("target {target} {role:?}#{nth} {directive:?}");
    let unresolved_refusal = unresolved_journal_refuses(&serving, &outcome, target, &case);
    let _ = serving.abort();

    let recovered = serve(store.path(), &case);
    let head = Journal::Transitions
        .of(&recovered)
        .head()
        .unwrap()
        .map_or(0, |entry| entry.position());
    match &outcome {
        Ok(_) => assert_eq!(head, target, "{case}: acknowledged entry lost"),
        Err(OrderedJournalPublicationFailure::ProvenNoEffect(_)) => {
            assert_eq!(head, target - 1, "{case}: a no-effect entry appeared")
        }
        Err(_) => assert!(head == target - 1 || head == target, "{case}: head {head}"),
    }
    assert_holds(&recovered, Journal::Transitions, head);
    assert_holds(&recovered, Journal::Commits, COMMITS);
    // The same publication again: durable state answers it.
    let retry = publish(&recovered, placement(), Journal::Transitions, target)
        .unwrap_or_else(|failure| panic!("{case}: retry: {failure:?}"));
    assert_eq!(
        matches!(retry, OrderedJournalPublished::AlreadyCommitted(_)),
        head == target,
        "{case}: retry answer"
    );
    for position in target + 1..=TRANSITIONS {
        publish(&recovered, placement(), Journal::Transitions, position)
            .unwrap_or_else(|failure| panic!("{case}: suffix {position}: {failure:?}"));
    }
    assert_holds(&recovered, Journal::Transitions, TRANSITIONS);
    assert_holds(&recovered, Journal::Commits, COMMITS);
    let _ = recovered.close();

    let again = serve(store.path(), &format!("{case} second recovery"));
    assert_holds(&again, Journal::Transitions, TRANSITIONS);
    assert_holds(&again, Journal::Commits, COMMITS);
    let _ = again.close();
    Exercised {
        unresolved_refusal,
        retry_after_unresolved: leaves_fate_unresolved(&outcome) && head == target - 1,
    }
}

/// After a publication whose fate may be durable, its journal refuses with
/// `Unresolved` and resolves the position as unresolved for as long as this
/// incarnation serves. Returns whether that was observed.
fn unresolved_journal_refuses(
    serving: &ServingPhysicalRuntime,
    outcome: &Outcome,
    target: u64,
    case: &str,
) -> bool {
    if !leaves_fate_unresolved(outcome) || serving.branch_transitions().is_err() {
        return false;
    }
    let journal = Journal::Transitions;
    let retry = publish(serving, placement(), journal, target);
    assert!(
        matches!(
            retry,
            Err(OrderedJournalPublicationFailure::Unresolved { position }) if position == target
        ),
        "{case}: an unresolved journal admitted {retry:?}"
    );
    let resolution = journal
        .of(serving)
        .resolve(target, digest(&journal.payload(target)))
        .unwrap();
    assert_eq!(
        resolution,
        OrderedJournalResolution::Unresolved,
        "{case}: resolve"
    );
    true
}

#[test]
fn every_media_operation_of_a_branch_transition_recovers_to_the_old_or_new_head() {
    let mut unresolved_refusals = 0;
    let mut retries_after_unresolved = 0;
    for target in TARGETS {
        let counts = measure(prepared_store(target).path(), target);
        assert!(
            !counts.is_empty(),
            "the publication performed media operations"
        );
        for (role, count) in counts {
            for nth in 1..=count {
                let nth = NonZeroU64::new(nth).unwrap();
                for directive in [
                    MediaFaultDirective::FailBefore {
                        kind: std::io::ErrorKind::Other,
                        raw_os_error: None,
                    },
                    MediaFaultDirective::IndeterminateAfterEffect,
                ] {
                    let exercised = run_case(target, role, nth, directive);
                    unresolved_refusals += usize::from(exercised.unresolved_refusal);
                    retries_after_unresolved += usize::from(exercised.retry_after_unresolved);
                }
            }
        }
    }
    assert!(
        unresolved_refusals > 0,
        "no indeterminate publication left a serving runtime to refuse its journal"
    );
    assert!(
        retries_after_unresolved > 0,
        "no indeterminate publication was retried after recovery found it absent"
    );
}
