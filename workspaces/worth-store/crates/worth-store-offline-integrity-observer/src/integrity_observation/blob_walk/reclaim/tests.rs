use super::super::coverage::Coverage;
use super::super::reuse_source_fixture::unread as unread_frame_of;
use super::*;
use crate::integrity_observation::OfflineIndeterminatePhysicalReason as Indeterminate;

/// Check the rows of a walk that visited every routed record.
fn check(rows: &mut [Selected], selected_root_generation: Option<u64>) {
    let found = RowIndex::new(rows, &Coverage::Complete);
    validate(rows, &found, selected_root_generation);
}

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
    check(&mut rows, None);
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
    check(&mut rows, None);
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
    check(&mut rows, None);
    assert!(matches!(rows[3].outcome, Outcome::Damaged(_)));
}

fn v2_graph() -> Vec<Selected> {
    let declaration = [1; 24];
    let abandoned = [2; 24];
    let manifest = [3; 24];
    let reserved = [4; 24];
    let descriptor = [5; 24];
    vec![
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
    ]
}

#[test]
fn v2_reservation_and_descriptor_join_exact_selected_manifest() {
    let mut rows = v2_graph();
    check(&mut rows, Some(5));
    assert!(rows.iter().all(|row| row.outcome == Outcome::Intact));

    let mut rows = v2_graph();
    rows.pop();
    check(&mut rows, Some(4));
    assert_eq!(rows[2].outcome, Outcome::Intact);
    assert_eq!(rows[3].outcome, Outcome::Intact);
}

#[test]
fn v2_mismatched_or_duplicate_reservation_cannot_validate_descriptor() {
    let mut rows = v2_graph();
    let Some(BlobFact::OriginalDropReserved {
        manifest_digest, ..
    }) = rows[3].fact.as_mut()
    else {
        panic!("reservation fixture");
    };
    *manifest_digest = [99; 32];
    check(&mut rows, Some(5));
    assert_eq!(rows[2].outcome, Outcome::Intact);
    assert!(matches!(rows[3].outcome, Outcome::Damaged(_)));
    assert!(matches!(rows[4].outcome, Outcome::Damaged(_)));

    let mut rows = v2_graph();
    let duplicate = row([7; 24], rows[3].fact.clone().unwrap());
    rows.push(duplicate);
    check(&mut rows, Some(5));
    assert!(matches!(rows[3].outcome, Outcome::Damaged(_)));
    assert!(matches!(rows[5].outcome, Outcome::Damaged(_)));
    assert!(matches!(rows[4].outcome, Outcome::Damaged(_)));
}

#[test]
fn future_reservation_generation_is_not_selected_custody() {
    let mut rows = v2_graph();
    check(&mut rows, Some(3));
    assert!(matches!(rows[3].outcome, Outcome::Damaged(_)));
    assert!(matches!(rows[4].outcome, Outcome::Damaged(_)));
}

/// A descriptor's reservation is found by what its frame says, so a row whose
/// frame could not be read may be it, and so may a row the walk did not visit.
#[test]
fn a_reservation_the_walk_could_not_see_leaves_its_descriptor_undecided() {
    const RESERVED: usize = 3;
    let descriptor = |rows: &[Selected]| rows.last().expect("the descriptor").outcome.clone();
    let unread = Outcome::Unknown(Unknown::PhysicalAliasNotReinspected);
    let bound = Outcome::Indeterminate(Indeterminate::EntryBoundExceeded);

    let mut absent = v2_graph();
    absent.remove(RESERVED);
    check(&mut absent, Some(5));
    assert_eq!(descriptor(&absent), damage(Cause::Pointer));

    let mut unread_frame = v2_graph();
    unread_frame[RESERVED].fact = None;
    unread_frame[RESERVED].outcome = unread.clone();
    check(&mut unread_frame, Some(5));
    assert_eq!(descriptor(&unread_frame), unread);

    let mut unvisited = v2_graph();
    unvisited.remove(RESERVED);
    let cut_short = RowIndex::new(&unvisited, &Coverage::CutShort(bound.clone()));
    validate(&mut unvisited, &cut_short, Some(5));
    assert_eq!(descriptor(&unvisited), bound);

    // A reservation that was read and agrees, but is itself undecided.
    let mut unselected_root = v2_graph();
    check(&mut unselected_root, None);
    let undecided = Outcome::Unknown(Unknown::SelectorUnavailable);
    assert_eq!(unselected_root[RESERVED].outcome, undecided);
    assert_eq!(descriptor(&unselected_root), undecided);

    // An unread frame of another kind is not the reservation; one whose kind
    // the walk could not tell may be.
    let beside = |kind: Option<u8>, outcome: &Outcome| {
        let mut rows = v2_graph();
        rows[RESERVED] = unread_frame_of([9; 24], kind, outcome.clone());
        check(&mut rows, Some(5));
        descriptor(&rows)
    };
    assert_eq!(beside(Some(2), &unread), damage(Cause::Pointer));
    assert_eq!(beside(Some(10), &unread), unread);
    assert_eq!(beside(None, &unread), unread);
    // A frame that was read and is damaged is not a reservation unseen.
    let damaged = damage(Cause::ChecksumMismatch);
    assert_eq!(beside(None, &damaged), damage(Cause::Pointer));
}

/// A descriptor says that the records its manifest dropped are gone. A record
/// that still has a row contradicts it, read or not; a walk that was cut short
/// cannot say that a record has none.
#[test]
fn a_dropped_record_the_walk_did_not_visit_leaves_its_descriptor_undecided() {
    const DROPPED: [u8; 24] = [6; 24];
    const DESCRIPTOR: usize = 4;
    let unread = Outcome::Unknown(Unknown::PhysicalAliasNotReinspected);
    let bound = Outcome::Indeterminate(Indeterminate::EntryBoundExceeded);
    let cut_short = Coverage::CutShort(bound.clone());
    let judged = |mut rows: Vec<Selected>, coverage: &Coverage| {
        let found = RowIndex::new(&rows, coverage);
        validate(&mut rows, &found, Some(5));
        rows
    };

    let unvisited = judged(v2_graph(), &cut_short);
    assert_eq!(unvisited[DESCRIPTOR].outcome, bound);
    let read = &unvisited[..DESCRIPTOR];
    assert!(read.iter().all(|row| row.outcome == Outcome::Intact));

    for coverage in [Coverage::Complete, cut_short] {
        let mut still_routed = v2_graph();
        still_routed.push(unread_frame_of(DROPPED, None, unread.clone()));
        let still_routed = judged(still_routed, &coverage);
        assert_eq!(still_routed[DESCRIPTOR].outcome, damage(Cause::Pointer));
    }
}
