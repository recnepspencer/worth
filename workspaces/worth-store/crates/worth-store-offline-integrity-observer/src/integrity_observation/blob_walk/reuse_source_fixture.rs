//! A one-chunk source publication and a destination session that reuses its
//! chunk: the selected rows a reuse claim and a dedupe quarantine are judged
//! against.

use super::super::blob_record::{BlobEdge, BlobFact, FrameKind, ReuseSourceWitness};
use super::super::journal_walk::SelectedCheckpointEvidence;
use super::{BlobRecordWalk, ChildExpectation, ChildScope, Outcome, Selected};
use worth_foundational::PhysicalArtifactFamily as Family;

pub(super) const STORE: [u8; 16] = [1; 16];
pub(super) const SCOPE: [u8; 32] = [3; 32];
pub(super) const DIGEST: [u8; 32] = [4; 32];
pub(super) const SOURCE: [u8; 16] = [5; 16];
pub(super) const DESTINATION: [u8; 16] = [6; 16];
/// A third session, selected once `declare_other` declares it.
pub(super) const OTHER: [u8; 16] = [20; 16];
pub(super) const CHUNK: u32 = 64 << 10;
const OBJECT: [u8; 16] = [8; 16];
const LEAF_FRAME: [u8; 32] = [10; 32];
const PUBLICATION_FRAME: [u8; 32] = [14; 32];

/// Row positions in `selected_source`; each row's record is `record(position + 1)`.
pub(super) const SOURCE_DECLARATION: usize = 0;
pub(super) const SOURCE_CHUNK: usize = 1;
pub(super) const SOURCE_LEAF: usize = 2;
pub(super) const SOURCE_PUBLICATION: usize = 3;
pub(super) const DESTINATION_DECLARATION: usize = 4;
pub(super) const CLAIM: usize = 5;

/// Row positions in `released_source`, whose rows before these are the
/// source's declaration, chunk and leaf and the destination's declaration.
pub(super) const RELEASED_CLAIM: usize = 4;
pub(super) const RELEASE_MANIFEST: usize = 5;
pub(super) const RELEASE_DESCRIPTOR: usize = 6;

pub(super) fn record(ordinal: u64) -> [u8; 24] {
    let mut bytes = [1; 24];
    bytes[16..].copy_from_slice(&ordinal.to_le_bytes());
    bytes
}

/// An intact row whose record and placement generation are both `ordinal`.
pub(super) fn row(ordinal: u64, fact: BlobFact) -> Selected {
    Selected {
        record: record(ordinal),
        path: String::new(),
        generation: ordinal,
        kind: Some(FrameKind::of(&fact)),
        fact: Some(fact),
        outcome: Outcome::Intact,
        route: None,
    }
}

fn declaration(session: [u8; 16], object: [u8; 16], frame_digest: [u8; 32]) -> BlobFact {
    BlobFact::Declaration {
        store: STORE,
        session,
        frame_digest,
        object,
        scope: SCOPE,
        chunk_size: CHUNK,
        total: u64::from(CHUNK),
        max_checkpoint_sequence: 9,
    }
}

/// Declare other bytes for the session of the declaration at `position`.
pub(super) fn declare_total(rows: &mut [Selected], position: usize, bytes: u64) {
    if let Some(BlobFact::Declaration { total, .. }) = rows[position].fact.as_mut() {
        *total = bytes;
    }
}

/// Declare `OTHER`, so that a chunk of it is that session's own intact
/// occurrence and only a row that names it for another session is wrong.
pub(super) fn declare_other(rows: &mut Vec<Selected>) {
    rows.push(row(20, declaration(OTHER, [22; 16], [23; 32])));
}

/// The rows as the claim graph leaves them: every table enters through the
/// walk, so no step of it can decide a row before the proof under test.
pub(super) fn walked(rows: Vec<Selected>) -> Vec<Selected> {
    graphed(walk_over(rows)).selected
}

/// A walk that read `rows` and visited every routed record, before its claim
/// graph: a test says through it what the walk could not read or did not visit.
pub(super) fn walk_over(rows: Vec<Selected>) -> BlobRecordWalk {
    let mut walk = BlobRecordWalk::new(64, SelectedCheckpointEvidence::Absent);
    walk.selected = rows;
    walk
}

/// The walk once its claim graph has judged its rows.
pub(super) fn graphed(mut walk: BlobRecordWalk) -> BlobRecordWalk {
    walk.validate_selected_claim_graph();
    walk
}

/// The frame of `record` that the selected root routes at `logical_offset`.
pub(super) fn frame(record: [u8; 24], logical_offset: u64) -> ChildExpectation {
    ChildExpectation {
        path: "arena".into(),
        family: Family::ExtentChunkFrame,
        generation: 1,
        format: [0; 10],
        offset: logical_offset,
        length: Some(144),
        checksum: None,
        scope: ChildScope::ExtentChunk {
            arena: 1,
            extent: 1,
            record,
            logical_bytes: 288,
            logical_offset,
            ordinal: u32::from(logical_offset != 0),
        },
    }
}

