use super::fact::{BlobFact, ReuseSourceWitness};
use super::primitives::{admitted_chunk_size, nonzero, u32_at, u64_at};

/// Independent literal parser for the selected destination reference claim.
/// Cross-record and scope authority is checked by the offline graph walk.
pub(super) fn decode(payload: &[u8]) -> Option<BlobFact> {
    if payload.len() != 168 {
        return None;
    }
    let store: [u8; 16] = payload[..16].try_into().ok()?;
    let session: [u8; 16] = payload[16..32].try_into().ok()?;
    let scope: [u8; 32] = payload[40..72].try_into().ok()?;
    let digest: [u8; 32] = payload[80..112].try_into().ok()?;
    let chunk_record: [u8; 24] = payload[112..136].try_into().ok()?;
    let source_publication: [u8; 24] = payload[136..160].try_into().ok()?;
    let chunk_epoch: [u8; 16] = chunk_record[..16].try_into().ok()?;
    let publication_epoch: [u8; 16] = source_publication[..16].try_into().ok()?;
    let chunk_size = u32_at(payload, 72);
    let length = u32_at(payload, 76);
    if !nonzero(&store)
        || !nonzero(&session)
        || !nonzero(&scope)
        || !nonzero(&digest)
        || !nonzero(&chunk_epoch)
        || !nonzero(&publication_epoch)
        || u64_at(&chunk_record, 16) == 0
        || u64_at(&source_publication, 16) == 0
        || !admitted_chunk_size(chunk_size)
        || length == 0
        || length > chunk_size
    {
        return None;
    }
    Some(BlobFact::ReuseClaim {
        store,
        session,
        ordinal: u64_at(payload, 32),
        scope,
        chunk_size,
        length: u64::from(length),
        digest,
        chunk_record,
        source_publication,
        source_ordinal: u64_at(payload, 160),
        source_witness: None,
    })
}

pub(super) fn decode_v2(
    payload: &[u8],
    counters: &mut crate::OfflineIntegrityObservationCounters,
) -> Option<BlobFact> {
    if payload.len() != 168 + 32 + 236 {
        return None;
    }
    let mut claim = decode(&payload[..168])?;
    let BlobFact::ReuseClaim {
        store,
        scope,
        chunk_size,
        length,
        source_ordinal,
        ..
    } = &claim
    else {
        return None;
    };
    let (store, scope, chunk_size, length, source_ordinal) =
        (*store, *scope, *chunk_size, *length, *source_ordinal);
    let publication_frame = &payload[200..];
    let BlobFact::Publication {
        store: publication_store,
        frame_digest,
        session,
        object,
        generation,
        root,
        root_digest,
        total,
        chunk_size: publication_size,
        scope: publication_scope,
        ..
    } = super::decode(publication_frame, Some(store), counters).ok()?
    else {
        return None;
    };
    let expected_start = source_ordinal.checked_mul(u64::from(chunk_size))?;
    if payload[168..200] != frame_digest
        || publication_store != store
        || publication_scope != scope
        || publication_size != chunk_size
        || expected_start >= total
        || length != (total - expected_start).min(u64::from(chunk_size))
    {
        return None;
    }
    let BlobFact::ReuseClaim { source_witness, .. } = &mut claim else {
        unreachable!("decoded V1 claim")
    };
    *source_witness = Some(ReuseSourceWitness {
        frame_digest,
        store,
        session,
        object,
        generation,
        root,
        root_digest,
        total,
        chunk_size,
        scope,
    });
    Some(claim)
}

#[cfg(test)]
mod tests {
    use super::super::tests::{control_frame, golden, rehash};
    use super::*;
    use crate::integrity_observation::sha256::sha256;

    const CHUNK: u32 = 64 << 10;
    // Byte-identical copies of the goldens in worth-store-physical-format's
    // `tests/blob_control_record_golden.rs`, which its encoder must produce.
    const REUSE_CLAIM_V1_FRAME_HEX: &str = "5752433131424c420b010100a80000000a1601cf025dabe87098a24d1133bf460152a9c09f2d9296ade59ae6cf343df6010101010101010101010101010101010202020202020202020202020202020207000000000000000303030303030303030303030303030303030303030303030303030303030303000001000000010004040404040404040404040404040404040404040404040404040404040404040505050505050505050505050505050506000000000000000707070707070707070707070707070708000000000000000000000000000000";
    const REUSE_CLAIM_V2_FRAME_HEX: &str = "5752433131424c420f010100b40100006a666da631f38b480e8e2701a6f56018fbbd2a0184b416aad35e9a1c74db6b360101010101010101010101010101010102020202020202020202020202020202000000000000000003030303030303030303030303030303030303030303030303030303030303030000010000000100040404040404040404040404040404040404040404040404040404040404040407070707070707070707070707070707060000000000000007070707070707070707070707070707090000000000000000000000000000001b2475c7d7b109d3d4e7afda0af5a55848b76e7a5367ac749fc019da4fd247835752433131424c4204010000bc000000008500f2dbd62317aa4d9b340e60645c805c8f5b6e245fb89d606ad78d549ade0101010101010101010101010101010109090909090909090909090909090909040404040404040404040404040404040100000000000000070707070707070707070707070707070800000000000000060606060606060606060606060606060606060606060606060606060606060600000100000000000505050505050505050505050505050505050505050505050505050505050505000001000303030303030303030303030303030303030303030303030303030303030303";

