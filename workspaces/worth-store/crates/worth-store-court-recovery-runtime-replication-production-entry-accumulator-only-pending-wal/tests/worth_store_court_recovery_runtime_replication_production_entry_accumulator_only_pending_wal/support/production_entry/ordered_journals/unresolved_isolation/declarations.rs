//! An unresolved publication orders only its own journal. While the runtime
//! serves, the branch-transitions journal refuses as `Unresolved` and resolves
//! its position as unresolved; the canonical commits keep reading and
//! resolving, and their journal does not refuse as unresolved. Store's
//! physical layer still admits one pending root-changing publication per
//! incarnation, so a commit in that window is refused as proven no-effect
//! (`ScopeConflict`) rather than ordered behind the transition. Recovery then
//! decides the transition's fate alone and both journals publish again.

pub(crate) use std::num::NonZeroU64;

pub(crate) use {
    worth_store::physical_runtime::certification::MediaFaultDirective,
    worth_store::physical_runtime::certification::MediaOperationRole,
};
pub(crate) use {
    worth_store::physical_runtime::OrderedJournalPublicationFailure,
    worth_store::physical_runtime::OrderedJournalResolution,
    worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause,
};

pub(crate) use {crate::production_entry::ordered_journals::branch_transitions_oracle::leaves_fate_unresolved, crate::production_entry::ordered_journals::branch_transitions_oracle::measure, crate::production_entry::ordered_journals::branch_transitions_oracle::prepared_store, crate::production_entry::ordered_journals::branch_transitions_oracle::COMMITS};
pub(crate) use {worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::open_with_faults, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::placement, worth_store_testkit_recovery_runtime::production_entry::ordered_journals::canonical_journal_oracle::serve};
pub(crate) use {crate::production_entry::ordered_journals::two_journals::assert_holds, crate::production_entry::ordered_journals::two_journals::digest, crate::production_entry::ordered_journals::two_journals::publish, crate::production_entry::ordered_journals::two_journals::Journal};

/// The swept transition; transition 1 is already durable.
pub(crate) const TARGET: u64 = 2;

/// Faults one media operation of transition `TARGET`. When the publication's
/// fate is left unresolved and the runtime still serves, checks the isolation
/// and recovery, and returns true.
pub(crate) fn isolates(
    role: MediaOperationRole,
    nth: NonZeroU64,
    directive: &MediaFaultDirective,
) -> bool {
    let store = prepared_store(TARGET);
    let Some((serving, activations)) =
        open_with_faults(store.path(), &[(role, nth, directive.clone())])
    else {
        return false;
    };
    activations[0].arm().unwrap();
    let outcome = publish(
        &serving,
        placement(),
        crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
        TARGET,
    );
    if !leaves_fate_unresolved(&outcome) || serving.branch_transitions().is_err() {
        let _ = serving.abort();
        return false;
    }
    let case = format!("{role:?}#{nth} {directive:?}");
    let transitions =
        crate::production_entry::ordered_journals::two_journals::Journal::Transitions.of(&serving);
    for position in [TARGET, TARGET + 1] {
        assert!(
            matches!(
                publish(&serving, placement(), crate::production_entry::ordered_journals::two_journals::Journal::Transitions, position),
                Err(worth_store::physical_runtime::OrderedJournalPublicationFailure::Unresolved { position: blocked })
                    if blocked == TARGET
            ),
            "{case}: the unresolved journal admitted {position}"
        );
    }
    assert_eq!(
        transitions
            .resolve(
                TARGET,
                digest(
                    &crate::production_entry::ordered_journals::two_journals::Journal::Transitions
                        .payload(TARGET)
                )
            )
            .unwrap(),
        worth_store::physical_runtime::OrderedJournalResolution::Unresolved
    );
    let durable = transitions.read(1, 1 << 20).expect("a durable transition");
    assert!(
        durable.bytes()
            == crate::production_entry::ordered_journals::two_journals::Journal::Transitions
                .payload(1)
    );

    // The commits journal is not ordered by the unresolved transition: it
    // reads and resolves, and its publication reaches Store, whose physical
    // layer refuses it with no effect while the transition is pending.
    assert_holds(
        &serving,
        crate::production_entry::ordered_journals::two_journals::Journal::Commits,
        COMMITS,
    );
    let commit = publish(
        &serving,
        placement(),
        crate::production_entry::ordered_journals::two_journals::Journal::Commits,
        COMMITS + 1,
    );
    assert!(
        matches!(
            &commit,
            Err(worth_store::physical_runtime::OrderedJournalPublicationFailure::ProvenNoEffect(fate))
                if fate.cause() == worth_store::physical_runtime::PhysicalMutationProvenNoEffectCause::ScopeConflict
        ),
        "{case}: commit beside an unresolved transition answered {commit:?}"
    );
    assert_holds(
        &serving,
        crate::production_entry::ordered_journals::two_journals::Journal::Commits,
        COMMITS,
    );
    let _ = serving.abort();

    let recovered = serve(store.path(), &case);
    assert_holds(
        &recovered,
        crate::production_entry::ordered_journals::two_journals::Journal::Commits,
        COMMITS,
    );
    let head = crate::production_entry::ordered_journals::two_journals::Journal::Transitions
        .of(&recovered)
        .head()
        .unwrap()
        .map_or(0, |entry| entry.position());
    assert!(head == TARGET - 1 || head == TARGET, "{case}: head {head}");
    publish(
        &recovered,
        placement(),
        crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
        TARGET,
    )
    .unwrap_or_else(|failure| panic!("{case}: retry after recovery: {failure:?}"));
    publish(
        &recovered,
        placement(),
        crate::production_entry::ordered_journals::two_journals::Journal::Commits,
        COMMITS + 1,
    )
    .unwrap_or_else(|failure| panic!("{case}: commit after recovery: {failure:?}"));
    assert_holds(
        &recovered,
        crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
        TARGET,
    );
    let _ = recovered.close();
    let again = serve(store.path(), &format!("{case} second recovery"));
    assert_holds(
        &again,
        crate::production_entry::ordered_journals::two_journals::Journal::Transitions,
        TARGET,
    );
    assert_holds(
        &again,
        crate::production_entry::ordered_journals::two_journals::Journal::Commits,
        COMMITS + 1,
    );
    let _ = again.close();
    true
}
