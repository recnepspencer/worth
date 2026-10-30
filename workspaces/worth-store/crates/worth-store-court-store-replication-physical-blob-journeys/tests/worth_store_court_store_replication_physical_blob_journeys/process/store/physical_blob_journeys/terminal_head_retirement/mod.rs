//! Production terminal head retired, in one process. No world here reopens:
//! recovery of the retirement member belongs to the recovery slice.

#[allow(unused_imports)]
use crate::physical_blob_journeys::terminal_head_retirement::*;

use worth_store_testkit_store::fixture::serving_from_initialization;

#[test]
fn retirement_removes_exactly_the_attested_head_and_the_next_checkpoint_ratchets_the_roster() {
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let first = publish(&serving, 0x31);
    let second = publish(&serving, 0x47);
    release_to_terminal(&serving, &first);
    release_to_terminal(&serving, &second);
    let source = observe(&serving);
    assert_eq!(source.effective_heads().0, 2);
    assert_eq!(source.checkpoint_heads(), source.effective_heads());
    assert_eq!(source.pending_retirements(), 0);
    let source_wal = wal(&serving);

    let receipt = retire(&serving, &first).expect("every dependency is settled");
    assert_eq!(receipt.object(), first.object());
    assert_eq!(receipt.generation(), first.generation());
    assert_eq!(receipt.source_root_generation(), source.root_generation());
    assert!(!receipt.head_tree_emptied());
    let retired = observe(&serving);
    assert_eq!(receipt.result_root_generation(), retired.root_generation());
    assert!(
        retired.is_head_only_successor_of(&source),
        "the result root may differ only in the head root and head block frontier:
{source:?}
{retired:?}"
    );
    assert!(retired.has_head_tree());
    assert_eq!(retired.effective_heads().0, 1);
    assert_ne!(retired.effective_heads().1, source.effective_heads().1);
    assert_eq!(retired.checkpoint_heads(), source.checkpoint_heads());
    assert_eq!(retired.pending_retirements(), 1);
    assert_eq!(retired.pending_drops(), source.pending_drops());
    assert!(wal(&serving).last_lsn_end() > source_wal.last_lsn_end());

    assert_settled_absence(&serving, &first);
    checkpoint(&serving, 0x82);
    let ratcheted = observe(&serving);
    assert_eq!(ratcheted.effective_heads(), retired.effective_heads());
    assert_eq!(ratcheted.checkpoint_heads(), retired.effective_heads());
    assert_eq!(ratcheted.pending_retirements(), 0);
    assert_settled_absence(&serving, &first);

    let last = retire(&serving, &second).expect("the other head is settled as well");
    assert!(last.head_tree_emptied());
    let emptied = observe(&serving);
    assert!(emptied.is_head_only_successor_of(&ratcheted));
    assert!(!emptied.has_head_tree());
    assert_eq!(emptied.effective_heads().0, 0);
    assert_eq!(emptied.checkpoint_heads(), ratcheted.checkpoint_heads());
    assert_settled_absence(&serving, &second);
    checkpoint(&serving, 0x83);
    let empty = observe(&serving);
    assert_eq!(empty.checkpoint_heads(), emptied.effective_heads());
    assert_eq!(empty.pending_retirements(), 0);
    assert_settled_absence(&serving, &second);
    assert_settled_absence(&serving, &first);
    serving.close();
}

/// Each retirement funds its own pending event at admission, before the WAL
/// effect. The commit after the effect only takes the funded slot, so a
/// whole roster retires inside one checkpoint window.
#[test]
fn a_roster_retired_inside_one_checkpoint_window_funds_each_pending_event_before_its_effect() {
    const SEEDS: [u8; 4] = [0x11, 0x21, 0x31, 0x41];
    let directory = tempfile::tempdir().unwrap();
    let serving = serving_from_initialization(directory.path());
    let proofs = SEEDS.map(|seed| publish(&serving, seed));
    for proof in &proofs {
        release_to_terminal(&serving, proof);
    }
    let source = observe(&serving);
    assert_eq!(source.effective_heads().0, SEEDS.len() as u64);
    assert_eq!(source.checkpoint_heads(), source.effective_heads());
    assert_eq!(
        (source.pending_drops(), source.pending_retirements()),
        (0, 0)
    );

    for (retired, proof) in proofs.iter().enumerate() {
        retire(&serving, proof).expect("the admission funded this retirement's pending event");
        let window = observe(&serving);
        assert_eq!(window.pending_retirements(), retired + 1);
        assert_eq!(
            window.effective_heads().0,
            (SEEDS.len() - retired - 1) as u64
        );
        assert_eq!(window.checkpoint_heads(), source.checkpoint_heads());
    }
    assert!(!observe(&serving).has_head_tree());
    checkpoint(&serving, 0x82);
    let ratcheted = observe(&serving);
    assert_eq!(ratcheted.checkpoint_heads(), ratcheted.effective_heads());
    assert_eq!(ratcheted.checkpoint_heads().0, 0);
    assert_eq!(ratcheted.pending_retirements(), 0);
    for proof in &proofs {
        assert_settled_absence(&serving, proof);
    }
    serving.close();
}
#[path = "denials/mod.rs"]
pub(crate) mod denials;
#[path = "frontier/mod.rs"]
pub(crate) mod frontier;
#[path = "non_reissue/mod.rs"]
pub(crate) mod non_reissue;