    fn record(epoch: u8, ordinal: u64) -> [u8; 24] {
        let mut record = [epoch; 24];
        record[16..].copy_from_slice(&ordinal.to_le_bytes());
        record
    }

    /// Literal kind-11 payload: store, session, ordinal, scope, chunk size,
    /// length, digest, chunk record, source publication, source ordinal.
    fn independently_encoded_claim(ordinal: u64, chunk: [u8; 24], source: [u8; 24]) -> Vec<u8> {
        let mut payload = Vec::with_capacity(168);
        payload.extend_from_slice(&[1; 16]);
        payload.extend_from_slice(&[2; 16]);
        payload.extend_from_slice(&ordinal.to_le_bytes());
        payload.extend_from_slice(&[3; 32]);
        payload.extend_from_slice(&CHUNK.to_le_bytes());
        payload.extend_from_slice(&CHUNK.to_le_bytes());
        payload.extend_from_slice(&[4; 32]);
        payload.extend_from_slice(&chunk);
        payload.extend_from_slice(&source);
        payload.extend_from_slice(&0_u64.to_le_bytes());
        payload
    }

    /// Literal kind-4 frame for one single-chunk generation in store `[1; 16]`.
    fn independently_encoded_publication() -> Vec<u8> {
        let mut payload = Vec::with_capacity(188);
        payload.extend_from_slice(&[1; 16]);
        payload.extend_from_slice(&[9; 16]);
        payload.extend_from_slice(&[4; 16]);
        payload.extend_from_slice(&1_u64.to_le_bytes());
        payload.extend_from_slice(&record(7, 8));
        payload.extend_from_slice(&[6; 32]);
        payload.extend_from_slice(&u64::from(CHUNK).to_le_bytes());
        payload.extend_from_slice(&[5; 32]);
        payload.extend_from_slice(&CHUNK.to_le_bytes());
        payload.extend_from_slice(&[3; 32]);
        control_frame(4, &payload)
    }

    #[test]
    fn independent_parser_reads_literal_claim_payload() {
        let payload = independently_encoded_claim(7, record(5, 6), record(7, 8));
        assert!(matches!(
            decode(&payload),
            Some(BlobFact::ReuseClaim {
                ordinal: 7,
                source_ordinal: 0,
                chunk_record,
                source_publication,
                ..
            }) if chunk_record == record(5, 6) && source_publication == record(7, 8)
        ));
    }

    #[test]
    fn encoder_golden_is_the_literal_layout_and_parses_as_a_whole_frame() {
        let frame = golden(REUSE_CLAIM_V1_FRAME_HEX);
        assert_eq!(
            frame[48..],
            independently_encoded_claim(7, record(5, 6), record(7, 8))
        );
        let mut counters = crate::OfflineIntegrityObservationCounters::default();
        assert!(matches!(
            super::super::decode(&frame, Some([1; 16]), &mut counters),
            Ok(BlobFact::ReuseClaim {
                ordinal: 7,
                source_witness: None,
                ..
            })
        ));
    }

    #[test]
    fn versioned_claim_requires_canonical_embedded_publication() {
        let publication = independently_encoded_publication();
        let digest = sha256(&publication);
        let mut payload = independently_encoded_claim(0, record(7, 6), record(7, 9));
        payload.extend_from_slice(&digest);
        payload.extend_from_slice(&publication);
        let mut encoded = control_frame(15, &payload);
        encoded[10..12].copy_from_slice(&1_u16.to_le_bytes());
        rehash(&mut encoded);
        assert_eq!(encoded, golden(REUSE_CLAIM_V2_FRAME_HEX));
        let mut counters = crate::OfflineIntegrityObservationCounters::default();
        assert!(
            matches!(super::super::decode(&encoded, Some([1; 16]), &mut counters),
            Ok(BlobFact::ReuseClaim {
                source_witness: Some(ReuseSourceWitness { frame_digest, .. }), ..
            }) if frame_digest == digest)
        );
        // A resealed frame whose embedded publication digest is wrong is
        // malformed, not a checksum failure.
        let mut changed = encoded;
        changed[48 + 168] ^= 1;
        rehash(&mut changed);
        assert!(matches!(
            super::super::decode(&changed, Some([1; 16]), &mut counters),
            Err(super::super::Outcome::Damaged(_))
        ));
    }
}
