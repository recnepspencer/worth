use super::super::super::blob_record::BlobEdge;
use super::super::OfflineArtifactFamily;
use super::*;
use worth_foundational::PhysicalArtifactFamily as Family;

fn record(ordinal: u64) -> [u8; 24] {
    let mut bytes = [1; 24];
    bytes[16..].copy_from_slice(&ordinal.to_le_bytes());
    bytes
}

fn row(record: [u8; 24], generation: u64, fact: BlobFact) -> Selected {
    Selected {
        record,
        path: String::new(),
        generation,
        family: OfflineArtifactFamily::Declared(Family::BlobResumeSession),
        fact: Some(fact),
        outcome: Outcome::Intact,
        route: None,
    }
}

#[test]
fn selected_source_edge_and_destination_scope_authorize_one_reuse_claim() {
    let scope = [3; 32];
    let digest = [4; 32];
    let source = [5; 16];
    let destination = [6; 16];
    let rows = vec![
        row(
            record(1),
            1,
            BlobFact::Declaration {
                store: [1; 16],
                session: source,
                frame_digest: [7; 32],
                object: [8; 16],
                scope,
                chunk_size: 64 << 10,
                total: 64 << 10,
                max_checkpoint_sequence: 9,
            },
        ),
        row(
            record(2),
            2,
            BlobFact::Chunk {
                store: [1; 16],
                session: source,
                ordinal: 0,
                chunk_size: 64 << 10,
                length: 64 << 10,
                digest,
            },
        ),
        row(
            record(3),
            3,
            BlobFact::Node {
                store: [1; 16],
                session: source,
                kind: 1,
                level: 0,
                index: 0,
                covered: 64 << 10,
                digest: [9; 32],
                frame_digest: [10; 32],
                entries: vec![BlobEdge {
                    digest,
                    record: record(2),
                    covered: 64 << 10,
                }],
            },
        ),
        row(
            record(4),
            4,
            BlobFact::Publication {
                store: [1; 16],
                frame_digest: [14; 32],
                session: source,
                object: [8; 16],
                generation: 1,
                root: record(3),
                root_digest: [10; 32],
                total: 64 << 10,
                logical_digest: [11; 32],
                chunk_size: 64 << 10,
                scope,
            },
        ),
        row(
            record(5),
            5,
            BlobFact::Declaration {
                store: [1; 16],
                session: destination,
                frame_digest: [12; 32],
                object: [13; 16],
                scope,
                chunk_size: 64 << 10,
                total: 64 << 10,
                max_checkpoint_sequence: 14,
            },
        ),
        row(
            record(6),
            6,
            BlobFact::ReuseClaim {
                store: [1; 16],
                session: destination,
                ordinal: 0,
                scope,
                chunk_size: 64 << 10,
                length: 64 << 10,
                digest,
                chunk_record: record(2),
                source_publication: record(4),
                source_ordinal: 0,
                source_witness: None,
            },
        ),
    ];
    let sessions = BTreeMap::from([(source, 0), (destination, 4)]);
    let records = rows
        .iter()
        .enumerate()
        .map(|(index, row)| (row.record, index))
        .collect();
    let mut accepted = rows;
    let claims = validate(&mut accepted, &sessions, &records);
    assert_eq!(claims.get(&(destination, 0)), Some(&5));
    assert_eq!(accepted[5].outcome, Outcome::Intact);
    accepted[3].generation = 100;
    validate(&mut accepted, &sessions, &records);
    assert_eq!(
        accepted[5].outcome,
        Outcome::Intact,
        "rewritten source placement is not a later source publication"
    );
    if let Some(BlobFact::Chunk { ordinal, .. }) = accepted[1].fact.as_mut() {
        *ordinal = 1;
    }
    validate(&mut accepted, &sessions, &records);
    assert!(matches!(accepted[5].outcome, Outcome::Damaged(_)));
    if let Some(BlobFact::Chunk { ordinal, .. }) = accepted[1].fact.as_mut() {
        *ordinal = 0;
    }
    accepted[5].outcome = Outcome::Intact;
    if let Some(BlobFact::ReuseClaim { scope, .. }) = accepted[5].fact.as_mut() {
        *scope = [99; 32];
    }
    validate(&mut accepted, &sessions, &records);
    assert!(matches!(accepted[5].outcome, Outcome::Damaged(_)));

    accepted.remove(3);
    accepted[4].outcome = Outcome::Intact;
    if let Some(BlobFact::ReuseClaim {
        scope,
        source_witness,
        ..
    }) = accepted[4].fact.as_mut()
    {
        *scope = [3; 32];
        *source_witness = Some(ReuseSourceWitness {
            frame_digest: [14; 32],
            store: [1; 16],
            session: source,
            object: [8; 16],
            generation: 1,
            root: record(3),
            root_digest: [10; 32],
            total: 64 << 10,
            chunk_size: 64 << 10,
            scope: [3; 32],
        });
    }
    accepted.push(row(
        record(7),
        7,
        BlobFact::ReleasedDropSetManifest {
            store: [1; 16],
            frame_digest: [15; 32],
            attempt: [16; 16],
            object: [8; 16],
            session: source,
            generation: 1,
            root: record(3),
            root_digest: [10; 32],
            publication_record: record(4),
            publication_digest: [14; 32],
            issuer_evidence_digest: [18; 32],
            basis_digest: [17; 32],
            dropped: vec![record(4)],
            never_reserved_slot_generation: 5,
        },
    ));
    accepted.push(row(
        record(8),
        8,
        BlobFact::ReleasedReclaimDescriptor {
            store: [1; 16],
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
    let post_release_records = accepted
        .iter()
        .enumerate()
        .map(|(index, row)| (row.record, index))
        .collect();
    let post_release_sessions = BTreeMap::from([(source, 0), (destination, 3)]);
    validate(&mut accepted, &post_release_sessions, &post_release_records);
    assert_eq!(
        accepted[4].outcome,
        Outcome::Unknown(Unknown::WalCoverageUnavailable),
        "selected V3/V2 provenance cannot invent selected WAL fate"
    );
}
