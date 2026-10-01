use super::*;
use crate::integrity_observation::blob_record::BlobEdge;
use crate::integrity_observation::OfflineArtifactFamily;

fn selected(record: [u8; 24], fact: BlobFact) -> Selected {
    Selected {
        record,
        path: "arena.bin".into(),
        generation: 1,
        family: fact.family(),
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

#[test]
fn bounded_child_does_not_become_pointer_damage_in_logical_walk() {
    let child = Selected {
        record: [1; 24],
        path: "arena.bin".into(),
        generation: 1,
        family: OfflineArtifactFamily::Unrecognized,
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
