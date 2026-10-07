use std::collections::BTreeMap;

use super::super::blob_record::{BlobFact, FrameRole, ReuseSourceWitness};
use super::super::OfflinePhysicalDamageCause as Cause;
use super::super::OfflineUnknownPhysicalReason as Unknown;
use super::proof::Proof;
use super::row_index::RowIndex;
use super::source_edge::{self, SourceClaim};
use super::{damage, Outcome, Selected};

/// Check a selected claim against both the destination declaration and a
/// selected source publication's exact tree edge. A derived dedupe leaf is
/// deliberately absent from this proof.
pub(super) fn validate(selected: &mut [Selected], rows: &RowIndex) {
    let mut claims: BTreeMap<([u8; 16], u64), usize> = BTreeMap::new();
    for index in 0..selected.len() {
        let Some(BlobFact::ReuseClaim {
            session, ordinal, ..
        }) = selected[index].fact.as_ref()
        else {
            continue;
        };
        let duplicate = claims.insert((*session, *ordinal), index);
        if let Some(prior) = duplicate {
            selected[prior].outcome = damage(Cause::DuplicateIdentity);
            selected[index].outcome = damage(Cause::DuplicateIdentity);
            continue;
        }
        if selected[index].outcome != Outcome::Intact {
            continue;
        }
        // Row generation is a current placement generation, not an immutable
        // append timestamp. Mutation-time C.5 root fencing proves chronology.
        let proof = claim_proof(selected, rows, index);
        selected[index].outcome = proof.outcome(Cause::ScopeMismatch);
    }
}

fn claim_proof(selected: &[Selected], rows: &RowIndex, index: usize) -> Proof {
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
    }) = selected[index].fact.as_ref()
    else {
        return Proof::Contradicted;
    };
    let destination = Proof::on_row(selected, rows.declaration(session), |declared| {
        let expected = |total: &u64| {
            ordinal
                .checked_mul(u64::from(*chunk_size))
                .and_then(|start| total.checked_sub(start))
                .map(|remaining| remaining.min(u64::from(*chunk_size)))
        };
        matches!(declared, BlobFact::Declaration {
            store: declared_store, scope: declared_scope, chunk_size: declared_size, total, ..
        } if declared_store == store
            && declared_scope == scope
            && declared_size == chunk_size
            && expected(total) == Some(*length))
    });
    let claim = SourceClaim {
        publication: *source_publication,
        ordinal: *source_ordinal,
        chunk: *chunk_record,
        digest: *digest,
        length: Some(*length),
        scope: *scope,
        chunk_size: *chunk_size,
    };
    let witness = source_witness.as_ref();
    let source = if rows.records.contains_key(source_publication) {
        witnessed_publication(selected, rows, &claim, witness)
            .and(source_edge::proof(selected, rows, &claim, witness))
    } else {
        // No row answers for the source publication. In a walk that visited
        // every routed record only a release can have removed it; in a walk
        // cut short, each row consulted here may be one it did not visit.
        // Checked: the borrowed chunk is selected and is the occurrence that
        // the claim, and its witness if it carries one, state; and a selected
        // release manifest names the publication as witnessed. Not checked:
        // the leaf edge, the root digest and the source declaration against
        // the witness. That tree proof is not done, so a released source is
        // never intact.
        source_edge::chunk_unpublished(selected, rows, &claim, witness)
            .and(released_source(selected, rows, &claim, witness))
    };
    destination.and(source)
}

