use super::{nonzero, u64_at, BlobFact};

/// Independent fixed-field decode; no runtime/format parser is imported.
pub(super) fn decode(p: &[u8]) -> Option<BlobFact> {
    if p.len() != 160 {
        return None;
    }
    let declaration_record: [u8; 24] = p[32..56].try_into().ok()?;
    let declaration_digest: [u8; 32] = p[56..88].try_into().ok()?;
    let last_chunk_record: [u8; 24] = p[104..128].try_into().ok()?;
    let last_chunk_digest: [u8; 32] = p[128..160].try_into().ok()?;
    if !valid_record(&declaration_record)
        || !valid_record(&last_chunk_record)
        || declaration_record == last_chunk_record
        || !nonzero(&declaration_digest)
        || !nonzero(&last_chunk_digest)
        || u64_at(p, 88) == 0
        || u64_at(p, 96) == 0
    {
        return None;
    }
    Some(BlobFact::Frontier {
        store: p[..16].try_into().ok()?,
        session: p[16..32].try_into().ok()?,
        declaration_record,
        declaration_digest,
        next_chunk_ordinal: u64_at(p, 88),
        durable_bytes: u64_at(p, 96),
        last_chunk_record,
        last_chunk_digest,
    })
}

fn valid_record(record: &[u8; 24]) -> bool {
    let epoch: [u8; 16] = record[..16].try_into().expect("fixed record");
    nonzero(&epoch) && u64_at(record, 16) != 0
}
