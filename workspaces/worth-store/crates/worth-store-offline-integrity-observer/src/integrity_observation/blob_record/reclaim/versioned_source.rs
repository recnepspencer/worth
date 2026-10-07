//! Independent tag 13/14 source interpretation. The issuer digest identifies
//! evidence previously admitted by Store; these bytes cannot grant release.

use super::super::fact::ReleasedDropCustodyFact;
use super::{valid_record, BlobFact, Sha256, BASIS_DOMAIN, DROP_DOMAIN, MAXIMUM_DROPPED};

const RELEASE_DOMAIN: &[u8] = b"store.physical.released-generation-reclaim-basis.v1";

pub(crate) fn decode_manifest_v3(payload: &[u8], frame_digest: [u8; 32]) -> Option<BlobFact> {
    let store: [u8; 16] = payload.get(0..16)?.try_into().ok()?;
    let attempt: [u8; 16] = payload.get(16..32)?.try_into().ok()?;
    let source_tag = *payload.get(32)?;
    let source_len = u16::from_le_bytes(payload.get(33..35)?.try_into().ok()?) as usize;
    // Phase 6 has no issuer for tagged failed-ingest frames. Do not let a
    // syntactically valid tag 13 masquerade as the older failed-ingest path.
    if store == [0; 16] || attempt == [0; 16] || (source_tag, source_len) != (2, 324) {
        return None;
    }
    let source_end = 35usize.checked_add(source_len)?;
    let source = payload.get(35..source_end)?;
    let count =
        u16::from_le_bytes(payload.get(source_end..source_end + 2)?.try_into().ok()?) as usize;
    if count == 0 || count > MAXIMUM_DROPPED {
        return None;
    }
    let dropped_start = source_end + 34;
    let dropped_end = dropped_start.checked_add(count.checked_mul(24)?)?;
    if payload.len() != dropped_end + 9 || payload[dropped_end + 8] != 1 {
        return None;
    }
    let selected_generation =
        u64::from_le_bytes(payload[dropped_end..dropped_end + 8].try_into().ok()?);
    if selected_generation == 0 {
        return None;
    }
    let mut dropped = Vec::with_capacity(count);
    for bytes in payload[dropped_start..dropped_end].chunks_exact(24) {
        let record: [u8; 24] = bytes.try_into().ok()?;
        if !valid_record(&record) || dropped.last().is_some_and(|prior| *prior >= record) {
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
    if payload[source_end + 2..dropped_start] != dropped_hash.finish() {
        return None;
    }
    let mut basis_hash = Sha256::new();
    match source_tag {
        1 => {
            let session: [u8; 16] = source[0..16].try_into().ok()?;
            let declaration_record: [u8; 24] = source[16..40].try_into().ok()?;
            let declaration_digest: [u8; 32] = source[40..72].try_into().ok()?;
            let abandoned_record: [u8; 24] = source[72..96].try_into().ok()?;
            let abandoned_digest: [u8; 32] = source[96..128].try_into().ok()?;
            if session == [0; 16]
                || !valid_record(&declaration_record)
                || !valid_record(&abandoned_record)
                || declaration_record == abandoned_record
                || declaration_digest == [0; 32]
                || abandoned_digest == [0; 32]
                || dropped.binary_search(&declaration_record).is_ok()
                || dropped.binary_search(&abandoned_record).is_ok()
            {
                return None;
            }
            basis_hash.update(BASIS_DOMAIN);
            basis_hash.update(&store);
            basis_hash.update(source);
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
                never_reserved_slot_generation: Some(selected_generation),
            })
        }
        2 => {
            let publication_record: [u8; 24] = source[0..24].try_into().ok()?;
            let publication_digest: [u8; 32] = source[24..56].try_into().ok()?;
            let issuer_evidence_digest: [u8; 32] = source[56..88].try_into().ok()?;
            let publication_frame = &source[88..324];
            if !valid_record(&publication_record)
                || issuer_evidence_digest == [0; 32]
                || super::super::super::sha256::sha256(publication_frame) != publication_digest
            {
                return None;
            }
            let mut counters = super::super::super::OfflineIntegrityObservationCounters::default();
            let BlobFact::Publication {
                object,
                session,
                generation,
                root,
                root_digest,
                ..
            } = super::super::decode(publication_frame, Some(store), &mut counters).ok()?
            else {
                return None;
            };
            basis_hash.update(RELEASE_DOMAIN);
            basis_hash.update(&store);
            basis_hash.update(&[2]);
            basis_hash.update(source);
            Some(BlobFact::ReleasedDropSetManifest {
                store,
                frame_digest,
                attempt,
                object,
                session,
                generation,
                root,
                root_digest,
                publication_record,
                publication_digest,
                issuer_evidence_digest,
                basis_digest: basis_hash.finish(),
                dropped,
                never_reserved_slot_generation: selected_generation,
            })
        }
        _ => None,
    }
}