/// A claim's own copy of its source publication against the selected
/// publication itself.
fn witnessed_publication(
    selected: &[Selected],
    rows: &RowIndex,
    claim: &SourceClaim,
    witness: Option<&ReuseSourceWitness>,
) -> Proof {
    let Some(witness) = witness else {
        return Proof::Holds;
    };
    Proof::on_row(selected, rows.record(&claim.publication), |publication| {
        matches!(publication, BlobFact::Publication {
            store, frame_digest, session, object, generation,
            root, root_digest, total, chunk_size, scope, ..
        } if *store == witness.store
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
}

/// What selected release custody says of a source publication that no
/// selected row answers for. Only a release removes a publication, and that
/// release cannot end while a selected claim protects a chunk of the
/// generation, so a manifest that names the publication stays selected. A
/// claim that no such manifest answers is contradicted, unless a row that the
/// walk could not read or did not visit may be that manifest.
fn released_source(
    selected: &[Selected],
    rows: &RowIndex,
    claim: &SourceClaim,
    witness: Option<&ReuseSourceWitness>,
) -> Proof {
    let mut manifests = selected
        .iter()
        .filter(|row| names_source(row, claim, witness))
        .peekable();
    if manifests.peek().is_none() {
        return rows.none_found(FrameRole::ReleaseManifest);
    }
    let removed_publication = |manifest: &Selected| {
        matches!(manifest.fact.as_ref(),
            Some(BlobFact::ReleasedDropSetManifest { dropped, .. })
                if dropped.binary_search(&claim.publication).is_ok()
                    && !dropped.contains(&claim.chunk))
    };
    // An unwitnessed (kind 11) claim on a released source cannot occur in the
    // producer, which defers the release while an unwitnessed claim exists.
    // This branch goes away with kind 11.
    let removal_selected = witness.is_some()
        && manifests.any(|manifest| {
            removed_publication(manifest) && first_descriptor_selected(selected, manifest)
        });
    Proof::Undecided(Outcome::Unknown(if removal_selected {
        // Selected records alone do not establish the WAL member that
        // removed the publication, nor its original tree edge.
        Unknown::WalCoverageUnavailable
    } else {
        Unknown::ParentScopeUnavailable
    }))
}

/// Whether `row` is a release manifest of the claim's source publication, as
/// the claim witnessed that publication if it carries a witness.
fn names_source(row: &Selected, claim: &SourceClaim, witness: Option<&ReuseSourceWitness>) -> bool {
    let Some(BlobFact::ReleasedDropSetManifest {
        store,
        object,
        session,
        generation,
        root,
        root_digest,
        publication_record,
        publication_digest,
        ..
    }) = row.fact.as_ref()
    else {
        return false;
    };
    !matches!(row.outcome, Outcome::Damaged(_))
        && *publication_record == claim.publication
        && witness.is_none_or(|witness| {
            *publication_digest == witness.frame_digest
                && *store == witness.store
                && *object == witness.object
                && *session == witness.session
                && *generation == witness.generation
                && *root == witness.root
                && *root_digest == witness.root_digest
        })
}

/// Whether the descriptor that opened a release with `manifest` is selected.
fn first_descriptor_selected(selected: &[Selected], manifest: &Selected) -> bool {
    let Some(BlobFact::ReleasedDropSetManifest {
        store,
        frame_digest,
        attempt,
        basis_digest,
        dropped,
        ..
    }) = manifest.fact.as_ref()
    else {
        return false;
    };
    selected.iter().any(|row| {
        matches!(row.fact.as_ref(),
            Some(BlobFact::ReleasedReclaimDescriptor {
                store: descriptor_store, attempt: descriptor_attempt,
                basis_digest: descriptor_basis, manifest_record, manifest_digest,
                manifest_count, predecessor: None, ..
            }) if !matches!(row.outcome, Outcome::Damaged(_))
                && descriptor_store == store
                && descriptor_attempt == attempt
                && descriptor_basis == basis_digest
                && *manifest_record == manifest.record
                && manifest_digest == frame_digest
                && usize::from(*manifest_count) == dropped.len())
    })
}

#[cfg(test)]
#[path = "reuse_claims/released_tests.rs"]
mod released_tests;
#[cfg(test)]
#[path = "reuse_claims/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "reuse_claims/unseen_tests.rs"]
mod unseen_tests;
