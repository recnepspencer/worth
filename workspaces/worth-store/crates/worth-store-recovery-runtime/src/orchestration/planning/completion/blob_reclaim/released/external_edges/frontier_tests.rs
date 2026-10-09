//! Resume frontiers of the released session against one of another owner.

use super::{admits_dropped_records, audit, validates_drop_result, ExternalBlobFact};
use std::collections::BTreeSet;
use worth_store_physical_format::PersistedRecordIdentity;

const CHUNK: u64 = 1;
const FRONTIER: u64 = 2;
const PUBLICATION: u64 = 9;

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([6; 16], ordinal).unwrap()
}

/// A chunk of the released session.
fn chunk(ordinal: u64) -> ExternalBlobFact {
    ExternalBlobFact {
        record: record(ordinal),
        frame_sha256: [ordinal as u8; 32],
        same_session: true,
        own_frontier: false,
        edges: vec![],
    }
}

/// A selected frontier naming the chunk `names`, written by the released
/// session or by another one.
fn frontier(ordinal: u64, names: u64, own_frontier: bool) -> ExternalBlobFact {
    ExternalBlobFact {
        record: record(ordinal),
        frame_sha256: [ordinal as u8; 32],
        same_session: false,
        own_frontier,
        edges: vec![(9, record(names))],
    }
}

/// One reached chunk and one selected frontier that names it.
fn facts(own_frontier: bool) -> [ExternalBlobFact; 2] {
    [chunk(CHUNK), frontier(FRONTIER, CHUNK, own_frontier)]
}

#[test]
fn only_a_frontier_of_another_session_protects_the_chunk_it_names() {
    let reached = BTreeSet::from([record(CHUNK)]);
    let own = audit(record(PUBLICATION), &reached, &facts(true)).unwrap();
    let other = audit(record(PUBLICATION), &reached, &facts(false)).unwrap();
    assert!(own.protected.is_empty());
    assert_eq!(other.protected, reached);
    // The audited rows do not depend on who wrote the frontier.
    assert_eq!(own.digest, other.digest);
    assert!(!own.publication_referenced && !other.publication_referenced);
}

#[test]
fn a_frontier_of_the_released_session_leaves_before_its_chunks_and_before_terminality() {
    let reached = BTreeSet::from([record(CHUNK)]);
    let facts = facts(true);
    let none = BTreeSet::new();
    let validates =
        |dropped: &[_], terminal| validates_drop_result(&reached, &facts, dropped, &none, terminal);
    assert!(validates(&[record(FRONTIER)], false));
    assert!(validates(&[record(CHUNK), record(FRONTIER)], true));
    // A frontier that stays selected is still owed by the release.
    assert!(!validates(&[record(FRONTIER)], true));
    assert!(!validates(&[record(CHUNK), record(FRONTIER)], false));
    // A chunk may not leave under a frontier that stays selected.
    assert!(!validates(&[record(CHUNK)], false));
    assert!(!validates(&[record(CHUNK)], true));
}

#[test]
fn a_released_drop_admits_its_own_frontier_and_no_other_record_outside_the_closure() {
    let reached = BTreeSet::from([record(CHUNK)]);
    let none = BTreeSet::new();
    let dropped = [record(CHUNK), record(FRONTIER), record(PUBLICATION)];
    let admits = |facts: &[ExternalBlobFact], protected: &BTreeSet<_>| {
        admits_dropped_records(record(PUBLICATION), &reached, facts, &dropped, protected)
    };
    assert!(admits(&facts(true), &none));
    assert!(!admits(&facts(false), &none));
    assert!(!admits(&facts(true), &reached));
}

/// The frontier is the only selected record the release still owes: a batch
/// that keeps it has not reached the terminal head.
#[test]
fn a_release_is_not_terminal_while_only_its_own_frontier_stays_selected() {
    let facts = [frontier(FRONTIER, CHUNK, true)];
    let none = BTreeSet::new();
    let validates =
        |dropped: &[_], terminal| validates_drop_result(&none, &facts, dropped, &none, terminal);
    assert!(validates(&[record(PUBLICATION)], false));
    assert!(!validates(&[record(PUBLICATION)], true));
    assert!(validates(&[record(FRONTIER), record(PUBLICATION)], true));
    assert!(!validates(&[record(FRONTIER), record(PUBLICATION)], false));
}

#[test]
fn every_frontier_of_the_released_session_leaves_before_its_first_chunk() {
    let validates = |facts: &[ExternalBlobFact], dropped: &[u64], terminal| {
        let reached = BTreeSet::from([record(1), record(2)]);
        let dropped: Vec<_> = dropped.iter().copied().map(record).collect();
        validates_drop_result(&reached, facts, &dropped, &BTreeSet::new(), terminal)
    };
    let both = [
        chunk(1),
        chunk(2),
        frontier(3, 1, true),
        frontier(4, 2, true),
    ];
    let reached = BTreeSet::from([record(1), record(2)]);
    let audited = audit(record(PUBLICATION), &reached, &both).unwrap();
    assert!(audited.protected.is_empty());
    // A batch too small for both frontiers takes either one and no chunk.
    assert!(validates(&both, &[3], false));
    assert!(validates(&both, &[4], false));
    assert!(validates(&both, &[3, 4], false));
    // No chunk leaves under a frontier that stays, whichever chunk it names.
    assert!(!validates(&both, &[1, 3], false));
    assert!(!validates(&both, &[2, 3], false));
    assert!(!validates(&both, &[1, 4], false));
    assert!(!validates(&both, &[1, 2, 3], true));
    assert!(validates(&both, &[1, 3, 4], false));
    assert!(validates(&both, &[1, 2, 3, 4], true));
    assert!(!validates(&both, &[1, 2, 3, 4], false));
    assert!(!validates(&both, &[3, 4], true));
    // An earlier batch took the first frontier: the second holds every chunk.
    let second = [chunk(1), chunk(2), frontier(4, 2, true)];
    assert!(validates(&second, &[4], false));
    assert!(!validates(&second, &[1], false));
    assert!(validates(&second, &[1, 4], false));
    assert!(validates(&second, &[1, 2, 4], true));
}
