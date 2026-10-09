use std::collections::BTreeMap;

use super::super::blob_record::FrameRole;
use super::super::OfflineUnknownPhysicalReason as Unknown;
use super::graph::dependency_uncertainty;
use super::proof::Proof;
use super::row_index::RowIndex;
use super::{damage, BlobFact, Cause, Outcome, Selected};

#[cfg(test)]
mod tests;

/// Selected custody is checked here; a manifest without a descriptor is not a drop.
pub(super) fn validate(
    rows: &mut [Selected],
    found: &RowIndex,
    selected_root_generation: Option<u64>,
) {
    let records = &found.records;
    for index in 0..rows.len() {
        if rows[index].outcome != Outcome::Intact {
            continue;
        }
        if matches!(
            rows[index].fact.as_ref(),
            Some(BlobFact::DropSetManifest { .. })
        ) {
            rows[index].outcome = manifest_outcome(rows, records, index, selected_root_generation);
        }
    }
    let mut reservations: BTreeMap<[u8; 24], usize> = BTreeMap::new();
    for index in 0..rows.len() {
        if rows[index].outcome != Outcome::Intact {
            continue;
        }
        let Some(BlobFact::OriginalDropReserved {
            manifest_record, ..
        }) = rows[index].fact.as_ref()
        else {
            continue;
        };
        if let Some(prior) = reservations.insert(*manifest_record, index) {
            rows[prior].outcome = damage(Cause::DuplicateIdentity);
            rows[index].outcome = damage(Cause::DuplicateIdentity);
        }
    }
    for index in 0..rows.len() {
        if rows[index].outcome == Outcome::Intact
            && matches!(
                rows[index].fact.as_ref(),
                Some(BlobFact::OriginalDropReserved { .. })
            )
        {
            rows[index].outcome =
                reservation_outcome(rows, records, index, selected_root_generation);
        }
    }
    for index in 0..rows.len() {
        if rows[index].outcome == Outcome::Intact
            && matches!(
                rows[index].fact.as_ref(),
                Some(BlobFact::ReclaimDescriptor { .. })
            )
        {
            rows[index].outcome = descriptor_outcome(rows, found, &reservations, index);
        }
    }
}

fn manifest_outcome(
    rows: &[Selected],
    records: &BTreeMap<[u8; 24], usize>,
    index: usize,
    selected_root_generation: Option<u64>,
) -> Outcome {
    let Some(BlobFact::DropSetManifest {
        store,
        session,
        declaration_record,
        declaration_digest,
        abandoned_record,
        abandoned_digest,
        never_reserved_slot_generation,
        ..
    }) = rows[index].fact.as_ref()
    else {
        return damage(Cause::Framing);
    };
    let declaration = records
        .get(declaration_record)
        .and_then(|row| rows.get(*row));
    let abandoned = records.get(abandoned_record).and_then(|row| rows.get(*row));
    let declaration_valid = declaration.is_some_and(|row| {
        row.outcome == Outcome::Intact
            && matches!(row.fact.as_ref(),
                Some(BlobFact::Declaration { store: found_store, session: found_session,
                    frame_digest, .. })
                if found_store == store && found_session == session
                    && frame_digest == declaration_digest)
    });
    let abandoned_valid = abandoned.is_some_and(|row| {
        row.outcome == Outcome::Intact
            && matches!(row.fact.as_ref(),
                Some(BlobFact::Abandoned { store: found_store, session: found_session,
                    frame_digest, declaration_record: found_declaration,
                    declaration_digest: found_digest, .. })
                if found_store == store && found_session == session
                    && frame_digest == abandoned_digest
                    && found_declaration == declaration_record
                    && found_digest == declaration_digest)
    });
    if !declaration_valid || !abandoned_valid {
        return damage(Cause::Pointer);
    }
    if never_reserved_slot_generation
        .zip(selected_root_generation)
        .is_some_and(|(slot, selected)| slot > selected)
    {
        return damage(Cause::Pointer);
    }
    Outcome::Intact
}

