use super::*;

#[test]
fn new_key_growth_admits_old_and_new_backing_together() {
    let first = entry(1, 1, false);
    let source = root(first.key(), first.key(), 1);
    let identity = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([9; 16]).unwrap(),
    )
    .published_identity();
    let bytes = std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64;
    let allocation = PhysicalRecoveryAllocationAdmission::new(identity, bytes * 4);
    let mut initial =
        StoreRejoinResidentLedger::from_retained_with_limit(allocation, 0, bytes * 4).unwrap();
    let mut roster =
        SelectedReleaseHeadRoster::from_selected_admitted(Some(source), [first], &mut initial)
            .unwrap();
    let original = roster.commitment().unwrap();
    let retained = roster.owned_heap_bytes().unwrap();
    let capacity = roster.entries.capacity();
    let mut growth =
        StoreRejoinResidentLedger::from_retained_with_limit(allocation, retained, bytes * 2)
            .unwrap();
    assert!(
        matches!(roster.reserve_for_key(entry(2, 1, false).key(), &mut growth),
        Err(RecoveredReleaseLedgerDenial::Resident(
            crate::physical_runtime::PhysicalRecoveryRejoinResidentDenial::BudgetExceeded { required, admitted }
        )) if required > admitted && admitted == bytes * 2)
    );
    assert_eq!(roster.entries.capacity(), capacity);
    assert_eq!(roster.commitment().unwrap(), original);
    assert_eq!(growth.used(), retained);
}

#[test]
fn invalid_result_endpoints_and_presence_leave_roster_unchanged() {
    let first = entry(1, 1, false);
    let last = entry(3, 1, true);
    let source = root(first.key(), last.key(), 1);
    let mut roster = SelectedReleaseHeadRoster::from_selected(Some(source), [first, last]).unwrap();
    let original = roster.commitment().unwrap();
    let mutation = ReleaseCustodyHeadMutationV1::Upsert {
        expected_prior: Some(first),
        next: successor(first, false),
    };
    for result in [
        None,
        Some(root(last.key(), last.key(), 2)),
        Some(root(first.key(), first.key(), 2)),
    ] {
        assert_eq!(
            roster.apply_transition(Some(source), result, mutation),
            Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch)
        );
        assert_eq!(roster.commitment().unwrap(), original);
        assert_eq!(roster.head(first.key()), Some(first));
    }
    assert!(roster
        .apply_transition(
            Some(source),
            None,
            ReleaseCustodyHeadMutationV1::RetireTerminal {
                expected_prior: last
            }
        )
        .is_err());
    assert_eq!(roster.commitment().unwrap(), original);
    roster
        .apply_transition(
            Some(source),
            Some(root(first.key(), first.key(), 3)),
            ReleaseCustodyHeadMutationV1::RetireTerminal {
                expected_prior: last,
            },
        )
        .unwrap();
    assert_eq!(roster.head(last.key()), None);
    assert_eq!(roster.commitment().unwrap().0, 1);
}

#[test]
fn existing_key_updates_reuse_backing_with_unrelated_heads() {
    for count in [1, 255] {
        let entries: Vec<_> = (1..=count).map(|object| entry(object, 1, false)).collect();
        let first = entries[0];
        let last = *entries.last().unwrap();
        let source = root(first.key(), last.key(), 1);
        let result = root(first.key(), last.key(), 2);
        let mut roster = SelectedReleaseHeadRoster::from_selected(Some(source), entries).unwrap();
        let capacity = roster.entries.capacity();
        let pointer = roster.entries.as_ptr();
        let next = successor(first, false);
        let transition = SelectedReleaseHeadStep::new(
            Some(source),
            Some(result),
            ReleaseCustodyHeadMutationV1::Upsert {
                expected_prior: Some(first),
                next,
            },
        )
        .prepare(&roster)
        .unwrap();
        assert!(transition.has_backing(&roster));
        transition.apply(&mut roster);
        assert_eq!(roster.entries.capacity(), capacity);
        assert_eq!(roster.entries.as_ptr(), pointer);
        assert_eq!(roster.head(first.key()), Some(next));
        assert_eq!(roster.len(), u64::from(count));
        if count > 1 {
            assert_eq!(roster.head(last.key()), Some(last));
        }
        // Commitment is deliberately requested only at the checkpoint seam.
        assert_eq!(roster.commitment().unwrap().0, u64::from(count));
    }
}
