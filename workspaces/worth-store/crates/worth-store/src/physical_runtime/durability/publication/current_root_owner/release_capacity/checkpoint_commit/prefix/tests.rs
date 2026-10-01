use super::*;
use crate::physical_runtime::durability::publication::current_root_owner::release_capacity::{
    heads::SelectedReleaseHeadStep, PendingReleaseEvent, SelectedReleaseBatchBasis,
};
use std::num::NonZeroU64;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    DurablePhysicalRootManifest, OriginalDropReservationRequestV1, PersistedRecordIdentity,
    PhysicalCheckpointIdentity, ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1,
    ReleaseCustodyHeadKeyV1, ReleaseCustodyHeadMutationV1, ReleasedDropTipProvenanceV1,
    ReleasedDropWalFateWitnessV1,
};

fn record(object: u8, ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([object; 16], ordinal).unwrap()
}

fn entry(object: u8, terminal: bool) -> ReleaseCustodyHeadEntryV1 {
    ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([object; 16], 1).unwrap(),
        record(object, 1),
        [1; 32],
        record(object, 2),
        [2; 32],
        record(object, 3),
        [3; 32],
        [4; 32],
        None,
        7,
        1,
        terminal,
    )
    .unwrap()
}

fn root(entry: ReleaseCustodyHeadEntryV1, block: u64) -> ReleaseCustodyHeadBlockReferenceV1 {
    ReleaseCustodyHeadBlockReferenceV1::new(
        9,
        block,
        0,
        entry.key(),
        entry.key(),
        [block as u8; 32],
    )
    .unwrap()
}

fn basis(object: u8, cumulative: u64) -> SelectedReleaseBatchBasis {
    SelectedReleaseBatchBasis {
        descriptor_record: record(object, 1),
        descriptor_frame_sha256: [1; 32],
        custody_digest: [5; 32],
        reservation_record: record(object, 3),
        reservation_frame_sha256: [3; 32],
        request: OriginalDropReservationRequestV1::new([1; 32], [2; 32], 4, 8).unwrap(),
        fate: ReleasedDropWalFateWitnessV1::new(1, 2, [6; 32], [7; 32]).unwrap(),
        candidate_root_generation: 9,
        candidate_root_sha256: [8; 32],
        predecessor: None,
        cumulative_dropped: cumulative,
        cumulative_digest: [cumulative as u8; 32],
        terminal: true,
    }
}

fn snapshot(root: Option<ReleaseCustodyHeadBlockReferenceV1>) -> SelectedCheckpointCustodySnapshot {
    let store = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([9; 16]).unwrap(),
    )
    .published_identity();
    let manifest = DurablePhysicalRootManifest::builder(10, 1, 4, 1)
        .next_release_custody_head_block(4)
        .release_custody_head_root(root)
        .admit()
        .unwrap();
    SelectedCheckpointCustodySnapshot::VerifiedLegacy {
        checkpoint: PhysicalCheckpointIdentity::new(store, NonZeroU64::new(1).unwrap()),
        root: manifest,
        root_sha256: [9; 32],
    }
}

fn events() -> (
    SelectedReleaseCustodyLedger,
    ReleaseCustodyHeadBlockReferenceV1,
) {
    let first = entry(1, true);
    let second = entry(2, true);
    let first_root = root(first, 1);
    let second_root = root(second, 2);
    let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    ledger.pending_events.push(
        PendingReleaseEvent::for_drop(
            basis(1, 1),
            SelectedReleaseHeadStep::new(
                None,
                Some(first_root),
                ReleaseCustodyHeadMutationV1::Upsert {
                    expected_prior: None,
                    next: first,
                },
            ),
        )
        .unwrap(),
    );
    ledger.pending_events.push(
        PendingReleaseEvent::for_retirement(SelectedReleaseHeadStep::new(
            Some(first_root),
            None,
            ReleaseCustodyHeadMutationV1::RetireTerminal {
                expected_prior: first,
            },
        ))
        .unwrap(),
    );
    ledger.pending_events.push(
        PendingReleaseEvent::for_drop(
            basis(2, 2),
            SelectedReleaseHeadStep::new(
                None,
                Some(second_root),
                ReleaseCustodyHeadMutationV1::Upsert {
                    expected_prior: None,
                    next: second,
                },
            ),
        )
        .unwrap(),
    );
    (ledger, second_root)
}

#[test]
fn drop_retirement_drop_prefix_requires_root_and_authenticated_drop_count() {
    let (mut ledger, second_root) = events();
    let no_head = snapshot(None);
    let first = expected_batch(&no_head, 0, basis(1, 1)).unwrap();
    let (heads, prefix_len) = checkpoint_prefix(&ledger, &no_head, &[first]).unwrap();
    assert_eq!(heads.root(), None);
    assert_eq!(prefix_len, 2); // The root recurs only after the retirement.
    ledger.pending_events.drain(..prefix_len);
    assert_eq!(ledger.pending_drop_count(), 1);
    assert_eq!(ledger.pending_events.len(), 1);
    let after = snapshot(Some(second_root));
    let second = expected_batch(&after, 0, basis(2, 2)).unwrap();
    let (heads, consumed) = checkpoint_prefix(&ledger, &after, &[second]).unwrap();
    assert_eq!(heads.root(), Some(second_root));
    assert_eq!(consumed, 1);
}

