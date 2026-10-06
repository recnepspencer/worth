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

    fn record(ordinal: u64) -> [u8; 24] {
        let mut record = [1; 24];
        record[16..].copy_from_slice(&ordinal.to_le_bytes());
        record
    }

    /// Literal kind-12 payload: store, scope, digest, chunk size, source
    /// publication and ordinal, source chunk, destination session and ordinal,
    /// and the conflicting chunk.
    fn independently_encoded_quarantine() -> Vec<u8> {
        let mut payload = Vec::with_capacity(188);
        payload.extend_from_slice(&[2; 16]);
        payload.extend_from_slice(&[3; 32]);
        payload.extend_from_slice(&[4; 32]);
        payload.extend_from_slice(&(64_u32 << 10).to_le_bytes());
        payload.extend_from_slice(&record(1));
        payload.extend_from_slice(&0_u64.to_le_bytes());
        payload.extend_from_slice(&record(2));
        payload.extend_from_slice(&[5; 16]);
        payload.extend_from_slice(&0_u64.to_le_bytes());
        payload.extend_from_slice(&record(3));
        payload
    }

    #[test]
    fn independent_parser_reads_kind12_payload_and_denies_alias() {
        let mut payload = independently_encoded_quarantine();
        assert!(matches!(
            decode(&payload),
            Some(BlobFact::DedupeQuarantine {
                store: [2, ..],
                source_ordinal: 0,
                destination_ordinal: 0,
                source_chunk,
                conflicting_chunk,
                ..
            }) if source_chunk == record(2) && conflicting_chunk == record(3)
        ));
        let aliased: [u8; 24] = payload[116..140].try_into().unwrap();
        payload[164..188].copy_from_slice(&aliased);
        assert!(decode(&payload).is_none());
    }

    // A byte-identical copy of the golden in worth-store-physical-format's
    // `tests/blob_control_record_golden.rs`, which its encoder must produce.
    const DEDUPE_QUARANTINE_V1_FRAME_HEX: &str = "5752433131424c420c010100bc000000edfb1a903ae2d0ebf5418e9eec893721608fd86e050b17cebda7f256fd0962770202020202020202020202020202020203030303030303030303030303030303030303030303030303030303030303030404040404040404040404040404040404040404040404040404040404040404000001000101010101010101010101010101010101000000000000000000000000000000010101010101010101010101010101010200000000000000050505050505050505050505050505050000000000000000010101010101010101010101010101010300000000000000";

    #[test]
    fn encoder_golden_is_the_literal_layout_and_parses_as_a_whole_frame() {
        let frame = super::super::tests::golden(DEDUPE_QUARANTINE_V1_FRAME_HEX);
        assert_eq!(frame[48..], independently_encoded_quarantine());
        let mut counters = crate::OfflineIntegrityObservationCounters::default();
        assert!(matches!(
            super::super::decode(&frame, Some([2; 16]), &mut counters),
            Ok(BlobFact::DedupeQuarantine { store: [2, ..], .. })
        ));
    }
}
