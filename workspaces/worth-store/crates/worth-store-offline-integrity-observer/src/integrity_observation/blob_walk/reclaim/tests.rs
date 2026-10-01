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

#[test]
fn manifest_alone_is_custody_and_descriptor_requires_exact_absence() {
    let declaration = [1; 24];
    let abandoned = [2; 24];
    let manifest = [3; 24];
    let descriptor = [4; 24];
    let dropped = [5; 24];
    let mut rows = vec![
        row(
            declaration,
            BlobFact::Declaration {
                store: [1; 16],
                session: [2; 16],
                frame_digest: [8; 32],
                object: [3; 16],
                scope: [4; 32],
                chunk_size: 64 << 10,
                total: 64 << 10,
                max_checkpoint_sequence: 5,
            },
        ),
        row(
            abandoned,
            BlobFact::Abandoned {
                store: [1; 16],
                frame_digest: [9; 32],
                session: [2; 16],
                declaration_record: declaration,
                declaration_digest: [8; 32],
                expiry_checkpoint: None,
            },
        ),
        row(
            manifest,
            BlobFact::DropSetManifest {
                store: [1; 16],
                frame_digest: [10; 32],
                attempt: [5; 16],
                session: [2; 16],
                declaration_record: declaration,
                declaration_digest: [8; 32],
                abandoned_record: abandoned,
                abandoned_digest: [9; 32],
                basis_digest: [11; 32],
                dropped: vec![dropped],
                never_reserved_slot_generation: None,
            },
        ),
    ];
    let mut records = BTreeMap::from([(declaration, 0), (abandoned, 1), (manifest, 2)]);
    validate(&mut rows, &records, None);
    assert_eq!(rows[2].outcome, Outcome::Intact);

    rows.push(row(
        descriptor,
        BlobFact::ReclaimDescriptor {
            store: [1; 16],
            attempt: [5; 16],
            basis_digest: [11; 32],
            manifest_record: manifest,
            manifest_digest: [10; 32],
            manifest_count: 1,
            source_root: 3,
            candidate_root: 4,
        },
    ));
    records.insert(descriptor, 3);
    validate(&mut rows, &records, None);
    assert_eq!(rows[3].outcome, Outcome::Intact);

    rows.push(row(
        dropped,
        BlobFact::Chunk {
            store: [1; 16],
            session: [2; 16],
            ordinal: 0,
            chunk_size: 64 << 10,
            length: 64 << 10,
            digest: [12; 32],
        },
    ));
    records.insert(dropped, 4);
    validate(&mut rows, &records, None);
    assert!(matches!(rows[3].outcome, Outcome::Damaged(_)));
}

fn v2_graph() -> (Vec<Selected>, BTreeMap<[u8; 24], usize>) {
    let declaration = [1; 24];
    let abandoned = [2; 24];
    let manifest = [3; 24];
    let reserved = [4; 24];
    let descriptor = [5; 24];
    let rows = vec![
        row(
            declaration,
            BlobFact::Declaration {
                store: [1; 16],
                session: [2; 16],
                frame_digest: [8; 32],
                object: [3; 16],
                scope: [4; 32],
                chunk_size: 64 << 10,
                total: 64 << 10,
                max_checkpoint_sequence: 5,
            },
        ),
        row(
            abandoned,
            BlobFact::Abandoned {
                store: [1; 16],
                frame_digest: [9; 32],
                session: [2; 16],
                declaration_record: declaration,
                declaration_digest: [8; 32],
                expiry_checkpoint: None,
            },
        ),
        row(
            manifest,
            BlobFact::DropSetManifest {
                store: [1; 16],
                frame_digest: [10; 32],
                attempt: [5; 16],
                session: [2; 16],
                declaration_record: declaration,
                declaration_digest: [8; 32],
                abandoned_record: abandoned,
                abandoned_digest: [9; 32],
                basis_digest: [11; 32],
                dropped: vec![[6; 24]],
                never_reserved_slot_generation: Some(3),
            },
        ),
        row(
            reserved,
            BlobFact::OriginalDropReserved {
                store: [1; 16],
                frame_digest: [12; 32],
                attempt: [5; 16],
                manifest_record: manifest,
                manifest_digest: [10; 32],
                basis_digest: [11; 32],
                manifest_selected_generation: 3,
                reserved_selected_generation: 4,
                idempotency: [13; 32],
                fingerprint: [14; 32],
                lease_issuance_generation: 3,
                lease_expiry_generation: 7,
            },
        ),
        row(
            descriptor,
            BlobFact::ReclaimDescriptor {
                store: [1; 16],
                attempt: [5; 16],
                basis_digest: [11; 32],
                manifest_record: manifest,
                manifest_digest: [10; 32],
                manifest_count: 1,
                source_root: 4,
                candidate_root: 5,
            },
        ),
    ];
    let records = BTreeMap::from([
        (declaration, 0),
        (abandoned, 1),
        (manifest, 2),
        (reserved, 3),
        (descriptor, 4),
    ]);
    (rows, records)
}

#[test]
fn v2_reservation_and_descriptor_join_exact_selected_manifest() {
    let (mut rows, records) = v2_graph();
    validate(&mut rows, &records, Some(5));
    assert!(rows.iter().all(|row| row.outcome == Outcome::Intact));

    let (mut rows, records) = v2_graph();
    rows.pop();
    validate(&mut rows, &records, Some(4));
    assert_eq!(rows[2].outcome, Outcome::Intact);
    assert_eq!(rows[3].outcome, Outcome::Intact);
}

#[test]
fn v2_mismatched_or_duplicate_reservation_cannot_validate_descriptor() {
    let (mut rows, records) = v2_graph();
    let Some(BlobFact::OriginalDropReserved {
        manifest_digest, ..
    }) = rows[3].fact.as_mut()
    else {
        panic!("reservation fixture");
    };
    *manifest_digest = [99; 32];
    validate(&mut rows, &records, Some(5));
    assert_eq!(rows[2].outcome, Outcome::Intact);
    assert!(matches!(rows[3].outcome, Outcome::Damaged(_)));
    assert!(matches!(rows[4].outcome, Outcome::Damaged(_)));

    let (mut rows, mut records) = v2_graph();
    let duplicate = row([7; 24], rows[3].fact.clone().unwrap());
    records.insert(duplicate.record, rows.len());
    rows.push(duplicate);
    validate(&mut rows, &records, Some(5));
    assert!(matches!(rows[3].outcome, Outcome::Damaged(_)));
    assert!(matches!(rows[5].outcome, Outcome::Damaged(_)));
    assert!(matches!(rows[4].outcome, Outcome::Damaged(_)));
}

#[test]
fn future_reservation_generation_is_not_selected_custody() {
    let (mut rows, records) = v2_graph();
    validate(&mut rows, &records, Some(3));
    assert!(matches!(rows[3].outcome, Outcome::Damaged(_)));
    assert!(matches!(rows[4].outcome, Outcome::Damaged(_)));
}
