use super::super::blob_record::BlobEdge;
use super::*;
use crate::integrity_observation::OfflineIntegrityObservationLimits;
use crate::integrity_observation::{BoundedMediaWalk, OfflineUnknownPhysicalReason};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use worth_foundational::PhysicalArtifactFamily as Family;

#[test]
fn bounded_second_extent_frame_keeps_blob_indeterminate_at_finish() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "worth-blob-second-frame-bound-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&root).unwrap();
    let root = std::fs::canonicalize(root).unwrap();
    let path = root.join("arena");
    let mut first_frame = vec![0_u8; 112];
    let mut prefix = vec![0_u8; 32];
    prefix[..8].copy_from_slice(b"WRC11BLB");
    prefix[8] = 2; // A chunk frame, whose remaining bytes are in frame two.
    first_frame.extend_from_slice(&prefix);
    let mut media = first_frame.clone();
    media.extend_from_slice(&[0_u8; 64]);
    std::fs::write(&path, media).unwrap();

    let limits = OfflineIntegrityObservationLimits::new(
        8,
        first_frame.len() as u64,
        if cfg!(windows) { 6 } else { 5 },
        4,
        0,
        10_000,
        4096,
    )
    .unwrap();
    let mut walk = BoundedMediaWalk::new(limits, root.clone(), Instant::now());
    let expected = |offset, length, logical_offset, ordinal| ChildExpectation {
        path: "arena".into(),
        family: Family::ExtentChunkFrame,
        generation: 1,
        format: [0; 10],
        offset,
        length: Some(length),
        checksum: None,
        scope: ChildScope::ExtentChunk {
            arena: 1,
            extent: 1,
            record: [7; 24],
            logical_bytes: 96,
            logical_offset,
            ordinal,
        },
    };
    let first = expected(0, first_frame.len() as u64, 0, 0);
    let acquired = walk
        .acquire_range(&path, 4, first.offset, first.length.unwrap())
        .unwrap();
    let mut blobs = BlobRecordWalk::new(8, SelectedCheckpointEvidence::Absent);
    blobs.observe_valid_extent_chunk(&first, &acquired.bytes, Some([1; 16]), walk.counters_mut());
    let second = expected(first_frame.len() as u64, 64, 32, 1);
    let outcome = walk
        .acquire_range(&path, 4, second.offset, second.length.unwrap())
        .unwrap_err();
    assert_eq!(
        outcome,
        Outcome::Indeterminate(OfflineIndeterminatePhysicalReason::ByteBoundExceeded)
    );
    blobs.note_outcome(&second, &outcome);
    // A later, unrelated queue bound must not relabel this frame's byte bound.
    assert_eq!(
        walk.entry_bound(),
        Outcome::Indeterminate(OfflineIndeterminatePhysicalReason::EntryBoundExceeded)
    );
    let rows = blobs.finish(&root, &mut walk, false);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].family(), Family::BlobChunkFrame);
    assert_eq!(rows[0].outcome(), &outcome);

    let mut aliased = BlobRecordWalk::new(8, SelectedCheckpointEvidence::Absent);
    aliased.pending = Some(Pending {
        record: [7; 24],
        logical_bytes: 96,
        path: "arena".into(),
        generation: 1,
        bytes: prefix,
        route: ExtentRoute {
            format: [0; 10],
            arena: 1,
            extent: 1,
            logical_bytes: 96,
            frames: vec![(0, first_frame.len() as u64)],
        },
        interruption: None,
    });
    let unavailable = Outcome::Unknown(OfflineUnknownPhysicalReason::PhysicalAliasNotReinspected);
    aliased.note_outcome(&second, &unavailable);
    let rows = aliased.finish(&root, &mut walk, false);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].outcome(), &unavailable);

    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(root).unwrap();
}

