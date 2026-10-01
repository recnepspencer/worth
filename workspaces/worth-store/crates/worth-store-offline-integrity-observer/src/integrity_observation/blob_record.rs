//! Independent C.11 inner-frame reader. This module intentionally does not
//! import the physical-format blob parser or C.9 validator.

use super::sha256::Sha256;
use super::{
    OfflineIntegrityObservationCounters, OfflineIntegrityOutcome as Outcome,
    OfflinePhysicalBlastRadius as Blast, OfflinePhysicalDamageCause as Cause,
    OfflineUnknownPhysicalReason,
};

mod abandoned;
mod fact;
mod frontier;
mod primitives;
mod quarantine;
mod reclaim;
mod reuse_claim;
#[cfg(test)]
mod tests;

pub(crate) use fact::{BlobEdge, BlobFact, ReuseSourceWitness};
use primitives::{admitted_chunk_size, nonzero, u32_at, u64_at};

const MAGIC: &[u8; 8] = b"WRC11BLB";
const HEADER: usize = 48;
const CHUNK_MAX: usize = 1 << 20;
const NODE_MAX: usize = 512 << 10;
const CONTROL_MAX: usize = 64 << 10;

pub(crate) fn is_blob_prefix(bytes: &[u8]) -> bool {
    bytes.starts_with(MAGIC)
}

pub(crate) fn decode(
    bytes: &[u8],
    expected_store: Option<[u8; 16]>,
    counters: &mut OfflineIntegrityObservationCounters,
) -> Result<BlobFact, Outcome> {
    let fail =
        |cause| super::record_walk::damage(cause, Some((0, bytes.len() as u64)), Blast::Artifact);
    if bytes.len() < HEADER || !is_blob_prefix(bytes) {
        return Err(fail(Cause::Framing));
    }
    let kind = bytes[8];
    let maximum = match kind {
        1 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 => CONTROL_MAX,
        2 => CHUNK_MAX,
        3 => NODE_MAX,
        _ => return Err(fail(Cause::Framing)),
    };
    if bytes[9] != 1
        || bytes.len() > maximum
        || bytes[12..16] != ((bytes.len() - HEADER) as u32).to_le_bytes()
    {
        return Err(fail(Cause::Framing));
    }
    let flags = u16::from_le_bytes(bytes[10..12].try_into().expect("fixed header"));
    if flags
        != if kind == 2 || kind == 3 || kind == 11 || kind == 12 || kind == 15 {
            1
        } else {
            0
        }
    {
        return Err(fail(Cause::MalformedPayload));
    }
    let payload = &bytes[HEADER..];
    let mut hasher = Sha256::new();
    hasher.update(&bytes[..16]);
    hasher.update(payload);
    counters.checksum_calculations += 1;
    if bytes[16..48] != hasher.finish() {
        return Err(fail(Cause::ChecksumMismatch));
    }
    if matches!(kind, 13 | 14) && payload.get(32) == Some(&1) {
        // The versioned failed-ingest source is format-valid but has no Store
        // writer or independent offline semantic admission in this phase.
        return Err(Outcome::Unknown(
            OfflineUnknownPhysicalReason::ParentScopeUnavailable,
        ));
    }
    let fact = match kind {
        1 => {
            counters.checksum_calculations += 1;
            declaration(payload, super::sha256::sha256(bytes))
                .ok_or_else(|| fail(Cause::MalformedPayload))?
        }
        2 => chunk(payload, counters).ok_or_else(|| fail(Cause::MalformedPayload))?,
        3 => {
            counters.checksum_calculations += 1;
            node(payload, counters, super::sha256::sha256(bytes))
                .ok_or_else(|| fail(Cause::MalformedPayload))?
        }
        4 => publication(payload, super::sha256::sha256(bytes))
            .ok_or_else(|| fail(Cause::MalformedPayload))?,
        5 => frontier::decode(payload).ok_or_else(|| fail(Cause::MalformedPayload))?,
        6 => abandoned::decode(payload, super::sha256::sha256(bytes))
            .ok_or_else(|| fail(Cause::MalformedPayload))?,
        7 => reclaim::decode_manifest(payload, super::sha256::sha256(bytes))
            .ok_or_else(|| fail(Cause::MalformedPayload))?,
        8 => reclaim::decode_descriptor(payload).ok_or_else(|| fail(Cause::MalformedPayload))?,
        9 => reclaim::decode_manifest_v2(payload, super::sha256::sha256(bytes))
            .ok_or_else(|| fail(Cause::MalformedPayload))?,
        10 => reclaim::decode_original_drop_reserved(payload, super::sha256::sha256(bytes))
            .ok_or_else(|| fail(Cause::MalformedPayload))?,
        11 => reuse_claim::decode(payload).ok_or_else(|| fail(Cause::MalformedPayload))?,
        12 => quarantine::decode(payload).ok_or_else(|| fail(Cause::MalformedPayload))?,
        13 => reclaim::decode_manifest_v3(payload, super::sha256::sha256(bytes))
            .ok_or_else(|| fail(Cause::MalformedPayload))?,
        14 => reclaim::decode_descriptor_v2(payload, super::sha256::sha256(bytes))
            .ok_or_else(|| fail(Cause::MalformedPayload))?,
        15 => reuse_claim::decode_v2(payload, counters)
            .ok_or_else(|| fail(Cause::MalformedPayload))?,
        16 => reclaim::decode_descriptor_v3(payload, super::sha256::sha256(bytes))
            .ok_or_else(|| fail(Cause::MalformedPayload))?,
        _ => unreachable!("known kind"),
    };
    let Some(expected_store) = expected_store else {
        return Err(Outcome::Unknown(
            OfflineUnknownPhysicalReason::StoreIdentityUnavailable,
        ));
    };
    if fact.store() == [0; 16]
        || fact.session().is_some_and(|session| session == [0; 16])
        || expected_store != fact.store()
    {
        return Err(fail(Cause::ScopeMismatch));
    }
    Ok(fact)
}