fn reservation_outcome(
    rows: &[Selected],
    records: &BTreeMap<[u8; 24], usize>,
    index: usize,
    selected_root_generation: Option<u64>,
) -> Outcome {
    let Some(BlobFact::OriginalDropReserved {
        store,
        attempt,
        manifest_record,
        manifest_digest,
        basis_digest,
        manifest_selected_generation,
        reserved_selected_generation,
        ..
    }) = rows[index].fact.as_ref()
    else {
        return damage(Cause::Framing);
    };
    let Some(selected) = selected_root_generation else {
        return Outcome::Unknown(Unknown::SelectorUnavailable);
    };
    if *reserved_selected_generation > selected || *manifest_record == rows[index].record {
        return damage(Cause::Pointer);
    }
    let Some(manifest) = records.get(manifest_record).and_then(|row| rows.get(*row)) else {
        return damage(Cause::Pointer);
    };
    if manifest.outcome != Outcome::Intact {
        return manifest.outcome.clone();
    }
    match manifest.fact.as_ref() {
        Some(BlobFact::DropSetManifest {
            store: manifest_store,
            frame_digest,
            attempt: manifest_attempt,
            basis_digest: manifest_basis,
            never_reserved_slot_generation: Some(slot_generation),
            ..
        }) if store == manifest_store
            && attempt == manifest_attempt
            && manifest_digest == frame_digest
            && basis_digest == manifest_basis
            && manifest_selected_generation == slot_generation =>
        {
            Outcome::Intact
        }
        _ => damage(Cause::Pointer),
    }
}

fn descriptor_outcome(
    rows: &[Selected],
    found: &RowIndex,
    reservations: &BTreeMap<[u8; 24], usize>,
    index: usize,
) -> Outcome {
    let Some(BlobFact::ReclaimDescriptor {
        store,
        attempt,
        basis_digest,
        manifest_record,
        manifest_digest,
        manifest_count,
        source_root,
        ..
    }) = rows[index].fact.as_ref()
    else {
        return damage(Cause::Framing);
    };
    if manifest_record == &rows[index].record {
        return damage(Cause::Pointer);
    }
    let records = &found.records;
    let Some(manifest) = records.get(manifest_record).and_then(|row| rows.get(*row)) else {
        return damage(Cause::Pointer);
    };
    if manifest.outcome != Outcome::Intact {
        return manifest.outcome.clone();
    }
    let Some(BlobFact::DropSetManifest {
        store: manifest_store,
        frame_digest,
        attempt: manifest_attempt,
        basis_digest: manifest_basis,
        dropped,
        never_reserved_slot_generation,
        ..
    }) = manifest.fact.as_ref()
    else {
        return damage(Cause::Pointer);
    };
    if store != manifest_store
        || attempt != manifest_attempt
        || basis_digest != manifest_basis
        || manifest_digest != frame_digest
        || usize::from(*manifest_count) != dropped.len()
    {
        return damage(Cause::Pointer);
    }
    let mut proof = found.all_gone(dropped);
    if never_reserved_slot_generation.is_some() {
        let reserved = reservations
            .get(manifest_record)
            .and_then(|index| rows.get(*index));
        let Some(reserved) = reserved else {
            // No reservation that was read names the manifest.
            let proof = proof.and(found.none_found(FrameRole::DropReservation));
            return proof.outcome(Cause::Pointer);
        };
        let exact = matches!(reserved.fact.as_ref(),
            Some(BlobFact::OriginalDropReserved {
                store: reserved_store,
                attempt: reserved_attempt,
                manifest_record: reserved_manifest,
                manifest_digest: reserved_digest,
                basis_digest: reserved_basis,
                reserved_selected_generation,
                ..
            }) if reserved_store == store
                && reserved_attempt == attempt
                && reserved_manifest == manifest_record
                && reserved_digest == manifest_digest
                && reserved_basis == basis_digest
                && reserved_selected_generation == source_root);
        if !exact {
            return damage(Cause::Pointer);
        }
        // A matching reservation that is itself undecided decides nothing.
        if reserved.outcome != Outcome::Intact {
            let undecided = dependency_uncertainty(reserved);
            proof = proof.and(undecided.map_or(Proof::Contradicted, Proof::Undecided));
        }
    }
    proof.outcome(Cause::Pointer)
}
