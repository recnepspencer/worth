use super::*;
use crate::integrity_observation::blob_record::{BlobEdge, FrameKind};

fn selected(record: [u8; 24], fact: BlobFact) -> Selected {
    Selected {
        record,
        path: "arena.bin".into(),
        generation: 1,
        kind: Some(FrameKind::of(&fact)),
        fact: Some(fact),
        outcome: Outcome::Intact,
        route: None,
    }
}

fn ordered_digest(reverse: bool) -> [u8; 32] {
    let mut edges = vec![
        BlobEdge {
            digest: [4; 32],
            record: [1; 24],
            covered: 64 << 10,
        },
        BlobEdge {
            digest: [5; 32],
            record: [2; 24],
            covered: 64 << 10,
        },
    ];
    if reverse {
        edges.reverse();
    }
    let selected = vec![
        selected(
            [1; 24],
            BlobFact::Chunk {
                store: [3; 16],
                session: [6; 16],
                ordinal: 0,
                chunk_size: 64 << 10,
                length: 64 << 10,
                digest: [4; 32],
            },
        ),
        selected(
            [2; 24],
            BlobFact::Chunk {
                store: [3; 16],
                session: [6; 16],
                ordinal: 1,
                chunk_size: 64 << 10,
                length: 64 << 10,
                digest: [5; 32],
            },
        ),
        selected(
            [3; 24],
            BlobFact::Node {
                store: [3; 16],
                session: [6; 16],
                kind: 1,
                level: 0,
                index: 0,
                covered: 128 << 10,
                digest: [7; 32],
                frame_digest: [8; 32],
                entries: edges,
            },
        ),
    ];
    let records = selected
        .iter()
        .enumerate()
        .map(|(index, row)| (row.record, index))
        .collect();
    let mut hasher = Sha256::new();
    visit_node([3; 24], &selected, &records, 0, &mut |chunk| {
        hasher.update(&vec![chunk.record[0]; 64 << 10]);
        Ok(())
    })
    .expect("selected tree order");
    hasher.finish()
}

#[test]
fn reversing_tree_edges_changes_portable_logical_digest() {
    assert_ne!(ordered_digest(false), ordered_digest(true));
}

/// One leaf whose single edge names a reuse claim that borrows chunk `[1; 24]`.
fn reused_leaf(source: Option<Selected>) -> Vec<Selected> {
    let claim = selected(
        [2; 24],
        BlobFact::ReuseClaim {
            store: [3; 16],
            session: [6; 16],
            ordinal: 0,
            scope: [9; 32],
            chunk_size: 64 << 10,
            length: 64 << 10,
            digest: [4; 32],
            chunk_record: [1; 24],
            source_publication: [10; 24],
            source_ordinal: 0,
            source_witness: None,
        },
    );
    let leaf = selected(
        [3; 24],
        BlobFact::Node {
            store: [3; 16],
            session: [6; 16],
            kind: 1,
            level: 0,
            index: 0,
            covered: 64 << 10,
            digest: [7; 32],
            frame_digest: [8; 32],
            entries: vec![BlobEdge {
                digest: [4; 32],
                record: [2; 24],
                covered: 64 << 10,
            }],
        },
    );
    source.into_iter().chain([claim, leaf]).collect()
}

fn visited_chunks(rows: &[Selected]) -> Result<Vec<[u8; 24]>, Outcome> {
    let records = rows
        .iter()
        .enumerate()
        .map(|(index, row)| (row.record, index))
        .collect();
    let mut visited = Vec::new();
    visit_node([3; 24], rows, &records, 0, &mut |chunk| {
        assert!(matches!(chunk.fact, Some(BlobFact::Chunk { .. })));
        visited.push(chunk.record);
        Ok(())
    })?;
    Ok(visited)
}

fn borrowed_chunk() -> Selected {
    selected(
        [1; 24],
        BlobFact::Chunk {
            store: [3; 16],
            session: [11; 16],
            ordinal: 0,
            chunk_size: 64 << 10,
            length: 64 << 10,
            digest: [4; 32],
        },
    )
}

#[test]
fn reused_leaf_reads_the_source_chunk_its_claim_borrows() {
    assert_eq!(
        visited_chunks(&reused_leaf(Some(borrowed_chunk()))),
        Ok(vec![[1; 24]])
    );
    assert_eq!(
        visited_chunks(&reused_leaf(None)),
        Err(damage(Cause::Pointer)),
        "a claim whose source chunk is not selected has no bytes"
    );

    let unavailable = Outcome::Unknown(OfflineUnknownPhysicalReason::ParentScopeUnavailable);
    let mut source = borrowed_chunk();
    source.outcome = unavailable.clone();
    assert_eq!(
        visited_chunks(&reused_leaf(Some(source))),
        Err(unavailable),
        "an unavailable source chunk is not pointer damage"
    );

    let mut not_a_chunk = borrowed_chunk();
    not_a_chunk.fact = reused_leaf(None).swap_remove(0).fact;
    assert_eq!(
        visited_chunks(&reused_leaf(Some(not_a_chunk))),
        Err(damage(Cause::Pointer)),
        "a claim borrows a chunk frame, never another claim"
    );
}

#[test]
fn reused_leaf_has_no_bytes_through_a_claim_that_is_not_intact() {
    let claim = 1;
    let mut rows = reused_leaf(Some(borrowed_chunk()));
    assert!(matches!(
        rows[claim].fact,
        Some(BlobFact::ReuseClaim { .. })
    ));
    rows[claim].outcome = damage(Cause::ScopeMismatch);
    assert_eq!(
        visited_chunks(&rows),
        Err(damage(Cause::Pointer)),
        "a damaged claim borrows nothing, even from an intact source chunk"
    );

    let unavailable = Outcome::Unknown(OfflineUnknownPhysicalReason::WalCoverageUnavailable);
    rows[claim].outcome = unavailable.clone();
    assert_eq!(
        visited_chunks(&rows),
        Err(unavailable),
        "an unavailable claim is not pointer damage"
    );
}

#[test]
fn bounded_child_does_not_become_pointer_damage_in_logical_walk() {
    let child = Selected {
        record: [1; 24],
        path: "arena.bin".into(),
        generation: 1,
        kind: None,
        fact: None,
        outcome: Outcome::Indeterminate(
            crate::integrity_observation::OfflineIndeterminatePhysicalReason::EntryBoundExceeded,
        ),
        route: None,
    };
    let root = selected(
        [2; 24],
        BlobFact::Node {
            store: [3; 16],
            session: [4; 16],
            kind: 1,
            level: 0,
            index: 0,
            covered: 64 << 10,
            digest: [5; 32],
            frame_digest: [6; 32],
            entries: vec![BlobEdge {
                digest: [7; 32],
                record: [1; 24],
                covered: 64 << 10,
            }],
        },
    );
    let rows = [child, root];
    let records = rows
        .iter()
        .enumerate()
        .map(|(index, row)| (row.record, index))
        .collect();
    let outcome = visit_node([2; 24], &rows, &records, 0, &mut |_| Ok(()));
    assert_eq!(
        outcome,
        Err(Outcome::Indeterminate(
            crate::integrity_observation::OfflineIndeterminatePhysicalReason::EntryBoundExceeded,
        ))
    );
}