#[test]
fn node_graph_edge_retention_has_distinct_declared_entry_cap() {
    let mut walk = BlobRecordWalk::new(0, SelectedCheckpointEvidence::Absent);
    let fact = BlobFact::Node {
        store: [1; 16],
        session: [2; 16],
        kind: 1,
        level: 0,
        index: 0,
        covered: 1,
        digest: [3; 32],
        frame_digest: [6; 32],
        entries: vec![BlobEdge {
            digest: [4; 32],
            record: [5; 24],
            covered: 1,
        }],
    };
    let (retained, outcome) = walk.admit_fact(fact);
    assert!(retained.is_none());
    assert_eq!(
        outcome,
        Outcome::Indeterminate(OfflineIndeterminatePhysicalReason::EntryBoundExceeded,)
    );
    assert_eq!(walk.retained_graph_edges, 0);
}

fn selected(record: [u8; 24], fact: BlobFact) -> Selected {
    Selected {
        record,
        path: "arena.bin".into(),
        generation: 1,
        kind: Some(blob_record::FrameKind::of(&fact)),
        fact: Some(fact),
        outcome: Outcome::Intact,
        route: None,
    }
}

fn one_chunk_claim_graph(chunk_session: [u8; 16], root_digest: [u8; 32]) -> BlobRecordWalk {
    let mut walk = BlobRecordWalk::new(10, SelectedCheckpointEvidence::Absent);
    walk.selected = vec![
        selected(
            [1; 24],
            BlobFact::Declaration {
                store: [1; 16],
                session: [2; 16],
                frame_digest: [11; 32],
                object: [3; 16],
                scope: [4; 32],
                chunk_size: 64 << 10,
                total: 64 << 10,
                max_checkpoint_sequence: 5,
            },
        ),
        selected(
            [5; 24],
            BlobFact::Chunk {
                store: [1; 16],
                session: chunk_session,
                ordinal: 0,
                chunk_size: 64 << 10,
                length: 64 << 10,
                digest: [6; 32],
            },
        ),
        selected(
            [7; 24],
            BlobFact::Node {
                store: [1; 16],
                session: [2; 16],
                kind: 1,
                level: 0,
                index: 0,
                covered: 64 << 10,
                digest: [8; 32],
                frame_digest: [9; 32],
                entries: vec![BlobEdge {
                    digest: [6; 32],
                    record: [5; 24],
                    covered: 64 << 10,
                }],
            },
        ),
        selected(
            [10; 24],
            BlobFact::Publication {
                store: [1; 16],
                frame_digest: [14; 32],
                session: [2; 16],
                object: [3; 16],
                generation: 1,
                root: [7; 24],
                root_digest,
                total: 64 << 10,
                logical_digest: [11; 32],
                chunk_size: 64 << 10,
                scope: [4; 32],
            },
        ),
    ];
    walk
}

#[test]
fn publication_requires_full_root_frame_sha_not_interior_content_digest() {
    let mut wrong = one_chunk_claim_graph([2; 16], [8; 32]);
    wrong.validate_selected_claim_graph();
    assert!(matches!(wrong.selected[3].outcome, Outcome::Damaged(_)));
    let mut right = one_chunk_claim_graph([2; 16], [9; 32]);
    right.validate_selected_claim_graph();
    assert_eq!(right.selected[3].outcome, Outcome::Intact);
}

#[test]
fn changed_occurrence_is_not_rescued_by_valid_content_digest() {
    let mut wrong = one_chunk_claim_graph([12; 16], [9; 32]);
    wrong.validate_selected_claim_graph();
    assert!(matches!(wrong.selected[1].outcome, Outcome::Damaged(_)));
    assert!(matches!(wrong.selected[2].outcome, Outcome::Damaged(_)));
}

#[test]
fn graph_edge_budget_preserves_indeterminate_publication_instead_of_pointer_damage() {
    let mut walk = one_chunk_claim_graph([2; 16], [9; 32]);
    let node = walk.selected[2].fact.take().expect("synthetic root node");
    let mut bounded = BlobRecordWalk::new(0, SelectedCheckpointEvidence::Absent);
    let (retained, outcome) = bounded.admit_fact(node);
    assert!(retained.is_none());
    assert_eq!(
        outcome,
        Outcome::Indeterminate(OfflineIndeterminatePhysicalReason::EntryBoundExceeded)
    );
    walk.selected[2].fact = retained;
    walk.selected[2].outcome = outcome.clone();
    walk.validate_selected_claim_graph();
    assert_eq!(walk.selected[2].outcome, outcome);
    assert_eq!(walk.selected[3].outcome, outcome);
}

