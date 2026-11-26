//! Retained pending reads are exactly those of live members' decisions: a
//! decision's frame is kept only once it is adopted, a chain's successor
//! replaces what its superseded round read, and a reset forgets all.
use super::{DecisionReads, PendingReads, ReadOwner};
use worth_query_execution_lineage::facade::primary_graph::invariant_projection::operation_reader::PendingNameRead;
use worth_relational::facade::lineage::PersistentNameSubjectKind;

fn birth(byte: u8) -> PendingNameRead {
    PendingNameRead::Birth {
        subject: PersistentNameSubjectKind::Entity,
        name: [byte; 32],
    }
}

fn all(reads: &PendingReads) -> Vec<PendingNameRead> {
    reads.all().copied().collect()
}

/// A decision that refuses, fails or retries never appends: its frame is
/// closed and dropped, and nothing it read is retained.
#[test]
fn a_decision_that_never_appends_retains_nothing() {
    let mut reads = PendingReads::default();
    reads.begin_decision();
    reads.retain(birth(1));
    assert!(all(&reads).is_empty(), "staged reads are not yet retained");
    let dropped = reads.end_decision();
    assert_eq!(dropped, DecisionReads::from([birth(1)]));
    assert!(all(&reads).is_empty());
}

/// A chain's successor round replaces what its superseded round read; an
/// authored stage's reads stay beside it.
#[test]
fn a_successor_round_replaces_its_superseded_rounds_reads() {
    let mut reads = PendingReads::default();
    reads.adopt(ReadOwner::Stage(1), DecisionReads::from([birth(9)]));
    reads.adopt(ReadOwner::Part(0), DecisionReads::from([birth(1)]));
    reads.adopt(ReadOwner::Part(0), DecisionReads::from([birth(2)]));
    assert_eq!(all(&reads), vec![birth(9), birth(2)]);
    reads.adopt(ReadOwner::Part(0), DecisionReads::new());
    assert_eq!(all(&reads), vec![birth(9)], "a round that read nothing");
}

/// Frames nest with nested preparation: each decision keeps only its own.
#[test]
fn nested_decisions_keep_their_own_reads() {
    let mut reads = PendingReads::default();
    reads.begin_decision();
    reads.retain(birth(1));
    reads.begin_decision();
    reads.retain(birth(2));
    assert_eq!(reads.end_decision(), DecisionReads::from([birth(2)]));
    reads.retain(birth(3));
    assert_eq!(
        reads.end_decision(),
        DecisionReads::from([birth(1), birth(3)])
    );
}

/// A read outside any decision frame is never dropped: it is always checked.
/// An `assemble` retry resets everything.
#[test]
fn an_unframed_read_is_always_checked_until_a_reset() {
    let mut reads = PendingReads::default();
    reads.retain(birth(4));
    reads.adopt(ReadOwner::Part(2), DecisionReads::from([birth(5)]));
    assert_eq!(all(&reads), vec![birth(4), birth(5)]);
    reads = PendingReads::default();
    assert!(all(&reads).is_empty());
}
