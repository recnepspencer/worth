use std::collections::BTreeMap;

use super::super::blob_record::{BlobFact, ReuseSourceWitness};
use super::super::OfflinePhysicalDamageCause as Cause;
use super::super::OfflineUnknownPhysicalReason as Unknown;
use super::{damage, declaration, Outcome, Selected};

pub(super) type ReuseClaims = BTreeMap<([u8; 16], u64), usize>;

pub(super) fn selected_reuse_edge(
    row: &Selected,
    edge: &super::super::blob_record::BlobEdge,
) -> Option<Outcome> {
    matches!(row.fact.as_ref(), Some(BlobFact::ReuseClaim {
        chunk_record, digest, length, ..
    }) if chunk_record == &edge.record && digest == &edge.digest && length == &edge.covered)
    .then(|| row.outcome.clone())
}

/// Check a selected claim against both the destination declaration and a
/// selected source publication's exact tree edge. A derived dedupe leaf is
/// deliberately absent from this proof.
pub(super) fn validate(
    selected: &mut [Selected],
    sessions: &BTreeMap<[u8; 16], usize>,
    records: &BTreeMap<[u8; 24], usize>,
) -> ReuseClaims {
    let mut claims = ReuseClaims::new();
    for index in 0..selected.len() {
        let Some(BlobFact::ReuseClaim {
            store,
            session,
            ordinal,
            scope,
            chunk_size,
            length,
            digest,
            chunk_record,
            source_publication,
            source_ordinal,
            source_witness,
        }) = selected[index].fact.as_ref().cloned()
        else {
            continue;
        };
        let duplicate = claims.insert((session, ordinal), index);
        if let Some(prior) = duplicate {
            selected[prior].outcome = damage(Cause::DuplicateIdentity);
            selected[index].outcome = damage(Cause::DuplicateIdentity);
            continue;
        }
        if selected[index].outcome != Outcome::Intact {
            continue;
        }
        let destination_valid = declaration(selected, sessions, session).is_some_and(
            |(declared_store, _, declared_scope, declared_size, declared_total)| {
                let expected = ordinal
                    .checked_mul(u64::from(chunk_size))
                    .and_then(|start| declared_total.checked_sub(start))
                    .map(|remaining| remaining.min(u64::from(chunk_size)));
                *declared_store == store
                    && *declared_scope == scope
                    && *declared_size == chunk_size
                    && expected == Some(length)
            },
        );
        if !destination_valid {
            selected[index].outcome = damage(Cause::ScopeMismatch);
            continue;
        }
        if !records.contains_key(&source_publication) {
            selected[index].outcome = released_source_provenance(
                selected,
                source_publication,
                chunk_record,
                source_witness,
            );
            continue;
        }
        if source_witness.is_some_and(|witness| {
            !records.get(&source_publication).is_some_and(|source| {
                matches!(selected[*source].fact.as_ref(), Some(BlobFact::Publication {
                    store, frame_digest, session, object, generation,
                    root, root_digest, total, chunk_size, scope, ..
                }) if *store == witness.store
                    && *frame_digest == witness.frame_digest
                    && *session == witness.session
                    && *object == witness.object
                    && *generation == witness.generation
                    && *root == witness.root
                    && *root_digest == witness.root_digest
                    && *total == witness.total
                    && *chunk_size == witness.chunk_size
                    && *scope == witness.scope)
            })
        }) {
            selected[index].outcome = damage(Cause::ScopeMismatch);
            continue;
        }
        let source_valid = source_edge(
            selected,
            sessions,
            records,
            source_publication,
            source_ordinal,
            chunk_record,
            digest,
            length,
            scope,
            chunk_size,
        );
        // Row generation is a current placement generation, not an immutable
        // append timestamp. Mutation-time C.5 root fencing proves chronology.
        if !source_valid {
            selected[index].outcome = damage(Cause::ScopeMismatch);
        }
    }
    claims
}