#[test]
fn unavailable_root_preserves_unknown_publication_instead_of_pointer_damage() {
    let mut walk = one_chunk_claim_graph([2; 16], [9; 32]);
    walk.selected[2].outcome =
        Outcome::Unknown(OfflineUnknownPhysicalReason::ParentScopeUnavailable);
    walk.validate_selected_claim_graph();
    assert_eq!(
        walk.selected[3].outcome,
        Outcome::Unknown(OfflineUnknownPhysicalReason::ParentScopeUnavailable)
    );
}

fn record_for(index: u64) -> [u8; 24] {
    let mut record = [1_u8; 24];
    record[16..].copy_from_slice(&index.to_le_bytes());
    record
}

#[test]
fn selected_graph_uses_four_thousand_ninety_six_entry_leaf_boundary() {
    const CHUNK: u64 = 64 << 10;
    let mut walk = BlobRecordWalk::new(4099, SelectedCheckpointEvidence::Absent);
    walk.selected.push(selected(
        record_for(5000),
        BlobFact::Declaration {
            store: [1; 16],
            session: [2; 16],
            frame_digest: [11; 32],
            object: [3; 16],
            scope: [4; 32],
            chunk_size: CHUNK as u32,
            total: 4097 * CHUNK,
            max_checkpoint_sequence: 5,
        },
    ));
    for ordinal in 0..4097 {
        walk.selected.push(selected(
            record_for(ordinal + 1),
            BlobFact::Chunk {
                store: [1; 16],
                session: [2; 16],
                ordinal,
                chunk_size: CHUNK as u32,
                length: CHUNK,
                digest: [6; 32],
            },
        ));
    }
    for (index, count, record, digest) in [
        (0_u64, 4096_u64, record_for(5001), [7; 32]),
        (1_u64, 1_u64, record_for(5002), [8; 32]),
    ] {
        let entries = (0..count)
            .map(|position| BlobEdge {
                digest: [6; 32],
                record: record_for(index * 4096 + position + 1),
                covered: CHUNK,
            })
            .collect();
        let node = BlobFact::Node {
            store: [1; 16],
            session: [2; 16],
            kind: 1,
            level: 0,
            index,
            covered: count * CHUNK,
            digest,
            frame_digest: [9; 32],
            entries,
        };
        let (retained, outcome) = walk.admit_fact(node);
        assert_eq!(outcome, Outcome::Intact);
        walk.selected.push(selected(record, retained.unwrap()));
    }
    let root = BlobFact::Node {
        store: [1; 16],
        session: [2; 16],
        kind: 2,
        level: 1,
        index: 0,
        covered: 4097 * CHUNK,
        digest: [10; 32],
        frame_digest: [12; 32],
        entries: vec![
            BlobEdge {
                digest: [7; 32],
                record: record_for(5001),
                covered: 4096 * CHUNK,
            },
            BlobEdge {
                digest: [8; 32],
                record: record_for(5002),
                covered: CHUNK,
            },
        ],
    };
    let (retained, outcome) = walk.admit_fact(root);
    assert_eq!(outcome, Outcome::Intact);
    assert_eq!(walk.retained_graph_edges, 4099);
    walk.selected
        .push(selected(record_for(5003), retained.unwrap()));
    walk.selected.push(selected(
        record_for(5004),
        BlobFact::Publication {
            store: [1; 16],
            frame_digest: [14; 32],
            session: [2; 16],
            object: [3; 16],
            generation: 1,
            root: record_for(5003),
            root_digest: [12; 32],
            total: 4097 * CHUNK,
            logical_digest: [13; 32],
            chunk_size: CHUNK as u32,
            scope: [4; 32],
        },
    ));
    walk.validate_selected_claim_graph();
    assert!(walk
        .selected
        .iter()
        .all(|row| row.outcome == Outcome::Intact));
}
