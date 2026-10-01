//! Postorder, terminality and unconnected-residue regression family.

use super::{validates_drop_result, ExternalBlobFact};
use std::collections::BTreeSet;
use worth_store_physical_format::PersistedRecordIdentity;

#[test]
fn a_parent_drop_cannot_strand_a_live_child_or_claim_false_terminality() {
    let record = |ordinal| PersistedRecordIdentity::new([7; 16], ordinal).unwrap();
    let facts = [
        ExternalBlobFact {
            record: record(1),
            frame_sha256: [1; 32],
            same_session: true,
            edges: vec![(1, record(2))],
        },
        ExternalBlobFact {
            record: record(2),
            frame_sha256: [2; 32],
            same_session: true,
            edges: vec![],
        },
    ];
    let reached = BTreeSet::from([record(1), record(2)]);
    let protected = BTreeSet::new();
    assert!(!validates_drop_result(
        &reached,
        &facts,
        &[record(1)],
        &protected,
        false
    ));
    assert!(validates_drop_result(
        &reached,
        &facts,
        &[record(2)],
        &protected,
        false
    ));
    assert!(!validates_drop_result(
        &reached,
        &facts,
        &[record(2)],
        &protected,
        true
    ));
    assert!(validates_drop_result(
        &reached,
        &facts,
        &[record(1), record(2)],
        &protected,
        true
    ));
    assert!(!validates_drop_result(
        &reached,
        &facts,
        &[record(1), record(2)],
        &protected,
        false
    ));
}

#[test]
fn same_session_residue_outside_the_publication_closure_cannot_be_abandoned() {
    let record = |ordinal| PersistedRecordIdentity::new([8; 16], ordinal).unwrap();
    let facts = [
        ExternalBlobFact {
            record: record(1),
            frame_sha256: [1; 32],
            same_session: true,
            edges: vec![],
        },
        ExternalBlobFact {
            record: record(2),
            frame_sha256: [2; 32],
            same_session: true,
            edges: vec![],
        },
    ];
    let reached = BTreeSet::from([record(1)]);
    assert!(!validates_drop_result(
        &reached,
        &facts,
        &[record(1)],
        &BTreeSet::new(),
        true
    ));
    // Protected disconnected bytes retain an explicit external owner. They
    // are not falsely included in this publication's reachable drop result.
    assert!(validates_drop_result(
        &reached,
        &facts,
        &[record(1)],
        &BTreeSet::from([record(2)]),
        true
    ));
}
