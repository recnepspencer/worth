use std::collections::BTreeMap;

use super::{damage, BlobFact, Cause, Outcome, Selected, SelectedCheckpointEvidence};

/// Terminal fate does not remove custody edges or authorize physical deletion.
pub(super) fn validate(
    rows: &mut [Selected],
    sessions: &BTreeMap<[u8; 16], usize>,
    records: &BTreeMap<[u8; 24], usize>,
    checkpoint: &SelectedCheckpointEvidence,
) {
    let mut terminals = BTreeMap::new();
    let mut terminal_objects = BTreeMap::new();
    let mut conflicts = Vec::new();
    for index in 0..rows.len() {
        if rows[index].outcome != Outcome::Intact {
            continue;
        }
        let Some(BlobFact::Abandoned {
            store,
            session,
            declaration_record,
            declaration_digest,
            expiry_checkpoint,
            ..
        }) = rows[index].fact.as_ref()
        else {
            continue;
        };
        let declared = records.get(declaration_record).copied();
        let valid = declared.is_some_and(|declared| {
            sessions.get(session) == Some(&declared)
                && rows[declared].outcome == Outcome::Intact
                && matches!(rows[declared].fact.as_ref(),
                    Some(BlobFact::Declaration {
                        store: declared_store, session: declared_session, frame_digest, ..
                    }) if declared_store == store && declared_session == session
                        && frame_digest == declaration_digest)
        });
        if !valid {
            rows[index].outcome = damage(Cause::Pointer);
            continue;
        }
        if let Some(witness) = expiry_checkpoint {
            let Some(BlobFact::Declaration {
                max_checkpoint_sequence,
                ..
            }) = rows[declared.expect("validated declaration")].fact.as_ref()
            else {
                unreachable!("validated declaration")
            };
            let outcome = expiry_eligibility(*witness, *max_checkpoint_sequence, checkpoint);
            if outcome != Outcome::Intact {
                rows[index].outcome = outcome;
                continue;
            }
        }
        if let Some(prior) = terminals.insert(*session, index) {
            if rows[prior].record != rows[index].record {
                conflicts.extend([prior, index]);
            }
        }
        let Some(BlobFact::Declaration { object, .. }) =
            rows[declared.expect("validated declaration")].fact.as_ref()
        else {
            unreachable!("validated declaration");
        };
        if let Some(prior) = terminal_objects.insert(*object, index) {
            if rows[prior].record != rows[index].record {
                conflicts.extend([prior, index]);
            }
        }
    }
    for (index, row) in rows.iter().enumerate() {
        let Some(BlobFact::Publication {
            session, object, ..
        }) = row.fact.as_ref()
        else {
            continue;
        };
        if let Some(terminal) = terminals.get(session) {
            conflicts.extend([*terminal, index]);
        }
        // A publication cannot evade terminal custody by renaming its session.
        if let Some(terminal) = terminal_objects.get(object) {
            conflicts.extend([*terminal, index]);
        }
    }
    for index in conflicts {
        if rows[index].outcome == Outcome::Intact {
            rows[index].outcome = damage(Cause::DuplicateIdentity);
        }
    }
}

fn expiry_eligibility(
    witness: u64,
    maximum: u64,
    checkpoint: &SelectedCheckpointEvidence,
) -> Outcome {
    if witness <= maximum {
        return damage(Cause::Pointer);
    }
    match checkpoint {
        SelectedCheckpointEvidence::Validated { sequence, .. } if *sequence >= witness => {
            Outcome::Intact
        }
        SelectedCheckpointEvidence::Unavailable(Outcome::Damaged(_)) => damage(Cause::Pointer),
        SelectedCheckpointEvidence::Unavailable(Outcome::Unsupported(_)) => Outcome::Unknown(
            crate::integrity_observation::OfflineUnknownPhysicalReason::ParentScopeUnavailable,
        ),
        SelectedCheckpointEvidence::Unavailable(outcome) => outcome.clone(),
        SelectedCheckpointEvidence::Absent | SelectedCheckpointEvidence::Validated { .. } => {
            damage(Cause::Pointer)
        }
    }
}

#[cfg(test)]
mod tests;
