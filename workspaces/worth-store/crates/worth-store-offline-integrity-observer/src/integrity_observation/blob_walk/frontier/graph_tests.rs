use super::super::{BlobRecordWalk, Selected};
use super::*;
use crate::integrity_observation::{
    OfflineIndeterminatePhysicalReason, OfflineUnknownPhysicalReason,
};

const STORE: [u8; 16] = [1; 16];
const SESSION: [u8; 16] = [2; 16];
const CHUNK: u64 = 64 << 10;
const FRONTIER_RECORD: [u8; 24] = [9; 24];

fn row(record: [u8; 24], fact: BlobFact, outcome: Outcome) -> Selected {
    Selected {
        record,
        path: "arena".into(),
        generation: 1,
        family: fact.family().into(),
        fact: Some(fact),
        outcome,
        route: None,
    }
}

fn declaration(total: u64) -> Selected {
    row(
        [3; 24],
        BlobFact::Declaration {
            store: STORE,
            session: SESSION,
            frame_digest: [4; 32],
            object: [5; 16],
            scope: [6; 32],
            chunk_size: CHUNK as u32,
            total,
            max_checkpoint_sequence: 5,
        },
        Outcome::Intact,
    )
}

fn chunk(record: [u8; 24], ordinal: u64, digest: [u8; 32], outcome: Outcome) -> Selected {
    row(
        record,
        BlobFact::Chunk {
            store: STORE,
            session: SESSION,
            ordinal,
            chunk_size: CHUNK as u32,
            length: CHUNK,
            digest,
        },
        outcome,
    )
}

fn frontier(next: u64, bytes: u64, last: [u8; 24], digest: [u8; 32]) -> Selected {
    row(
        FRONTIER_RECORD,
        BlobFact::Frontier {
            store: STORE,
            session: SESSION,
            declaration_record: [3; 24],
            declaration_digest: [4; 32],
            next_chunk_ordinal: next,
            durable_bytes: bytes,
            last_chunk_record: last,
            last_chunk_digest: digest,
        },
        Outcome::Intact,
    )
}

fn validated(rows: Vec<Selected>) -> Vec<Selected> {
    let mut walk = BlobRecordWalk {
        checkpoint: crate::integrity_observation::journal_walk::SelectedCheckpointEvidence::Absent,
        pending: None,
        selected: rows,
        maximum_graph_edges: 16,
        retained_graph_edges: 0,
        selected_root_generation: None,
        historical_source: None,
        routes: crate::integrity_observation::record_walk::route_inventory::RouteInventory::new(),
    };
    walk.validate_selected_claim_graph();
    walk.selected
}

fn frontier_outcome(rows: &[Selected]) -> &Outcome {
    &rows
        .iter()
        .find(|row| row.record == FRONTIER_RECORD)
        .unwrap()
        .outcome
}

#[test]
fn shuffled_selected_rows_validate_chunks_before_frontier_prefix() {
    let rows = validated(vec![
        frontier(2, 2 * CHUNK, [8; 24], [10; 32]),
        chunk([8; 24], 1, [10; 32], Outcome::Intact),
        declaration(2 * CHUNK),
        chunk([7; 24], 0, [11; 32], Outcome::Intact),
    ]);
    assert_eq!(frontier_outcome(&rows), &Outcome::Intact);

    let invalid = validated(vec![
        frontier(2, 2 * CHUNK, [8; 24], [10; 32]),
        chunk([8; 24], 1, [10; 32], Outcome::Intact),
        declaration(CHUNK),
        chunk([7; 24], 0, [11; 32], Outcome::Intact),
    ]);
    assert!(matches!(invalid[1].outcome, Outcome::Damaged(_)));
    assert!(matches!(frontier_outcome(&invalid), Outcome::Damaged(_)));
}

#[test]
fn duplicate_endpoint_ordinal_cannot_hide_behind_first_matching_record() {
    let rows = validated(vec![
        frontier(1, CHUNK, [7; 24], [11; 32]),
        chunk([7; 24], 0, [11; 32], Outcome::Intact),
        declaration(CHUNK),
        chunk([8; 24], 0, [12; 32], Outcome::Intact),
    ]);
    assert!(matches!(frontier_outcome(&rows), Outcome::Damaged(_)));

    // Re-observation of the identical selected record is not a second claim.
    let rows = validated(vec![
        frontier(1, CHUNK, [7; 24], [11; 32]),
        chunk([7; 24], 0, [11; 32], Outcome::Intact),
        declaration(CHUNK),
        chunk([7; 24], 0, [11; 32], Outcome::Intact),
    ]);
    assert_eq!(frontier_outcome(&rows), &Outcome::Intact);
}

#[test]
fn nonlast_unavailable_prefix_claim_does_not_become_pointer_damage() {
    for unavailable in [
        Outcome::Unknown(OfflineUnknownPhysicalReason::ParentScopeUnavailable),
        Outcome::Indeterminate(OfflineIndeterminatePhysicalReason::EntryBoundExceeded),
    ] {
        let rows = validated(vec![
            frontier(2, 2 * CHUNK, [8; 24], [10; 32]),
            chunk([8; 24], 1, [10; 32], Outcome::Intact),
            declaration(2 * CHUNK),
            chunk([7; 24], 0, [11; 32], unavailable.clone()),
        ]);
        assert_eq!(frontier_outcome(&rows), &unavailable);
    }
}
