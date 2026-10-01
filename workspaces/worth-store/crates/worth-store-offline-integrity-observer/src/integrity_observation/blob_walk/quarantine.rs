use std::collections::BTreeMap;
use std::path::Path;

use super::super::blob_record::{self, BlobFact};
use super::super::{BoundedMediaWalk, OfflinePhysicalDamageCause as Cause};
use super::{damage, graph, logical_digest, reuse_claims, Outcome, Selected};

/// A kind-12 frame is intact only when its source key and both selected
/// original occurrences are proven and the chunks have different bytes. This
/// proof reopens only the two routed chunks and retains no global byte cache.
pub(super) fn validate(rows: &mut [Selected], root: &Path, walk: &mut BoundedMediaWalk) {
    let records: BTreeMap<[u8; 24], usize> = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.fact.is_some())
        .map(|(index, row)| (row.record, index))
        .collect();
    let sessions: BTreeMap<[u8; 16], usize> = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| match row.fact.as_ref() {
            Some(BlobFact::Declaration { session, .. }) => Some((*session, index)),
            _ => None,
        })
        .collect();
    for index in 0..rows.len() {
        if rows[index].outcome != Outcome::Intact
            || !matches!(
                rows[index].fact.as_ref(),
                Some(BlobFact::DedupeQuarantine { .. })
            )
        {
            continue;
        }
        rows[index].outcome = outcome(rows, &sessions, &records, index, root, walk);
    }
}

fn outcome(
    rows: &[Selected],
    sessions: &BTreeMap<[u8; 16], usize>,
    records: &BTreeMap<[u8; 24], usize>,
    index: usize,
    root: &Path,
    walk: &mut BoundedMediaWalk,
) -> Outcome {
    let Some(BlobFact::DedupeQuarantine {
        store,
        scope,
        digest,
        chunk_size,
        source_publication,
        source_ordinal,
        source_chunk,
        destination_session,
        destination_ordinal,
        conflicting_chunk,
    }) = rows[index].fact.as_ref()
    else {
        return damage(Cause::Framing);
    };
    let Some(source_index) = records.get(source_chunk).copied() else {
        return damage(Cause::Pointer);
    };
    let Some(conflict_index) = records.get(conflicting_chunk).copied() else {
        return damage(Cause::Pointer);
    };
    let Some(publication_index) = records.get(source_publication).copied() else {
        return damage(Cause::Pointer);
    };
    for dependency in [publication_index, source_index, conflict_index] {
        if rows[dependency].outcome != Outcome::Intact {
            return graph::dependency_uncertainty(&rows[dependency])
                .unwrap_or_else(|| damage(Cause::Pointer));
        }
    }
    let destination_valid = graph::declaration(rows, sessions, *destination_session).is_some_and(
        |(declared_store, _, declared_scope, declared_size, declared_total)| {
            declared_store == store
                && declared_scope == scope
                && declared_size == chunk_size
                && destination_ordinal
                    .checked_mul(u64::from(*chunk_size))
                    .and_then(|start| declared_total.checked_sub(start))
                    .is_some_and(|remaining| remaining > 0)
        },
    );
    let Some(BlobFact::Chunk {
        store: source_store,
        length: source_length,
        digest: source_digest,
        ..
    }) = rows[source_index].fact.as_ref()
    else {
        return damage(Cause::Pointer);
    };
    let conflict_valid = matches!(rows[conflict_index].fact.as_ref(), Some(BlobFact::Chunk {
        store: conflict_store, session, ordinal, chunk_size: size,
        length, ..
    }) if conflict_store == store && session == destination_session
        && ordinal == destination_ordinal && size == chunk_size
        && destination_ordinal.checked_mul(u64::from(*chunk_size))
            .and_then(|start| graph::declaration(rows, sessions, *destination_session)
                .and_then(|(_, _, _, _, total)| total.checked_sub(start)))
            .map(|remaining| remaining.min(u64::from(*chunk_size))) == Some(*length));
    if !destination_valid
        || !conflict_valid
        || source_store != store
        || source_digest != digest
        || !reuse_claims::source_edge(
            rows,
            sessions,
            records,
            *source_publication,
            *source_ordinal,
            *source_chunk,
            *digest,
            *source_length,
            *scope,
            *chunk_size,
        )
    {
        return damage(Cause::Pointer);
    }
    let source = match selected_chunk_bytes(&rows[source_index], root, walk) {
        Ok(bytes) => bytes,
        Err(outcome) => return outcome,
    };
    let conflicting = match selected_chunk_bytes(&rows[conflict_index], root, walk) {
        Ok(bytes) => bytes,
        Err(outcome) => return outcome,
    };
    if source == conflicting {
        damage(Cause::ScopeMismatch)
    } else {
        Outcome::Intact
    }
}

fn selected_chunk_bytes(
    row: &Selected,
    root: &Path,
    walk: &mut BoundedMediaWalk,
) -> Result<Vec<u8>, Outcome> {
    let payload = logical_digest::reread_selected_chunk(row, root, walk)?;
    let decoded = blob_record::decode(
        &payload,
        row.fact.as_ref().map(BlobFact::store),
        walk.counters_mut(),
    )?;
    if row.fact.as_ref() != Some(&decoded) || payload.len() < 140 {
        return Err(damage(Cause::ChecksumMismatch));
    }
    Ok(payload[140..].to_vec())
}
