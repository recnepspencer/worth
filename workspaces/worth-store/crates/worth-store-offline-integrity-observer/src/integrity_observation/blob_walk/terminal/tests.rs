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

fn rows() -> Vec<Selected> {
    vec![
        row(
            [3; 24],
            BlobFact::Declaration {
                store: [1; 16],
                session: [2; 16],
                frame_digest: [4; 32],
                object: [5; 16],
                scope: [6; 32],
                chunk_size: 65536,
                total: 131072,
                max_checkpoint_sequence: 5,
            },
        ),
        row(
            [7; 24],
            BlobFact::Abandoned {
                store: [1; 16],
                frame_digest: [8; 32],
                session: [2; 16],
                declaration_record: [3; 24],
                declaration_digest: [4; 32],
                expiry_checkpoint: None,
            },
        ),
    ]
}

fn validate_rows(rows: &mut [Selected]) {
    let sessions = BTreeMap::from([([2; 16], 0)]);
    let records = rows
        .iter()
        .enumerate()
        .map(|(index, row)| (row.record, index))
        .collect();
    validate(
        rows,
        &sessions,
        &records,
        &SelectedCheckpointEvidence::Absent,
    );
}

#[test]
fn expiry_requires_completed_current_checkpoint_crossing_original_maximum() {
    for (witness, current, valid) in [(5, 8, false), (6, 6, true), (6, 20, true), (9, 8, false)] {
        assert_eq!(
            expiry_eligibility(
                witness,
                5,
                &SelectedCheckpointEvidence::Validated {
                    sequence: current,
                    release_claim: None,
                }
            ) == Outcome::Intact,
            valid,
        );
    }
    assert!(matches!(
        expiry_eligibility(6, 5, &SelectedCheckpointEvidence::Absent),
        Outcome::Damaged(_)
    ));
    let interrupted =
        Outcome::Indeterminate(super::super::OfflineIndeterminatePhysicalReason::ByteBoundExceeded);
    assert_eq!(
        expiry_eligibility(
            6,
            5,
            &SelectedCheckpointEvidence::Unavailable(interrupted.clone())
        ),
        interrupted
    );
}

#[test]
fn expiry_graph_uses_authenticated_declaration_maximum() {
    let mut rows = rows();
    let Some(BlobFact::Abandoned {
        expiry_checkpoint, ..
    }) = rows[1].fact.as_mut()
    else {
        unreachable!()
    };
    *expiry_checkpoint = Some(5);
    let sessions = BTreeMap::from([([2; 16], 0)]);
    let records = BTreeMap::from([([3; 24], 0), ([7; 24], 1)]);
    validate(
        &mut rows,
        &sessions,
        &records,
        &SelectedCheckpointEvidence::Validated {
            sequence: 20,
            release_claim: None,
        },
    );
    assert!(matches!(rows[1].outcome, Outcome::Damaged(_)));
}

#[test]
fn checkpoint_damage_is_relocalized_to_terminal_dependency() {
    let damaged_checkpoint = crate::integrity_observation::record_walk::damage(
        Cause::ChecksumMismatch,
        Some((160, 4)),
        super::super::Blast::Field,
    );
    let outcome = expiry_eligibility(
        6,
        5,
        &SelectedCheckpointEvidence::Unavailable(damaged_checkpoint),
    );
    let Outcome::Damaged(localization) = outcome else {
        panic!("dependency must be damaged")
    };
    assert_eq!(localization.cause(), Cause::Pointer);
    assert_eq!(localization.damaged_range(), None);
}

#[test]
fn abort_graph_binds_selected_declaration_not_only_session_name() {
    let mut valid = rows();
    validate_rows(&mut valid);
    assert_eq!(valid[1].outcome, Outcome::Intact);
    for mismatch in 0..3 {
        let mut invalid = rows();
        let Some(BlobFact::Abandoned {
            store,
            declaration_record,
            declaration_digest,
            ..
        }) = invalid[1].fact.as_mut()
        else {
            unreachable!()
        };
        match mismatch {
            0 => *store = [9; 16],
            1 => *declaration_record = [9; 24],
            _ => *declaration_digest = [9; 32],
        }
        validate_rows(&mut invalid);
        assert!(matches!(invalid[1].outcome, Outcome::Damaged(_)));
    }
}

#[test]
fn abort_graph_rejects_duplicate_terminal_and_any_publication_for_object() {
    let mut duplicates = rows();
    duplicates.push(row([8; 24], duplicates[1].fact.clone().unwrap()));
    validate_rows(&mut duplicates);
    assert!(duplicates[1..]
        .iter()
        .all(|row| matches!(row.outcome, Outcome::Damaged(_))));
    for session in [[2; 16], [9; 16]] {
        let mut conflict = rows();
        conflict.push(row(
            [8; 24],
            BlobFact::Publication {
                store: [1; 16],
                frame_digest: [13; 32],
                session,
                object: [5; 16],
                generation: 1,
                root: [10; 24],
                root_digest: [11; 32],
                total: 131072,
                logical_digest: [12; 32],
                chunk_size: 65536,
                scope: [6; 32],
            },
        ));
        validate_rows(&mut conflict);
        assert!(conflict[1..]
            .iter()
            .all(|row| matches!(row.outcome, Outcome::Damaged(_))));
    }
}
