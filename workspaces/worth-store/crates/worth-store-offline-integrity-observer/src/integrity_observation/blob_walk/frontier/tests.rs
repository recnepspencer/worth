use super::super::coverage::Coverage;
use super::super::reuse_source_fixture::unread;
use super::*;
use crate::integrity_observation::OfflineIndeterminatePhysicalReason as Indeterminate;
use crate::integrity_observation::OfflineUnknownPhysicalReason as Unknown;

fn row(record: [u8; 24], fact: BlobFact) -> Selected {
    Selected {
        record,
        path: "arena".into(),
        generation: 1,
        kind: Some(crate::integrity_observation::blob_record::FrameKind::of(
            &fact,
        )),
        fact: Some(fact),
        outcome: Outcome::Intact,
        route: None,
    }
}

fn claim() -> Claim {
    Claim {
        store: [1; 16],
        session: [2; 16],
        declaration_record: [3; 24],
        declaration_digest: [4; 32],
        next_chunk_ordinal: 2,
        durable_bytes: 128 << 10,
        last_chunk_record: [6; 24],
        last_chunk_digest: [8; 32],
    }
}

fn selected() -> Vec<Selected> {
    vec![
        row(
            [3; 24],
            BlobFact::Declaration {
                store: [1; 16],
                session: [2; 16],
                frame_digest: [4; 32],
                object: [9; 16],
                scope: [10; 32],
                chunk_size: 64 << 10,
                total: 128 << 10,
                max_checkpoint_sequence: 5,
            },
        ),
        row(
            [5; 24],
            BlobFact::Chunk {
                store: [1; 16],
                session: [2; 16],
                ordinal: 0,
                chunk_size: 64 << 10,
                length: 64 << 10,
                digest: [7; 32],
            },
        ),
        row(
            [6; 24],
            BlobFact::Chunk {
                store: [1; 16],
                session: [2; 16],
                ordinal: 1,
                chunk_size: 64 << 10,
                length: 64 << 10,
                digest: [8; 32],
            },
        ),
        row(
            [11; 24],
            BlobFact::Frontier {
                store: [1; 16],
                session: [2; 16],
                declaration_record: [3; 24],
                declaration_digest: [4; 32],
                next_chunk_ordinal: 2,
                durable_bytes: 128 << 10,
                last_chunk_record: [6; 24],
                last_chunk_digest: [8; 32],
            },
        ),
    ]
}

/// The claim index of a walk that visited every routed record.
fn claims(rows: &[Selected], maximum_claims: u64) -> Option<ClaimIndex> {
    let found = RowIndex::new(rows, &Coverage::Complete);
    ClaimIndex::new(rows, &found, maximum_claims)
}

fn record_index(rows: &[Selected]) -> BTreeMap<[u8; 24], usize> {
    rows.iter()
        .enumerate()
        .map(|(index, row)| (row.record, index))
        .collect()
}

#[test]
fn selected_frontier_requires_complete_ordered_prefix_and_exact_dependencies() {
    let rows = selected();
    let index = claims(&rows, 2).expect("two bounded claims");
    assert_eq!(
        validate(&rows, &record_index(&rows), &index, claim()),
        Outcome::Intact
    );
    let wrong_declaration_digest = Claim {
        declaration_digest: [0; 32],
        ..claim()
    };
    assert_eq!(
        validate(
            &rows,
            &record_index(&rows),
            &index,
            wrong_declaration_digest
        ),
        damage(Cause::Pointer)
    );
    let wrong_chunk_digest = Claim {
        last_chunk_digest: [0; 32],
        ..claim()
    };
    assert_eq!(
        validate(&rows, &record_index(&rows), &index, wrong_chunk_digest),
        damage(Cause::Pointer)
    );
    assert!(
        claims(&rows, 1).is_none(),
        "graph entry bound is not waived"
    );

    let mut gap = selected();
    if let Some(BlobFact::Chunk { ordinal, .. }) = gap[1].fact.as_mut() {
        *ordinal = 3;
    }
    let index = claims(&gap, 2).expect("bounded gap");
    assert_eq!(
        validate(&gap, &record_index(&gap), &index, claim()),
        damage(Cause::Pointer)
    );

    // A walk cut short may not have visited the chunk that fills the gap.
    let bound = Outcome::Indeterminate(Indeterminate::EntryBoundExceeded);
    let cut_short = RowIndex::new(&gap, &Coverage::CutShort(bound.clone()));
    let index = ClaimIndex::new(&gap, &cut_short, 2).expect("bounded gap");
    assert_eq!(validate(&gap, &record_index(&gap), &index, claim()), bound);
}