pub(crate) fn decode_descriptor_v2(payload: &[u8], frame_digest: [u8; 32]) -> Option<BlobFact> {
    if payload.len() != 205 {
        return None;
    }
    let store: [u8; 16] = payload[0..16].try_into().ok()?;
    let attempt: [u8; 16] = payload[16..32].try_into().ok()?;
    let source_tag = payload[32];
    let basis_digest: [u8; 32] = payload[33..65].try_into().ok()?;
    let manifest_record: [u8; 24] = payload[65..89].try_into().ok()?;
    let manifest_digest: [u8; 32] = payload[89..121].try_into().ok()?;
    let manifest_count = u16::from_le_bytes(payload[121..123].try_into().ok()?);
    let source_root = u64::from_le_bytes(payload[123..131].try_into().ok()?);
    let candidate_root = u64::from_le_bytes(payload[131..139].try_into().ok()?);
    let predecessor = match payload[139] {
        0 if payload[140..196] == [0; 56] => None,
        1 => {
            let record: [u8; 24] = payload[140..164].try_into().ok()?;
            let digest: [u8; 32] = payload[164..196].try_into().ok()?;
            if !valid_record(&record) || digest == [0; 32] {
                return None;
            }
            Some((record, digest))
        }
        _ => return None,
    };
    let cumulative_dropped = u64::from_le_bytes(payload[196..204].try_into().ok()?);
    let terminal = match payload[204] {
        0 => false,
        1 => true,
        _ => return None,
    };
    if store == [0; 16]
        || attempt == [0; 16]
        || !matches!(source_tag, 1 | 2)
        || basis_digest == [0; 32]
        || !valid_record(&manifest_record)
        || manifest_digest == [0; 32]
        || manifest_count == 0
        || usize::from(manifest_count) > MAXIMUM_DROPPED
        || source_root == 0
        || source_root.checked_add(1) != Some(candidate_root)
        || cumulative_dropped < u64::from(manifest_count)
        || (predecessor.is_none() && cumulative_dropped != u64::from(manifest_count))
        || (predecessor.is_some() && cumulative_dropped == u64::from(manifest_count))
    {
        return None;
    }
    if source_tag == 2 {
        Some(BlobFact::ReleasedReclaimDescriptor {
            store,
            frame_digest,
            attempt,
            basis_digest,
            manifest_record,
            manifest_digest,
            manifest_count,
            source_root,
            candidate_root,
            predecessor,
            cumulative_dropped,
            terminal,
            custody: None,
        })
    } else {
        None
    }
}

pub(crate) fn decode_descriptor_v3(payload: &[u8], frame_digest: [u8; 32]) -> Option<BlobFact> {
    if payload.len() != 478 || payload[205] != 1 {
        return None;
    }
    let mut descriptor = decode_descriptor_v2(&payload[..205], frame_digest)?;
    let digest_at = |index: usize| -> Option<[u8; 32]> {
        let digest: [u8; 32] = payload.get(index..index + 32)?.try_into().ok()?;
        (digest != [0; 32]).then_some(digest)
    };
    let source_root_sha256 = digest_at(206)?;
    let source_free_space_sha256 = digest_at(238)?;
    let selected_routes_sha256 = digest_at(270)?;
    let closure_sha256 = digest_at(302)?;
    let external_edges_sha256 = digest_at(334)?;
    let postorder_sha256 = digest_at(366)?;
    let idempotency = digest_at(398)?;
    let fingerprint = digest_at(430)?;
    let lease_issuance = u64::from_le_bytes(payload[462..470].try_into().ok()?);
    let lease_expiry = u64::from_le_bytes(payload[470..478].try_into().ok()?);
    if lease_expiry <= lease_issuance {
        return None;
    }
    let mut hasher = Sha256::new();
    hasher.update(b"store.physical.released-drop-custody.v1");
    hasher.update(payload);
    let BlobFact::ReleasedReclaimDescriptor { custody, .. } = &mut descriptor else {
        unreachable!("released V2 descriptor")
    };
    *custody = Some(ReleasedDropCustodyFact {
        digest: hasher.finish(),
        source_root_sha256,
        source_free_space_sha256,
        selected_routes_sha256,
        closure_sha256,
        external_edges_sha256,
        postorder_sha256,
        idempotency,
        fingerprint,
        lease_issuance,
        lease_expiry,
    });
    Some(descriptor)
}

#[cfg(test)]
#[path = "versioned_source/tests.rs"]
mod tests;
