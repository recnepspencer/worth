use super::fact::BlobFact;
use super::primitives::{admitted_chunk_size, nonzero, u32_at, u64_at};

/// Literal parser independent of the production kind-12 codec. Selected
/// source, conflicting occurrence, and unequal bytes are checked by the walk.
pub(super) fn decode(payload: &[u8]) -> Option<BlobFact> {
    if payload.len() != 188 {
        return None;
    }
    let store: [u8; 16] = payload[..16].try_into().ok()?;
    let scope: [u8; 32] = payload[16..48].try_into().ok()?;
    let digest: [u8; 32] = payload[48..80].try_into().ok()?;
    let source_publication: [u8; 24] = payload[84..108].try_into().ok()?;
    let source_chunk: [u8; 24] = payload[116..140].try_into().ok()?;
    let destination_session: [u8; 16] = payload[140..156].try_into().ok()?;
    let conflicting_chunk: [u8; 24] = payload[164..188].try_into().ok()?;
    let valid_record =
        |record: &[u8; 24]| record[..16].iter().any(|byte| *byte != 0) && u64_at(record, 16) != 0;
    let chunk_size = u32_at(payload, 80);
    if !nonzero(&store)
        || !nonzero(&scope)
        || !nonzero(&digest)
        || !nonzero(&destination_session)
        || !admitted_chunk_size(chunk_size)
        || !valid_record(&source_publication)
        || !valid_record(&source_chunk)
        || !valid_record(&conflicting_chunk)
        || source_chunk == conflicting_chunk
    {
        return None;
    }
    Some(BlobFact::DedupeQuarantine {
        store,
        scope,
        digest,
        chunk_size,
        source_publication,
        source_ordinal: u64_at(payload, 108),
        source_chunk,
        destination_session,
        destination_ordinal: u64_at(payload, 156),
        conflicting_chunk,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_store_physical_format::{BlobDedupeQuarantineV1, PersistedRecordIdentity};

    fn record(ordinal: u64) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([1; 16], ordinal).unwrap()
    }

    #[test]
    fn independent_parser_matches_kind12_vector_and_denies_alias() {
        let value = BlobDedupeQuarantineV1::new(
            [2; 16],
            [3; 32],
            [4; 32],
            64 << 10,
            record(1),
            0,
            record(2),
            [5; 16],
            0,
            record(3),
        )
        .unwrap();
        assert!(matches!(
            decode(&value.encode()[48..]),
            Some(BlobFact::DedupeQuarantine {
                source_ordinal: 0,
                destination_ordinal: 0,
                ..
            })
        ));
        let mut payload = value.encode()[48..].to_vec();
        let aliased: [u8; 24] = payload[116..140].try_into().unwrap();
        payload[164..188].copy_from_slice(&aliased);
        assert!(decode(&payload).is_none());
    }
}
