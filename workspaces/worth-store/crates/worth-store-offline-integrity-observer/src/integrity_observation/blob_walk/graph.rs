use std::collections::BTreeMap;

use super::super::blob_record::{BlobEdge, BlobFact};
use super::super::OfflineIntegrityOutcome as Outcome;
use super::super::OfflinePhysicalDamageCause as Cause;
use super::proof::Proof;
use super::row_index::RowIndex;
use super::Selected;

/// Classify every selected chunk against its session's declaration before a
/// frontier summarizes any prefix. Selected rows have no ordinal order, so
/// this must precede frontier indexing.
pub(super) fn validate_chunks(selected: &mut [Selected], index: &RowIndex) {
    for row in 0..selected.len() {
        if selected[row].outcome != Outcome::Intact {
            continue;
        }
        let Some(BlobFact::Chunk {
            store,
            session,
            ordinal,
            chunk_size,
            length,
            ..
        }) = selected[row].fact.as_ref()
        else {
            continue;
        };
        let size = u64::from(*chunk_size);
        let expected_length = |total: &u64| {
            ordinal
                .checked_mul(size)
                .and_then(|start| total.checked_sub(start))
                .map(|remaining| remaining.min(size))
        };
        let declared = Proof::on_row(selected, index.declaration(session), |declaration| {
            matches!(declaration, BlobFact::Declaration {
                store: declared_store, chunk_size: declared_size, total, ..
            } if declared_store == store
                && declared_size == chunk_size
                && expected_length(total) == Some(*length))
        });
        selected[row].outcome = declared.outcome(Cause::ScopeMismatch);
    }
}

/// The position a tree node gives one of its edges.
pub(super) struct EdgePosition<'a> {
    pub(super) session: &'a [u8; 16],
    pub(super) kind: u8,
    pub(super) level: u8,
    /// Chunk ordinal under a leaf, child node index under a branch.
    pub(super) index: Option<u64>,
}

/// A branch edge names the next-lower node of its session. A leaf edge names
/// the selected row that claims that ordinal for its session: the session's
/// own chunk frame, or its reuse claim, whose authenticated source edge
/// `reuse_claims` checks. It never names another session's chunk.
pub(super) fn edge_names_child(
    position: &EdgePosition<'_>,
    edge: &BlobEdge,
    child: &BlobFact,
) -> bool {
    match (position.kind, child) {
        (
            1,
            BlobFact::Chunk {
                session,
                ordinal,
                length,
                digest,
                ..
            }
            | BlobFact::ReuseClaim {
                session,
                ordinal,
                length,
                digest,
                ..
            },
        ) => {
            session == position.session
                && Some(*ordinal) == position.index
                && *length == edge.covered
                && *digest == edge.digest
        }
        (
            2,
            BlobFact::Node {
                session,
                level,
                index,
                covered,
                digest,
                ..
            },
        ) => {
            session == position.session
                && level.checked_add(1) == Some(position.level)
                && Some(*index) == position.index
                && *covered == edge.covered
                && *digest == edge.digest
        }
        _ => false,
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

/// The outcome of the first row `fact` names that is undecided, or that the
/// walk cannot say is absent. The checks that follow this gate read the rows
/// by record and by session without asking again: once the gate passes, a row
/// they do not find is absent from the store.
pub(super) fn uncertain_dependency(
    fact: &BlobFact,
    selected: &[Selected],
    index: &RowIndex,
) -> Option<Outcome> {
    let outcome_of = |sought: Result<usize, Proof>| match sought {
        Ok(row) => dependency_uncertainty(&selected[row]),
        Err(Proof::Undecided(outcome)) => Some(outcome),
        Err(Proof::Holds | Proof::Contradicted) => None,
    };
    let record_outcome = |record: &[u8; 24]| outcome_of(index.record(record));
    // A declaration that a frame names by session alone.
    let declared_outcome = |session: &[u8; 16]| outcome_of(index.declaration(session));
    // The session's declaration when the frame also names it by record: the
    // record lookup says what its absence means.
    let session_outcome = |session: &[u8; 16]| {
        let declaration = index.sessions.get(session);
        declaration.and_then(|row| dependency_uncertainty(&selected[*row]))
    };
    match fact {
        BlobFact::Declaration { .. } => None,
        BlobFact::Abandoned {
            session,
            declaration_record,
            ..
        } => session_outcome(session).or_else(|| record_outcome(declaration_record)),
        // A chunk, a reuse claim and a dedupe quarantine weigh the rows they
        // could not observe against the rows that contradict them in their
        // own proofs.
        BlobFact::Chunk { .. }
        | BlobFact::ReuseClaim { .. }
        | BlobFact::DedupeQuarantine { .. } => None,
        BlobFact::Node {
            session, entries, ..
        } => declared_outcome(session)
            .or_else(|| entries.iter().find_map(|edge| record_outcome(&edge.record))),
        BlobFact::Publication { session, root, .. } => {
            declared_outcome(session).or_else(|| record_outcome(root))
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

#[cfg(test)]
mod tests;

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