fn released_source_provenance(
    selected: &[Selected],
    source_publication: [u8; 24],
    chunk_record: [u8; 24],
    witness: Option<ReuseSourceWitness>,
) -> Outcome {
    let Some(witness) = witness else {
        return Outcome::Unknown(Unknown::ParentScopeUnavailable);
    };
    let matching = selected.iter().filter_map(|row| {
        let Some(BlobFact::ReleasedDropSetManifest {
            store,
            frame_digest,
            attempt,
            basis_digest,
            dropped,
            publication_record,
            publication_digest,
            object,
            session,
            generation,
            root,
            root_digest,
            ..
        }) = row.fact.as_ref()
        else {
            return None;
        };
        (matches!(
            row.outcome,
            Outcome::Intact | Outcome::Unknown(Unknown::ParentScopeUnavailable)
        ) && publication_record == &source_publication
            && publication_digest == &witness.frame_digest
            && store == &witness.store
            && object == &witness.object
            && session == &witness.session
            && generation == &witness.generation
            && root == &witness.root
            && root_digest == &witness.root_digest
            && dropped.binary_search(&source_publication).is_ok()
            && !dropped.contains(&chunk_record))
        .then_some((
            row.record,
            *store,
            *frame_digest,
            *attempt,
            *basis_digest,
            dropped.len(),
        ))
    });
    for (record, store, digest, attempt, basis, count) in matching {
        if selected.iter().any(|row| matches!(row.fact.as_ref(),
            Some(BlobFact::ReleasedReclaimDescriptor {
                store: descriptor_store, attempt: descriptor_attempt,
                basis_digest, manifest_record, manifest_digest,
                manifest_count, predecessor: None, ..
            }) if matches!(row.outcome, Outcome::Intact | Outcome::Unknown(Unknown::WalCoverageUnavailable))
                && *descriptor_store == store
                && descriptor_attempt == &attempt
                && basis_digest == &basis
                && manifest_record == &record
                && manifest_digest == &digest
                && usize::from(*manifest_count) == count
        )) {
            // Selected records alone do not establish the WAL member that
            // removed the publication, nor its original tree edge.
            return Outcome::Unknown(Unknown::WalCoverageUnavailable);
        }
    }
    Outcome::Unknown(Unknown::ParentScopeUnavailable)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn source_edge(
    selected: &[Selected],
    sessions: &BTreeMap<[u8; 16], usize>,
    records: &BTreeMap<[u8; 24], usize>,
    publication_record: [u8; 24],
    source_ordinal: u64,
    chunk_record: [u8; 24],
    digest: [u8; 32],
    length: u64,
    scope: [u8; 32],
    chunk_size: u32,
) -> bool {
    let Some(pub_row) = records
        .get(&publication_record)
        .map(|index| &selected[*index])
    else {
        return false;
    };
    let Some(BlobFact::Publication {
        store,
        session,
        root,
        root_digest,
        total,
        chunk_size: source_size,
        scope: source_scope,
        ..
    }) = pub_row.fact.as_ref()
    else {
        return false;
    };
    if pub_row.outcome != Outcome::Intact || *source_scope != scope || *source_size != chunk_size {
        return false;
    }
    let source_declaration = declaration(selected, sessions, *session).is_some_and(
        |(declared_store, _, declared_scope, declared_size, declared_total)| {
            declared_store == store
                && *declared_scope == scope
                && *declared_size == chunk_size
                && declared_total == total
        },
    );
    if !source_declaration {
        return false;
    }
    let Some(mut offset) = source_ordinal.checked_mul(u64::from(chunk_size)) else {
        return false;
    };
    if offset >= *total || length != (*total - offset).min(u64::from(chunk_size)) {
        return false;
    }
    let mut node_record = *root;
    let mut expected_digest = *root_digest;
    for depth in 0..7 {
        let Some(row) = records.get(&node_record).map(|index| &selected[*index]) else {
            return false;
        };
        let Some(BlobFact::Node {
            session: node_session,
            kind,
            covered,
            digest: node_digest,
            frame_digest,
            entries,
            ..
        }) = row.fact.as_ref()
        else {
            return false;
        };
        if row.outcome != Outcome::Intact
            || node_session != session
            || (if depth == 0 {
                frame_digest
            } else {
                node_digest
            }) != &expected_digest
            || offset >= *covered
        {
            return false;
        }
        let mut selected_edge = None;
        for edge in entries {
            if offset < edge.covered {
                selected_edge = Some(edge);
                break;
            }
            offset -= edge.covered;
        }
        let Some(edge) = selected_edge else {
            return false;
        };
        if *kind == 1 {
            return edge.record == chunk_record
                && edge.digest == digest
                && edge.covered == length
                && offset == 0
                && records.get(&chunk_record).is_some_and(|index| {
                    selected[*index].outcome == Outcome::Intact
                        && matches!(selected[*index].fact.as_ref(), Some(BlobFact::Chunk {
                        store: chunk_store, chunk_size: size, length: bytes,
                        digest: stored, session: chunk_session, ordinal: chunk_ordinal,
                    }) if chunk_store == store && *size == chunk_size
                        && *bytes == length && *stored == digest
                        && chunk_session == session && *chunk_ordinal == source_ordinal)
                });
        }
        if *kind != 2 {
            return false;
        }
        node_record = edge.record;
        expected_digest = edge.digest;
    }
    false
}

#[cfg(test)]
#[path = "reuse_claims/tests.rs"]
mod tests;