/// The extent manifest of `record` that the selected root routes.
pub(super) fn manifest(record: [u8; 24]) -> ChildExpectation {
    ChildExpectation {
        family: Family::ExtentManifest,
        scope: ChildScope::ExtentManifest {
            arena: 1,
            extent: 1,
            record,
            logical_bytes: 288,
            allocated_bytes: 288,
        },
        ..frame(record, 0)
    }
}

/// The row of a record whose frame could not be read, of the kind `code`
/// when its route or its first bytes declare one.
pub(super) fn unread(record: [u8; 24], code: Option<u8>, outcome: Outcome) -> Selected {
    let kind = code.map(|code| FrameKind::declared(code).expect("a declared kind"));
    Selected {
        record,
        path: "arena".into(),
        generation: 1,
        kind,
        fact: None,
        outcome,
        route: None,
    }
}

/// Every set of `rows`, the empty one first.
pub(super) fn subsets(rows: &[usize]) -> impl Iterator<Item = Vec<usize>> + '_ {
    (0..1_usize << rows.len()).map(|chosen| {
        let chosen = |bit: &usize| chosen >> bit & 1 == 1;
        (0..rows.len())
            .filter(chosen)
            .map(|bit| rows[bit])
            .collect()
    })
}

/// A chunk frame of `session`.
pub(super) fn chunk(session: [u8; 16], ordinal: u64, length: u64, digest: [u8; 32]) -> BlobFact {
    BlobFact::Chunk {
        store: STORE,
        session,
        ordinal,
        chunk_size: CHUNK,
        length,
        digest,
    }
}

/// The source publication as a claim carries it.
pub(super) fn witness() -> ReuseSourceWitness {
    ReuseSourceWitness {
        frame_digest: PUBLICATION_FRAME,
        store: STORE,
        session: SOURCE,
        object: OBJECT,
        generation: 1,
        root: record(3),
        root_digest: LEAF_FRAME,
        total: u64::from(CHUNK),
        chunk_size: CHUNK,
        scope: SCOPE,
    }
}

/// The destination's claim on ordinal 0, borrowing the source chunk.
pub(super) fn claim(source_witness: Option<ReuseSourceWitness>) -> BlobFact {
    BlobFact::ReuseClaim {
        store: STORE,
        session: DESTINATION,
        ordinal: 0,
        scope: SCOPE,
        chunk_size: CHUNK,
        length: u64::from(CHUNK),
        digest: DIGEST,
        chunk_record: record(2),
        source_publication: record(4),
        source_ordinal: 0,
        source_witness,
    }
}

pub(super) fn selected_source() -> Vec<Selected> {
    vec![
        row(1, declaration(SOURCE, OBJECT, [7; 32])),
        row(2, chunk(SOURCE, 0, u64::from(CHUNK), DIGEST)),
        row(
            3,
            BlobFact::Node {
                store: STORE,
                session: SOURCE,
                kind: 1,
                level: 0,
                index: 0,
                covered: u64::from(CHUNK),
                digest: [9; 32],
                frame_digest: LEAF_FRAME,
                entries: vec![BlobEdge {
                    digest: DIGEST,
                    record: record(2),
                    covered: u64::from(CHUNK),
                }],
            },
        ),
        row(
            4,
            BlobFact::Publication {
                store: STORE,
                frame_digest: PUBLICATION_FRAME,
                session: SOURCE,
                object: OBJECT,
                generation: 1,
                root: record(3),
                root_digest: LEAF_FRAME,
                total: u64::from(CHUNK),
                logical_digest: [11; 32],
                chunk_size: CHUNK,
                scope: SCOPE,
            },
        ),
        row(5, declaration(DESTINATION, [13; 16], [12; 32])),
        row(6, claim(None)),
    ]
}

/// The same store after the source publication was released: the claim carries
/// the publication as its witness, and the release's manifest and first
/// descriptor are selected in the publication's place.
pub(super) fn released_source() -> Vec<Selected> {
    let mut rows = selected_source();
    rows[CLAIM] = row(6, claim(Some(witness())));
    rows.remove(SOURCE_PUBLICATION);
    rows.push(row(
        7,
        BlobFact::ReleasedDropSetManifest {
            store: STORE,
            frame_digest: [15; 32],
            attempt: [16; 16],
            object: OBJECT,
            session: SOURCE,
            generation: 1,
            root: record(3),
            root_digest: LEAF_FRAME,
            publication_record: record(4),
            publication_digest: PUBLICATION_FRAME,
            issuer_evidence_digest: [18; 32],
            basis_digest: [17; 32],
            dropped: vec![record(4)],
            never_reserved_slot_generation: 5,
        },
    ));
    rows.push(row(
        8,
        BlobFact::ReleasedReclaimDescriptor {
            store: STORE,
            frame_digest: [19; 32],
            attempt: [16; 16],
            basis_digest: [17; 32],
            manifest_record: record(7),
            manifest_digest: [15; 32],
            manifest_count: 1,
            source_root: 6,
            candidate_root: 7,
            predecessor: None,
            cumulative_dropped: 1,
            terminal: false,
            custody: None,
        },
    ));
    rows
}
