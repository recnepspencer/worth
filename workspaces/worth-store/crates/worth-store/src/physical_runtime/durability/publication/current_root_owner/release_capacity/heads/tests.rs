use super::*;
use crate::physical_runtime::PhysicalRecoveryAllocationAdmission;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::PersistedRecordIdentity;

#[path = "tests/local_transition.rs"]
mod local_transition;

fn entry(object: u8, cumulative_dropped: u64, terminal: bool) -> ReleaseCustodyHeadEntryV1 {
    let record = |ordinal| PersistedRecordIdentity::new([object; 16], ordinal).unwrap();
    ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([object; 16], 1).unwrap(),
        record(1),
        [1; 32],
        record(2),
        [2; 32],
        record(3),
        [3; 32],
        [4; 32],
        None,
        7,
        cumulative_dropped,
        terminal,
    )
    .unwrap()
}

fn successor(prior: ReleaseCustodyHeadEntryV1, terminal: bool) -> ReleaseCustodyHeadEntryV1 {
    ReleaseCustodyHeadEntryV1::new(
        prior.key(),
        PersistedRecordIdentity::new(prior.key().object(), 4).unwrap(),
        [5; 32],
        prior.manifest_record(),
        prior.manifest_frame_sha256(),
        prior.reservation_record(),
        prior.reservation_frame_sha256(),
        prior.source_basis_digest(),
        Some(
            ReleasedDropPredecessorV1::new(
                prior.descriptor_record(),
                prior.descriptor_frame_sha256(),
            )
            .unwrap(),
        ),
        8,
        prior.cumulative_dropped() + 1,
        terminal,
    )
    .unwrap()
}

fn root(
    first: ReleaseCustodyHeadKeyV1,
    last: ReleaseCustodyHeadKeyV1,
    block: u64,
) -> ReleaseCustodyHeadBlockReferenceV1 {
    ReleaseCustodyHeadBlockReferenceV1::new(9, block, 0, first, last, [1; 32]).unwrap()
}

#[test]
fn selected_roster_preserves_strict_membership_endpoints_and_digest() {
    let first = entry(1, 1, false);
    let second = entry(2, 1, false);
    let reference = root(first.key(), second.key(), 1);
    let roster =
        SelectedReleaseHeadRoster::from_selected(Some(reference), [first, second]).unwrap();
    assert_eq!(roster.head(first.key()), Some(first));
    assert_eq!(roster.head(second.key()), Some(second));
    assert_eq!(roster.head(entry(3, 1, false).key()), None);
    let mut expected = ReleaseCustodyHeadRosterDigestV1::new(Some(reference), 2);
    expected.push(first).unwrap();
    expected.push(second).unwrap();
    assert_eq!(roster.commitment().unwrap(), expected.finish());

    for invalid in [[second, first], [first, first]] {
        assert!(matches!(
            SelectedReleaseHeadRoster::from_selected(Some(reference), invalid),
            Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch)
        ));
    }
    assert!(matches!(
        SelectedReleaseHeadRoster::from_selected(
            Some(root(second.key(), second.key(), 2)),
            [first, second]
        ),
        Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch)
    ));
    assert!(matches!(
        SelectedReleaseHeadRoster::from_selected(None, [first]),
        Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch)
    ));
    assert!(matches!(
        SelectedReleaseHeadRoster::from_selected(Some(reference), []),
        Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch)
    ));
}

#[test]
fn selected_roster_transition_replaces_exact_prior_and_retires_only_terminal_head() {
    let first = entry(1, 1, false);
    let initial = root(first.key(), first.key(), 1);
    let next_root = root(first.key(), first.key(), 2);
    let next = successor(first, true);
    let mut roster = SelectedReleaseHeadRoster::from_selected(Some(initial), [first]).unwrap();
    assert_eq!(
        roster.apply_transition(
            Some(initial),
            Some(next_root),
            ReleaseCustodyHeadMutationV1::Upsert {
                expected_prior: Some(first),
                next,
            },
        ),
        Ok(())
    );
    assert_eq!(roster.head(first.key()), Some(next));
    assert_eq!(roster.root(), Some(next_root));
    assert_eq!(
        roster.apply_transition(
            Some(initial),
            None,
            ReleaseCustodyHeadMutationV1::RetireTerminal {
                expected_prior: next,
            },
        ),
        Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch)
    );
    assert_eq!(
        roster.apply_transition(
            Some(next_root),
            None,
            ReleaseCustodyHeadMutationV1::RetireTerminal {
                expected_prior: next,
            },
        ),
        Ok(())
    );
    assert_eq!(roster.root(), None);
    assert_eq!(roster.head(first.key()), None);
    assert_eq!(
        roster.commitment(),
        SelectedReleaseHeadRoster::empty().commitment()
    );

    let mut nonterminal = SelectedReleaseHeadRoster::from_selected(Some(initial), [first]).unwrap();
    assert_eq!(
        nonterminal.apply_transition(
            Some(initial),
            None,
            ReleaseCustodyHeadMutationV1::RetireTerminal {
                expected_prior: first,
            },
        ),
        Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch)
    );
}

#[test]
fn selected_roster_heap_census_counts_actual_reserved_backing() {
    let first = entry(1, 1, false);
    let roster =
        SelectedReleaseHeadRoster::from_selected(Some(root(first.key(), first.key(), 1)), [first])
            .unwrap();
    assert_eq!(
        roster.owned_heap_bytes(),
        Some((roster.entries.capacity() * std::mem::size_of::<ReleaseCustodyHeadEntryV1>()) as u64)
    );
    assert_eq!(
        SelectedReleaseHeadRoster::empty().owned_heap_bytes(),
        Some(0)
    );
}

#[test]
fn recovered_roster_denies_before_backing_allocation_and_charges_both_copies() {
    let first = entry(1, 1, false);
    let reference = root(first.key(), first.key(), 1);
    let identity = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([9; 16]).unwrap(),
    )
    .published_identity();
    let entry_bytes = std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64;
    let admission = PhysicalRecoveryAllocationAdmission::new(identity, entry_bytes * 4);
    let mut too_small =
        StoreRejoinResidentLedger::from_retained_with_limit(admission, 0, entry_bytes - 1).unwrap();
    assert!(matches!(
        SelectedReleaseHeadRoster::from_selected_admitted(Some(reference), [first], &mut too_small),
        Err(RecoveredReleaseLedgerDenial::Resident(
            crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { .. }
        ))
    ));
    assert_eq!(too_small.used(), 0);

    let mut sufficient =
        StoreRejoinResidentLedger::from_retained_with_limit(admission, 0, entry_bytes * 4).unwrap();
    let roster = SelectedReleaseHeadRoster::from_selected_admitted(
        Some(reference),
        [first],
        &mut sufficient,
    )
    .unwrap();
    let clone = roster.clone_admitted(&mut sufficient).unwrap();
    assert_eq!(roster.commitment(), clone.commitment());
    assert_eq!(
        sufficient.used(),
        roster.owned_heap_bytes().unwrap() + clone.owned_heap_bytes().unwrap()
    );
}