fn declaration(p: &[u8], frame_digest: [u8; 32]) -> Option<BlobFact> {
    if p.len() != 108 {
        return None;
    }
    let object = p[32..48].try_into().ok()?;
    let scope = p[48..80].try_into().ok()?;
    let chunk_size = u32_at(p, 80);
    let total = u64_at(p, 84);
    if !nonzero(&object)
        || !nonzero(&scope)
        || !admitted_chunk_size(chunk_size)
        || total == 0
        || u64_at(p, 92) == 0
        || u64_at(p, 100) == 0
    {
        return None;
    }
    Some(BlobFact::Declaration {
        store: p[..16].try_into().ok()?,
        session: p[16..32].try_into().ok()?,
        frame_digest,
        object,
        scope,
        chunk_size,
        total,
        max_checkpoint_sequence: u64_at(p, 100),
    })
}

fn chunk(p: &[u8], counters: &mut OfflineIntegrityObservationCounters) -> Option<BlobFact> {
    if p.len() < 92 {
        return None;
    }
    let chunk_size = u32_at(p, 40);
    let length = u32_at(p, 44);
    let digest: [u8; 32] = p[48..80].try_into().ok()?;
    let content = &p[80..];
    if !admitted_chunk_size(chunk_size)
        || length == 0
        || length > chunk_size
        || content.len() != 12 + length as usize
        || content[..4] != [1, 0, 0, 0]
        || u32_at(content, 4) != chunk_size
        || u32_at(content, 8) != length
        || !nonzero(&digest)
    {
        return None;
    }
    counters.checksum_calculations += 1;
    if super::sha256::sha256(content) != digest {
        return None;
    }
    Some(BlobFact::Chunk {
        store: p[..16].try_into().ok()?,
        session: p[16..32].try_into().ok()?,
        ordinal: u64_at(p, 32),
        chunk_size,
        length: u64::from(length),
        digest,
    })
}

fn node(
    p: &[u8],
    counters: &mut OfflineIntegrityObservationCounters,
    frame_digest: [u8; 32],
) -> Option<BlobFact> {
    if p.len() < 88 {
        return None;
    }
    let content = &p[80..];
    let kind = content[0];
    let level = content[1];
    let count = u32_at(content, 4) as usize;
    if !((kind == 1 && level == 0) || (kind == 2 && level > 0))
        || content[2..4] != [0; 2]
        || count == 0
        || count > 4096
        || content.len() != 8 + count * 64
    {
        return None;
    }
    let digest: [u8; 32] = p[48..80].try_into().ok()?;
    if !nonzero(&digest) {
        return None;
    }
    counters.checksum_calculations += 1;
    if super::sha256::sha256(content) != digest {
        return None;
    }
    let mut entries = Vec::with_capacity(count);
    let mut covered = 0_u64;
    for entry in content[8..].chunks_exact(64) {
        let edge = BlobEdge {
            digest: entry[..32].try_into().ok()?,
            record: entry[32..56].try_into().ok()?,
            covered: u64_at(entry, 56),
        };
        let epoch: [u8; 16] = edge.record[..16].try_into().ok()?;
        if !nonzero(&edge.digest)
            || !nonzero(&epoch)
            || u64_at(&edge.record, 16) == 0
            || edge.covered == 0
        {
            return None;
        }
        covered = covered.checked_add(edge.covered)?;
        entries.push(edge);
    }
    if u64_at(p, 40) != covered {
        return None;
    }
    Some(BlobFact::Node {
        store: p[..16].try_into().ok()?,
        session: p[16..32].try_into().ok()?,
        kind,
        level,
        index: u64_at(p, 32),
        covered,
        digest,
        frame_digest,
        entries,
    })
}

fn publication(p: &[u8], frame_digest: [u8; 32]) -> Option<BlobFact> {
    if p.len() != 188 {
        return None;
    }
    let object = p[32..48].try_into().ok()?;
    let root: [u8; 24] = p[56..80].try_into().ok()?;
    let root_digest = p[80..112].try_into().ok()?;
    let scope = p[156..188].try_into().ok()?;
    let generation = u64_at(p, 48);
    let total = u64_at(p, 112);
    let chunk_size = u32_at(p, 152);
    let epoch: [u8; 16] = root[..16].try_into().ok()?;
    let logical_digest: [u8; 32] = p[120..152].try_into().ok()?;
    if !nonzero(&object)
        || !nonzero(&epoch)
        || u64_at(&root, 16) == 0
        || !nonzero(&root_digest)
        || !nonzero(&logical_digest)
        || !nonzero(&scope)
        || generation == 0
        || total == 0
        || !admitted_chunk_size(chunk_size)
    {
        return None;
    }
    Some(BlobFact::Publication {
        store: p[..16].try_into().ok()?,
        frame_digest,
        session: p[16..32].try_into().ok()?,
        object,
        generation,
        root,
        root_digest,
        total,
        logical_digest,
        chunk_size,
        scope,
    })
}
