use super::*;

fn row(record: [u8; 24], fact: BlobFact) -> Selected {
    Selected {
        record,
        path: "arena".into(),
        generation: 1,
        family: fact.family().into(),
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

fn record_index(rows: &[Selected]) -> BTreeMap<[u8; 24], usize> {
    rows.iter()
        .enumerate()
        .map(|(index, row)| (row.record, index))
        .collect()
}

#[test]
fn selected_frontier_requires_complete_ordered_prefix_and_exact_dependencies() {
    let rows = selected();
    let index = ClaimIndex::new(&rows, 2).expect("two bounded claims");
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
        ClaimIndex::new(&rows, 1).is_none(),
        "graph entry bound is not waived"
    );

    let mut gap = selected();
    if let Some(BlobFact::Chunk { ordinal, .. }) = gap[1].fact.as_mut() {
        *ordinal = 3;
    }
    let index = ClaimIndex::new(&gap, 2).expect("bounded gap");
    assert_eq!(
        validate(&gap, &record_index(&gap), &index, claim()),
        damage(Cause::Pointer)
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
    let index = ClaimIndex::new(&rows, 1).expect("repeated same record costs one ordinal");
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
        ClaimIndex::new(&selected(), 1).is_none(),
        "second unique ordinal denies"
    );
}
