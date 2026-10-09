//! The two joins that bind the selected ledger's head roster to the
//! published root: the attestation before the retirement and the commit of
//! its completed member after it.

use worth_store_physical_format::{
    DurablePhysicalRootManifest, PersistedRecordIdentity, ReleaseCustodyHeadBlockReferenceV1,
    ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
};

use super::super::heads::SelectedReleaseHeadRoster;
use super::{
    published_head_step, ReleaseCertificateCapacityDenial, SelectedReleaseCustodyLedger,
    TerminalHeadAttestationDenial as Denial,
};
use crate::physical_runtime::terminal_head_retirement_fixture::retirement;

const TREE: u64 = 9;
/// The generation the ledger's head blocks were written at.
const HEAD_GENERATION: u64 = 4;

fn head(terminal: bool) -> ReleaseCustodyHeadEntryV1 {
    let record = |ordinal| PersistedRecordIdentity::new([1; 16], ordinal).unwrap();
    ReleaseCustodyHeadEntryV1::new(
        ReleaseCustodyHeadKeyV1::new([1; 16], 1).unwrap(),
        record(1),
        [1; 32],
        record(2),
        [2; 32],
        record(3),
        [3; 32],
        [4; 32],
        None,
        7,
        1,
        terminal,
    )
    .unwrap()
}

fn reference(key: ReleaseCustodyHeadKeyV1, block: u64) -> ReleaseCustodyHeadBlockReferenceV1 {
    ReleaseCustodyHeadBlockReferenceV1::new(HEAD_GENERATION, block, 0, key, key, [1; 32]).unwrap()
}

fn published(
    generation: u64,
    tree: u64,
    head_root: Option<ReleaseCustodyHeadBlockReferenceV1>,
) -> DurablePhysicalRootManifest {
    DurablePhysicalRootManifest::builder(generation, tree, 2, 1)
        .release_custody_head_root(head_root)
        .next_release_custody_head_block(64)
        .admit()
        .unwrap()
}

/// A ledger whose checkpoint and effective rosters both hold `entry` under
/// `root`.
fn ledger(
    root: ReleaseCustodyHeadBlockReferenceV1,
    entry: ReleaseCustodyHeadEntryV1,
) -> SelectedReleaseCustodyLedger {
    let mut ledger = SelectedReleaseCustodyLedger::trusted_genesis();
    ledger.checkpoint_heads =
        SelectedReleaseHeadRoster::from_selected(Some(root), [entry]).unwrap();
    ledger.effective_heads = SelectedReleaseHeadRoster::from_selected(Some(root), [entry]).unwrap();
    ledger
}

#[test]
fn a_head_is_attested_only_at_the_root_that_names_the_ledgers_head_tree() {
    let entry = head(true);
    let root = reference(entry.key(), 1);
    let ledger = ledger(root, entry);
    let current = published(5, TREE, Some(root));
    let attested = ledger
        .attest_terminal_head(&current, entry.key())
        .expect("the checkpointed terminal head of the published tree");
    assert_eq!(attested.entry(), entry);
    assert_eq!(attested.root(), current.root_cell());

    for other in [None, Some(reference(entry.key(), 2))] {
        assert_eq!(
            ledger
                .attest_terminal_head(&published(5, TREE, other), entry.key())
                .map(|attested| attested.entry()),
            Err(Denial::SelectedHeadRootMismatch),
            "the ledger describes another head tree than the published one"
        );
    }
}

#[test]
fn the_head_roots_agreement_answers_before_the_heads_own_state() {
    let entry = head(false);
    let root = reference(entry.key(), 1);
    let ledger = ledger(root, entry);
    let attest = |current: &DurablePhysicalRootManifest, key| {
        ledger
            .attest_terminal_head(current, key)
            .map(|attested| attested.entry())
    };
    assert_eq!(
        attest(&published(5, TREE, None), entry.key()),
        Err(Denial::SelectedHeadRootMismatch)
    );
    let current = published(5, TREE, Some(root));
    assert_eq!(attest(&current, entry.key()), Err(Denial::NonterminalHead));
    assert_eq!(
        attest(&current, ReleaseCustodyHeadKeyV1::new([2; 16], 1).unwrap()),
        Err(Denial::NoHead)
    );
}

#[test]
fn only_the_member_that_published_the_current_root_becomes_a_roster_step() {
    let member = retirement();
    let (tree, generation) = (member.tree_identity(), member.source_root_generation() + 1);
    let joins = |completed: u64, current: &DurablePhysicalRootManifest| {
        published_head_step(&member, completed, current).map(drop)
    };
    assert_eq!(
        joins(
            generation,
            &published(generation, tree, member.result_root())
        ),
        Ok(())
    );
    let mismatch = Err(ReleaseCertificateCapacityDenial::SelectedFactMismatch);
    assert_eq!(
        joins(
            generation,
            &published(generation, tree + 1, member.result_root())
        ),
        mismatch,
        "another tree"
    );
    assert_eq!(
        joins(
            generation,
            &published(generation, tree, Some(member.source_root()))
        ),
        mismatch,
        "the published root still names the source head tree"
    );
    assert_eq!(
        joins(
            generation,
            &published(generation + 1, tree, member.result_root())
        ),
        mismatch,
        "a later root was published over the member's"
    );
    assert_eq!(
        joins(
            generation + 1,
            &published(generation, tree, member.result_root())
        ),
        mismatch,
        "the member completed another generation"
    );
}
