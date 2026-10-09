//! Independent interpretation of C.11 reclaim records, without runtime APIs.

use super::super::sha256::Sha256;
use super::BlobFact;

#[path = "reclaim/versioned_source.rs"]
mod versioned_source;
pub(super) use versioned_source::{decode_descriptor_v2, decode_descriptor_v3, decode_manifest_v3};

const BASIS_DOMAIN: &[u8] = b"store.physical.failed-ingest-reclaim-basis.v1";
const DROP_DOMAIN: &[u8] = b"store.physical.blob-drop-set-identities.v1";
const MAXIMUM_DROPPED: usize = 1024;

pub(super) fn decode_manifest(payload: &[u8], frame_digest: [u8; 32]) -> Option<BlobFact> {
    if payload.len() < 194 {
        return None;
    }
    let store: [u8; 16] = payload[..16].try_into().ok()?;
    let attempt: [u8; 16] = payload[16..32].try_into().ok()?;
    let session: [u8; 16] = payload[32..48].try_into().ok()?;
    let declaration_record: [u8; 24] = payload[48..72].try_into().ok()?;
    let declaration_digest: [u8; 32] = payload[72..104].try_into().ok()?;
    let abandoned_record: [u8; 24] = payload[104..128].try_into().ok()?;
    let abandoned_digest: [u8; 32] = payload[128..160].try_into().ok()?;
    let count = u16::from_le_bytes(payload[160..162].try_into().ok()?) as usize;
    if count == 0
        || count > MAXIMUM_DROPPED
        || payload.len() != 194 + 24 * count
        || attempt == [0; 16]
        || !valid_record(&declaration_record)
        || !valid_record(&abandoned_record)
        || declaration_record == abandoned_record
        || declaration_digest == [0; 32]
        || abandoned_digest == [0; 32]
    {
        return None;
    }
    let mut dropped = Vec::with_capacity(count);
    for bytes in payload[194..].chunks_exact(24) {
        let record: [u8; 24] = bytes.try_into().ok()?;
        if !valid_record(&record)
            || record == declaration_record
            || record == abandoned_record
            || dropped.last().is_some_and(|prior| *prior >= record)
        {
            return None;
        }
        dropped.push(record);
    }
    let mut dropped_hash = Sha256::new();
    dropped_hash.update(DROP_DOMAIN);
    dropped_hash.update(&(count as u16).to_le_bytes());
    for record in &dropped {
        dropped_hash.update(record);
    }
    if payload[162..194] != dropped_hash.finish() {
        return None;
    }
    let mut basis_hash = Sha256::new();
    basis_hash.update(BASIS_DOMAIN);
    basis_hash.update(&store);
    basis_hash.update(&payload[32..160]);
    Some(BlobFact::DropSetManifest {
        store,
        frame_digest,
        attempt,
        session,
        declaration_record,
        declaration_digest,
        abandoned_record,
        abandoned_digest,
        basis_digest: basis_hash.finish(),
        dropped,
        never_reserved_slot_generation: None,
    })
}

/// Tag 9 appends one sealed initial slot to the v1 drop-set representation.
/// Its presence does not itself prove no later reservation was selected.
pub(super) fn decode_manifest_v2(payload: &[u8], frame_digest: [u8; 32]) -> Option<BlobFact> {
    let split = payload.len().checked_sub(9)?;
    let selected_generation = u64::from_le_bytes(payload[split..split + 8].try_into().ok()?);
    if selected_generation == 0 || payload[split + 8] != 1 {
        return None;
    }
    let BlobFact::DropSetManifest {
        store,
        attempt,
        session,
        declaration_record,
        declaration_digest,
        abandoned_record,
        abandoned_digest,
        basis_digest,
        dropped,
        ..
    } = decode_manifest(&payload[..split], frame_digest)?
    else {
        return None;
    };
    Some(BlobFact::DropSetManifest {
        store,
        frame_digest,
        attempt,
        session,
        declaration_record,
        declaration_digest,
        abandoned_record,
        abandoned_digest,
        basis_digest,
        dropped,
        never_reserved_slot_generation: Some(selected_generation),
    })
}

/// Tag 10 is a selected control record, not a drop descriptor or a license
/// inferred from the manifest's initial slot.
pub(super) fn decode_original_drop_reserved(
    payload: &[u8],
    frame_digest: [u8; 32],
) -> Option<BlobFact> {
    if payload.len() != 216 {
        return None;
    }
    let store: [u8; 16] = payload[..16].try_into().ok()?;
    let attempt: [u8; 16] = payload[16..32].try_into().ok()?;
    let manifest_record: [u8; 24] = payload[32..56].try_into().ok()?;
    let manifest_digest: [u8; 32] = payload[56..88].try_into().ok()?;
    let basis_digest: [u8; 32] = payload[88..120].try_into().ok()?;
    let manifest_selected_generation = u64::from_le_bytes(payload[120..128].try_into().ok()?);
    let reserved_selected_generation = u64::from_le_bytes(payload[128..136].try_into().ok()?);
    let idempotency: [u8; 32] = payload[136..168].try_into().ok()?;
    let fingerprint: [u8; 32] = payload[168..200].try_into().ok()?;
    let lease_issuance_generation = u64::from_le_bytes(payload[200..208].try_into().ok()?);
    let lease_expiry_generation = u64::from_le_bytes(payload[208..216].try_into().ok()?);
    if store == [0; 16]
        || attempt == [0; 16]
        || !valid_record(&manifest_record)
        || manifest_digest == [0; 32]
        || basis_digest == [0; 32]
        || manifest_selected_generation == 0
        || reserved_selected_generation <= manifest_selected_generation
        || idempotency == [0; 32]
        || fingerprint == [0; 32]
        || lease_expiry_generation <= lease_issuance_generation
    {
        return None;
    }
    Some(BlobFact::OriginalDropReserved {
        store,
        frame_digest,
        attempt,
        manifest_record,
        manifest_digest,
        basis_digest,
        manifest_selected_generation,
        reserved_selected_generation,
        idempotency,
        fingerprint,
        lease_issuance_generation,
        lease_expiry_generation,
    })
}

pub(super) fn decode_descriptor(payload: &[u8]) -> Option<BlobFact> {
    if payload.len() != 138 {
        return None;
    }
    let store: [u8; 16] = payload[..16].try_into().ok()?;
    let attempt: [u8; 16] = payload[16..32].try_into().ok()?;
    let basis_digest: [u8; 32] = payload[32..64].try_into().ok()?;
    let manifest_record: [u8; 24] = payload[64..88].try_into().ok()?;
    let manifest_digest: [u8; 32] = payload[88..120].try_into().ok()?;
    let manifest_count = u16::from_le_bytes(payload[120..122].try_into().ok()?);
    let source_root = u64::from_le_bytes(payload[122..130].try_into().ok()?);
    let candidate_root = u64::from_le_bytes(payload[130..138].try_into().ok()?);
    if attempt == [0; 16]
        || basis_digest == [0; 32]
        || manifest_digest == [0; 32]
        || !valid_record(&manifest_record)
        || manifest_count == 0
        || usize::from(manifest_count) > MAXIMUM_DROPPED
        || source_root == 0
        || source_root.checked_add(1) != Some(candidate_root)
    {
        return None;
    }
    Some(BlobFact::ReclaimDescriptor {
        store,
        attempt,
        basis_digest,
        manifest_record,
        manifest_digest,
        manifest_count,
        source_root,
        candidate_root,
    })
}

fn valid_record(record: &[u8; 24]) -> bool {
    record[..16] != [0; 16] && record[16..24] != [0; 8]
}
