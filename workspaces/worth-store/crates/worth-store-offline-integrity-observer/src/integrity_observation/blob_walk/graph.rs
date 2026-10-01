use std::collections::BTreeMap;

use super::super::blob_record::BlobFact;
use super::super::OfflineIntegrityOutcome as Outcome;
use super::super::OfflinePhysicalDamageCause as Cause;
use super::{damage, BlobRecordWalk, ChildExpectation, ChildScope, Selected};

/// Classify every selected chunk before a frontier summarizes any prefix.
/// Selected rows have no ordinal order, so this must precede frontier indexing.
pub(super) fn validate_chunks(
    selected: &mut [Selected],
    sessions: &BTreeMap<[u8; 16], usize>,
    records: &BTreeMap<[u8; 24], usize>,
) {
    for index in 0..selected.len() {
        if selected[index].outcome != Outcome::Intact {
            continue;
        }
        let Some(BlobFact::Chunk {
            store,
            session,
            ordinal,
            chunk_size,
            length,
            ..
        }) = selected[index].fact.as_ref()
        else {
            continue;
        };
        if let Some(outcome) = uncertain_dependency(
            selected[index].fact.as_ref().expect("selected chunk"),
            selected,
            sessions,
            records,
        ) {
            selected[index].outcome = outcome;
            continue;
        }
        let valid = declaration(selected, sessions, *session).is_some_and(
            |(declared_store, _, _, declared_chunk_size, total)| {
                let start = ordinal.checked_mul(u64::from(*chunk_size));
                let expected_length = start.and_then(|start| {
                    total
                        .checked_sub(start)
                        .map(|remaining| remaining.min(u64::from(*chunk_size)))
                });
                *declared_store == *store
                    && *declared_chunk_size == *chunk_size
                    && expected_length == Some(*length)
            },
        );
        if !valid {
            selected[index].outcome = damage(Cause::ScopeMismatch);
        }
    }
}

impl BlobRecordWalk {
    pub(crate) fn note_extent_chunk_outcome(
        &mut self,
        expected: &ChildExpectation,
        outcome: &Outcome,
    ) {
        let ChildScope::ExtentChunk { record, .. } = expected.scope else {
            return;
        };
        if *outcome == Outcome::Intact {
            return;
        }
        if let Some(pending) = self
            .pending
            .as_mut()
            .filter(|pending| pending.record == record)
        {
            if pending.interruption.is_none() {
                pending.interruption = Some(outcome.clone());
            }
        }
    }
}

/// A bound or unavailable selected dependency is not proof of a bad pointer.
pub(super) fn dependency_uncertainty(row: &Selected) -> Option<Outcome> {
    match &row.outcome {
        Outcome::Unknown(_) | Outcome::Indeterminate(_) | Outcome::Unsupported(_) => {
            Some(row.outcome.clone())
        }
        Outcome::Intact | Outcome::Damaged(_) => None,
    }
}

pub(super) fn uncertain_dependency(
    fact: &BlobFact,
    selected: &[Selected],
    sessions: &BTreeMap<[u8; 16], usize>,
    records: &BTreeMap<[u8; 24], usize>,
) -> Option<Outcome> {
    let session_outcome = |session: &[u8; 16]| {
        sessions
            .get(session)
            .and_then(|prior| dependency_uncertainty(&selected[*prior]))
    };
    let record_outcome = |record: &[u8; 24]| {
        records
            .get(record)
            .and_then(|prior| dependency_uncertainty(&selected[*prior]))
    };
    match fact {
        BlobFact::Declaration { .. } => None,
        BlobFact::Abandoned {
            session,
            declaration_record,
            ..
        } => session_outcome(session).or_else(|| record_outcome(declaration_record)),
        BlobFact::Chunk { session, .. } => session_outcome(session),
        BlobFact::ReuseClaim {
            session,
            chunk_record,
            source_publication,
            ..
        } => session_outcome(session)
            .or_else(|| record_outcome(chunk_record))
            .or_else(|| record_outcome(source_publication)),
        BlobFact::DedupeQuarantine {
            source_publication,
            source_chunk,
            destination_session,
            conflicting_chunk,
            ..
        } => session_outcome(destination_session)
            .or_else(|| record_outcome(source_publication))
            .or_else(|| record_outcome(source_chunk))
            .or_else(|| record_outcome(conflicting_chunk)),
        BlobFact::Node {
            session, entries, ..
        } => session_outcome(session)
            .or_else(|| entries.iter().find_map(|edge| record_outcome(&edge.record))),
        BlobFact::Publication { session, root, .. } => {
            session_outcome(session).or_else(|| record_outcome(root))
        }
        BlobFact::Frontier {
            session,
            declaration_record,
            last_chunk_record,
            ..
        } => session_outcome(session)
            .or_else(|| record_outcome(declaration_record))
            .or_else(|| record_outcome(last_chunk_record)),
        BlobFact::DropSetManifest {
            declaration_record,
            abandoned_record,
            ..
        } => record_outcome(declaration_record).or_else(|| record_outcome(abandoned_record)),
        BlobFact::ReleasedDropSetManifest {
            publication_record, ..
        } => record_outcome(publication_record),
        BlobFact::OriginalDropReserved {
            manifest_record, ..
        } => record_outcome(manifest_record),
        BlobFact::ReclaimDescriptor {
            manifest_record, ..
        }
        | BlobFact::ReleasedReclaimDescriptor {
            manifest_record, ..
        } => record_outcome(manifest_record),
    }
}

pub(super) fn declaration<'a>(
    selected: &'a [Selected],
    sessions: &BTreeMap<[u8; 16], usize>,
    session: [u8; 16],
) -> Option<(&'a [u8; 16], &'a [u8; 16], &'a [u8; 32], &'a u32, &'a u64)> {
    let row = selected.get(*sessions.get(&session)?)?;
    if row.outcome != Outcome::Intact {
        return None;
    }
    match row.fact.as_ref()? {
        BlobFact::Declaration {
            store,
            object,
            scope,
            chunk_size,
            total,
            ..
        } => Some((store, object, scope, chunk_size, total)),
        _ => None,
    }
}