#[test]
fn reordered_or_missing_tag_seven_batch_never_drains_selected_events() {
    let (ledger, second_root) = events();
    let target = snapshot(Some(second_root));
    let first = expected_batch(&target, 0, basis(1, 1)).unwrap();
    let second = expected_batch(&target, 1, basis(2, 2)).unwrap();
    let original = ledger.pending_events.len();
    for incorrect in [&[second, first][..], &[first][..]] {
        assert!(checkpoint_prefix(&ledger, &target, incorrect).is_err());
        assert_eq!(ledger.pending_events.len(), original);
        assert_eq!(ledger.checkpoint_heads.root(), None);
    }
    assert_eq!(
        checkpoint_prefix(&ledger, &target, &[first, second])
            .unwrap()
            .1,
        3
    );
}

#[test]
fn event_constructors_reject_wrong_mutation_and_nonterminal_retirement() {
    let first = entry(1, true);
    let root = root(first, 1);
    assert!(
        PendingReleaseEvent::for_retirement(SelectedReleaseHeadStep::new(
            None,
            Some(root),
            ReleaseCustodyHeadMutationV1::Upsert {
                expected_prior: None,
                next: first,
            },
        ))
        .is_err()
    );
    assert!(PendingReleaseEvent::for_drop(
        basis(1, 1),
        SelectedReleaseHeadStep::new(
            Some(root),
            None,
            ReleaseCustodyHeadMutationV1::RetireTerminal {
                expected_prior: first,
            },
        ),
    )
    .is_err());
    assert!(
        PendingReleaseEvent::for_retirement(SelectedReleaseHeadStep::new(
            Some(root),
            None,
            ReleaseCustodyHeadMutationV1::RetireTerminal {
                expected_prior: entry(1, false),
            },
        ))
        .is_err()
    );
}

#[test]
fn prefix_rejects_wrong_source_and_result_key_before_any_drain() {
    let (mut ledger, _) = events();
    let target = snapshot(None);
    let certificate = expected_batch(&target, 0, basis(1, 1)).unwrap();
    let first = entry(1, true);
    let wrong_source = root(first, 3);
    ledger.pending_events[0] = PendingReleaseEvent::for_drop(
        basis(1, 1),
        SelectedReleaseHeadStep::new(
            Some(wrong_source),
            Some(root(first, 1)),
            ReleaseCustodyHeadMutationV1::Upsert {
                expected_prior: None,
                next: first,
            },
        ),
    )
    .unwrap();
    assert!(checkpoint_prefix(&ledger, &target, &[certificate]).is_err());
    assert_eq!(ledger.pending_events.len(), 3);
    assert_eq!(ledger.checkpoint_heads.root(), None);

    let wrong_entry = entry(2, true);
    ledger.pending_events[0] = PendingReleaseEvent::for_drop(
        basis(1, 1),
        SelectedReleaseHeadStep::new(
            None,
            Some(root(first, 1)),
            ReleaseCustodyHeadMutationV1::Upsert {
                expected_prior: None,
                next: wrong_entry,
            },
        ),
    )
    .unwrap();
    assert!(checkpoint_prefix(&ledger, &target, &[certificate]).is_err());
    assert_eq!(ledger.pending_events.len(), 3);
}

#[test]
fn zero_batch_retirement_preserves_prior_ratchet_and_later_drop() {
    let prior = entry(1, true);
    let prior_root = root(prior, 1);
    let later = entry(2, true);
    let later_root = root(later, 2);
    let prior_batch = basis(1, 7);
    let tip = ReleasedDropTipProvenanceV1::new(
        prior_batch.descriptor_record,
        prior_batch.descriptor_frame_sha256,
        prior_batch.reservation_record,
        prior_batch.reservation_frame_sha256,
        prior_batch.request,
        prior_batch.fate,
        prior_batch.candidate_root_generation,
        prior_batch.candidate_root_sha256,
    )
    .unwrap();
    let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    ledger.checkpoint_heads =
        SelectedReleaseHeadRoster::from_selected(Some(prior_root), [prior]).unwrap();
    ledger.checkpoint = Some(snapshot(Some(prior_root)).checkpoint());
    ledger.prior_cumulative_dropped = 7;
    ledger.cumulative_dropped = 7;
    ledger.prior_cumulative_digest = [7; 32];
    ledger.cumulative_digest = [7; 32];
    ledger.prior_tip = Some(tip);
    ledger.selected_tip = Some(tip);
    ledger.prior_terminal = true;
    ledger.terminal = true;
    ledger.pending_events.push(
        PendingReleaseEvent::for_retirement(SelectedReleaseHeadStep::new(
            Some(prior_root),
            None,
            ReleaseCustodyHeadMutationV1::RetireTerminal {
                expected_prior: prior,
            },
        ))
        .unwrap(),
    );
    ledger.pending_events.push(
        PendingReleaseEvent::for_drop(
            basis(2, 8),
            SelectedReleaseHeadStep::new(
                None,
                Some(later_root),
                ReleaseCustodyHeadMutationV1::Upsert {
                    expected_prior: None,
                    next: later,
                },
            ),
        )
        .unwrap(),
    );
    let target = snapshot(None);
    let (folded, consumed) = checkpoint_prefix(&ledger, &target, &[]).unwrap();
    assert_eq!(folded.root(), None);
    assert_eq!(consumed, 1);
    ledger.pending_events.drain(..consumed);
    ledger.checkpoint_heads = folded;
    assert_eq!(ledger.pending_drop_count(), 1);
    assert_eq!(ledger.cumulative_dropped, 7);
    assert_eq!(ledger.cumulative_digest, [7; 32]);
    assert_eq!(ledger.selected_tip, Some(tip));
    assert_eq!(ledger.prior_tip, Some(tip));
    let later_checkpoint = snapshot(Some(later_root));
    let later_batch = expected_batch(&later_checkpoint, 0, basis(2, 8)).unwrap();
    assert_eq!(
        checkpoint_prefix(&ledger, &later_checkpoint, &[later_batch])
            .unwrap()
            .1,
        1
    );
}
