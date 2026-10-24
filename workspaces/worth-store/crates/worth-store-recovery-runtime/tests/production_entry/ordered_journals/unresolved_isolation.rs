//! An unresolved publication orders only its own journal. While the runtime
//! serves, the branch-transitions journal refuses as `Unresolved` and resolves
//! its position as unresolved; the canonical commits keep reading and
//! resolving, and their journal does not refuse as unresolved. Store's
//! physical layer still admits one pending root-changing publication per
//! incarnation, so a commit in that window is refused as proven no-effect
//! (`ScopeConflict`) rather than ordered behind the transition. Recovery then
//! decides the transition's fate alone and both journals publish again.

use std::num::NonZeroU64;

use worth_store::physical_runtime::certification::{MediaFaultDirective, MediaOperationRole};
use worth_store::physical_runtime::{
    OrderedJournalPublicationFailure, OrderedJournalResolution, PhysicalMutationProvenNoEffectCause,
};

use super::branch_transitions_oracle::{leaves_fate_unresolved, measure, prepared_store, COMMITS};
use super::canonical_journal_oracle::{open_with_faults, placement, serve};
use super::two_journals::{assert_holds, digest, publish, Journal};

/// The swept transition; transition 1 is already durable.
const TARGET: u64 = 2;

#[test]
fn an_unresolved_journal_blocks_only_itself_until_reopen() {
    let counts = measure(prepared_store(TARGET).path(), TARGET);
    let directives = [
        MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
        MediaFaultDirective::IndeterminateAfterEffect,
    ];
    let isolated = directives.iter().any(|directive| {
        counts.iter().any(|(role, count)| {
            (1..=*count).any(|nth| isolates(*role, NonZeroU64::new(nth).unwrap(), directive))
        })
    });
    assert!(
        isolated,
        "no unresolved transition left a serving runtime to observe"
    );
}

/// Faults one media operation of transition `TARGET`. When the publication's
/// fate is left unresolved and the runtime still serves, checks the isolation
/// and recovery, and returns true.
fn isolates(role: MediaOperationRole, nth: NonZeroU64, directive: &MediaFaultDirective) -> bool {
    let store = prepared_store(TARGET);
    let Some((serving, activations)) =
        open_with_faults(store.path(), &[(role, nth, directive.clone())])
    else {
        return false;
    };
    activations[0].arm().unwrap();
    let outcome = publish(&serving, placement(), Journal::Transitions, TARGET);
    if !leaves_fate_unresolved(&outcome) || serving.branch_transitions().is_err() {
        let _ = serving.abort();
        return false;
    }
    let case = format!("{role:?}#{nth} {directive:?}");
    let transitions = Journal::Transitions.of(&serving);
    for position in [TARGET, TARGET + 1] {
        assert!(
            matches!(
                publish(&serving, placement(), Journal::Transitions, position),
                Err(OrderedJournalPublicationFailure::Unresolved { position: blocked })
                    if blocked == TARGET
            ),
            "{case}: the unresolved journal admitted {position}"
        );
    }
    assert_eq!(
        transitions
            .resolve(TARGET, digest(&Journal::Transitions.payload(TARGET)))
            .unwrap(),
        OrderedJournalResolution::Unresolved
    );
    let durable = transitions.read(1, 1 << 20).expect("a durable transition");
    assert!(durable.bytes() == Journal::Transitions.payload(1));

    // The commits journal is not ordered by the unresolved transition: it
    // reads and resolves, and its publication reaches Store, whose physical
    // layer refuses it with no effect while the transition is pending.
    assert_holds(&serving, Journal::Commits, COMMITS);
    let commit = publish(&serving, placement(), Journal::Commits, COMMITS + 1);
    assert!(
        matches!(
            &commit,
            Err(OrderedJournalPublicationFailure::ProvenNoEffect(fate))
                if fate.cause() == PhysicalMutationProvenNoEffectCause::ScopeConflict
        ),
        "{case}: commit beside an unresolved transition answered {commit:?}"
    );
    assert_holds(&serving, Journal::Commits, COMMITS);
    let _ = serving.abort();

    let recovered = serve(store.path(), &case);
    assert_holds(&recovered, Journal::Commits, COMMITS);
    let head = Journal::Transitions
        .of(&recovered)
        .head()
        .unwrap()
        .map_or(0, |entry| entry.position());
    assert!(head == TARGET - 1 || head == TARGET, "{case}: head {head}");
    publish(&recovered, placement(), Journal::Transitions, TARGET)
        .unwrap_or_else(|failure| panic!("{case}: retry after recovery: {failure:?}"));
    publish(&recovered, placement(), Journal::Commits, COMMITS + 1)
        .unwrap_or_else(|failure| panic!("{case}: commit after recovery: {failure:?}"));
    assert_holds(&recovered, Journal::Transitions, TARGET);
    let _ = recovered.close();
    let again = serve(store.path(), &format!("{case} second recovery"));
    assert_holds(&again, Journal::Transitions, TARGET);
    assert_holds(&again, Journal::Commits, COMMITS + 1);
    let _ = again.close();
    true
}