/// A frame that could not be read fills a gap only if it may be a chunk claim.
#[test]
fn an_unread_frame_leaves_a_gap_undecided_only_if_it_may_be_a_chunk_claim() {
    let aliased = Outcome::Unknown(Unknown::PhysicalAliasNotReinspected);
    let beside = |kind: Option<u8>| {
        let mut gap = selected();
        gap.remove(1);
        gap.push(unread([16; 24], kind, aliased.clone()));
        let index = claims(&gap, 2).expect("bounded gap");
        validate(&gap, &record_index(&gap), &index, claim())
    };
    // A chunk frame, a reuse claim, or a frame of no kind the walk could tell.
    for may_be in [Some(2), Some(11), Some(15), None] {
        assert_eq!(beside(may_be), aliased, "{may_be:?}");
    }
    // A declaration, a frontier and an abandonment share a reuse claim's
    // family and are no chunk claim.
    for other in [1, 5, 6, 3, 4, 13, 12] {
        assert_eq!(beside(Some(other)), damage(Cause::Pointer), "{other}");
    }
    // A frame that was read and is damaged fills no gap.
    let mut gap = selected();
    gap.remove(1);
    gap.push(unread([16; 24], None, damage(Cause::ChecksumMismatch)));
    let index = claims(&gap, 2).expect("bounded gap");
    assert_eq!(
        validate(&gap, &record_index(&gap), &index, claim()),
        damage(Cause::Pointer)
    );
}

const BORROWED_CHUNK: [u8; 24] = [12; 24];

/// The session's claim on ordinal one as a reuse of another session's chunk.
fn reuse_claim(record: [u8; 24]) -> Selected {
    row(
        record,
        BlobFact::ReuseClaim {
            store: [1; 16],
            session: [2; 16],
            ordinal: 1,
            scope: [10; 32],
            chunk_size: 64 << 10,
            length: 64 << 10,
            digest: [8; 32],
            chunk_record: BORROWED_CHUNK,
            source_publication: [13; 24],
            source_ordinal: 0,
            source_witness: None,
        },
    )
}

fn reused_last_chunk() -> Vec<Selected> {
    let mut rows = selected();
    rows[2] = reuse_claim([6; 24]);
    rows
}

#[test]
fn frontier_over_a_reused_last_chunk_names_the_claim_row_not_the_borrowed_chunk() {
    let rows = reused_last_chunk();
    let index = claims(&rows, 2).expect("two bounded claims");
    assert_eq!(
        validate(&rows, &record_index(&rows), &index, claim()),
        Outcome::Intact
    );
    for wrong in [BORROWED_CHUNK, [5; 24], [14; 24]] {
        let wrong_record = Claim {
            last_chunk_record: wrong,
            ..claim()
        };
        assert_eq!(
            validate(&rows, &record_index(&rows), &index, wrong_record),
            damage(Cause::Pointer),
            "last chunk record {wrong:?}"
        );
    }
    let wrong_digest = Claim {
        last_chunk_digest: [7; 32],
        ..claim()
    };
    assert_eq!(
        validate(&rows, &record_index(&rows), &index, wrong_digest),
        damage(Cause::Pointer)
    );
}

#[test]
fn a_second_selected_row_claiming_a_reused_ordinal_damages_the_prefix() {
    for second in [
        reuse_claim([15; 24]),
        row(
            [15; 24],
            BlobFact::Chunk {
                store: [1; 16],
                session: [2; 16],
                ordinal: 1,
                chunk_size: 64 << 10,
                length: 64 << 10,
                digest: [8; 32],
            },
        ),
    ] {
        let mut rows = reused_last_chunk();
        rows.push(second);
        let index = claims(&rows, 2).expect("one ordinal is one bounded claim");
        assert_eq!(
            validate(&rows, &record_index(&rows), &index, claim()),
            damage(Cause::Pointer)
        );
    }

    // Re-observation of the identical selected claim row is not a second claim.
    let mut rows = reused_last_chunk();
    rows.push(reuse_claim([6; 24]));
    let index = claims(&rows, 2).expect("repeated same record costs one ordinal");
    assert_eq!(
        validate(&rows, &record_index(&rows), &index, claim()),
        Outcome::Intact
    );
}

#[test]
fn index_admits_one_ordinal_before_allocation_and_reuses_identical_evidence() {
    let mut rows = selected();
    rows.remove(2); // Keep declaration, ordinal zero, and the frontier marker.
    rows.push(row(
        [5; 24],
        BlobFact::Chunk {
            store: [1; 16],
            session: [2; 16],
            ordinal: 0,
            chunk_size: 64 << 10,
            length: 64 << 10,
            digest: [7; 32],
        },
    ));
    let index = claims(&rows, 1).expect("repeated same record costs one ordinal");
    let first = Claim {
        next_chunk_ordinal: 1,
        durable_bytes: 64 << 10,
        last_chunk_record: [5; 24],
        last_chunk_digest: [7; 32],
        ..claim()
    };
    assert_eq!(
        validate(&rows, &record_index(&rows), &index, first),
        Outcome::Intact
    );
    assert!(
        claims(&selected(), 1).is_none(),
        "second unique ordinal denies"
    );
}
