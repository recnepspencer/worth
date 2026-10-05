use std::path::Path;

use super::super::blob_record::{self, BlobFact};
use super::super::{BoundedMediaWalk, OfflinePhysicalDamageCause as Cause};
use super::coverage::Coverage;
use super::proof::{needed_row, Needed, Proof};
use super::row_index::RowIndex;
use super::source_edge::{self, SourceClaim};
use super::{damage, logical_digest, Outcome, Selected};

/// A kind-12 frame is intact only when its source key and both selected
/// original occurrences are proven and the chunks have different bytes. This
/// proof reopens only the two routed chunks and retains no global byte cache.
pub(super) fn validate(
    rows: &mut [Selected],
    coverage: &Coverage,
    root: &Path,
    walk: &mut BoundedMediaWalk,
) {
    validate_reading(rows, coverage, &mut |row| {
        selected_chunk_bytes(row, root, walk)
    });
}

type ChunkBytes<'a> = dyn FnMut(&Selected) -> Result<Vec<u8>, Outcome> + 'a;

fn validate_reading(rows: &mut [Selected], coverage: &Coverage, chunk_bytes: &mut ChunkBytes) {
    // Every selected row, as the claim graph indexes them: a row that could not
    // be read still answers for its record with its own outcome.
    let found = RowIndex::new(rows, coverage);
    for index in 0..rows.len() {
        if rows[index].outcome != Outcome::Intact {
            continue;
        }
        let Some(fact @ BlobFact::DedupeQuarantine { .. }) = rows[index].fact.as_ref() else {
            continue;
        };
        let outcome = outcome(rows, &found, fact, chunk_bytes);
        rows[index].outcome = outcome;
    }
}

fn outcome(
    rows: &[Selected],
    found: &RowIndex,
    fact: &BlobFact,
    chunk_bytes: &mut ChunkBytes,
) -> Outcome {
    let BlobFact::DedupeQuarantine {
        source_chunk,
        conflicting_chunk,
        ..
    } = fact
    else {
        return damage(Cause::Framing);
    };
    let graph = graph_proof(rows, found, fact);
    // Rows that are not the two occurrences the quarantine names have no
    // bytes to compare: the contradiction stands however a reread would end.
    if graph == Proof::Contradicted {
        return damage(Cause::Pointer);
    }
    // The byte comparison reads only the two chunks, so it is decided whenever
    // both are intact, whatever else could not be observed.
    let intact = |record: &[u8; 24]| {
        let row = found.records.get(record).map(|index| &rows[*index]);
        row.filter(|row| row.outcome == Outcome::Intact)
    };
    let (Some(source), Some(conflicting)) = (intact(source_chunk), intact(conflicting_chunk))
    else {
        return graph.outcome(Cause::Pointer);
    };
    let source = match chunk_bytes(source) {
        Ok(bytes) => bytes,
        Err(outcome) => return outcome,
    };
    let conflicting = match chunk_bytes(conflicting) {
        Ok(bytes) => bytes,
        Err(outcome) => return outcome,
    };
    if source == conflicting {
        damage(Cause::ScopeMismatch)
    } else {
        graph.outcome(Cause::Pointer)
    }
}

/// Everything a quarantine says of selected rows short of their bytes: its
/// destination declaration, the conflicting chunk as that destination's own
/// occurrence, and the source chunk as an occurrence of the source generation.
fn graph_proof(rows: &[Selected], found: &RowIndex, fact: &BlobFact) -> Proof {
    let BlobFact::DedupeQuarantine {
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
    } = fact
    else {
        return Proof::Contradicted;
    };
    let size = u64::from(*chunk_size);
    let remaining = |total: u64| {
        destination_ordinal
            .checked_mul(size)
            .and_then(|start| total.checked_sub(start))
    };
    let declaration = || found.declaration(destination_session);
    let destination = Proof::on_row(rows, declaration(), |declared| {
        matches!(declared, BlobFact::Declaration {
            store: declared_store, scope: declared_scope, chunk_size: declared_size, total, ..
        } if declared_store == store
            && declared_scope == scope
            && declared_size == chunk_size
            && remaining(*total).is_some_and(|bytes| bytes > 0))
    });
    // Only a destination declaration whose frame was read decides the
    // conflicting chunk's length.
    let declared_total = match needed_row(rows, declaration()) {
        Ok(Needed {
            fact: BlobFact::Declaration { total, .. },
            ..
        }) => Some(*total),
        _ => None,
    };
    let conflicting = Proof::on_row(rows, found.record(conflicting_chunk), |chunk| {
        matches!(chunk, BlobFact::Chunk {
            store: conflict_store, session, ordinal, chunk_size: conflict_size, length, ..
        } if conflict_store == store
            && session == destination_session
            && ordinal == destination_ordinal
            && conflict_size == chunk_size
            && declared_total.is_none_or(|total| {
                remaining(total).map(|bytes| bytes.min(size)) == Some(*length)
            }))
    });
    let source_store = Proof::on_row(rows, found.record(source_chunk), |chunk| {
        chunk.store() == *store
    });
    let source = SourceClaim {
        publication: *source_publication,
        ordinal: *source_ordinal,
        chunk: *source_chunk,
        digest: *digest,
        length: None,
        scope: *scope,
        chunk_size: *chunk_size,
    };
    destination
        .and(conflicting)
        .and(source_store)
        .and(source_edge::proof(rows, found, &source, None))
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

#[cfg(test)]
#[path = "quarantine/tests.rs"]
mod tests;
