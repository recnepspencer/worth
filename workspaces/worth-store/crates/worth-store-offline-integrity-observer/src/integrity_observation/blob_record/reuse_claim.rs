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
    use worth_store_physical_format::{
        BlobChunkReuseClaimV1, BlobChunkReuseClaimV2, BlobGenerationPublicationV1,
        PersistedRecordIdentity,
    };

    use super::*;

    #[test]
    fn independent_parser_reads_codec_vector() {
        let claim = BlobChunkReuseClaimV1::new(
            [1; 16],
            [2; 16],
            7,
            [3; 32],
            64 << 10,
            64 << 10,
            [4; 32],
            PersistedRecordIdentity::new([5; 16], 6).unwrap(),
            PersistedRecordIdentity::new([7; 16], 8).unwrap(),
            9,
        )
        .unwrap();
        let bytes = claim.encode();
        assert!(matches!(
            decode(&bytes[48..]),
            Some(BlobFact::ReuseClaim {
                ordinal: 7,
                source_ordinal: 9,
                ..
            })
        ));
    }

    #[test]
    fn versioned_claim_requires_canonical_embedded_publication() {
        let record = |ordinal| PersistedRecordIdentity::new([7; 16], ordinal).unwrap();
        let publication = BlobGenerationPublicationV1::new(
            [1; 16],
            [9; 16],
            [4; 16],
            1,
            record(8),
            [6; 32],
            64 << 10,
            [5; 32],
            64 << 10,
            [3; 32],
        )
        .unwrap();
        let base = BlobChunkReuseClaimV1::new(
            [1; 16],
            [2; 16],
            0,
            [3; 32],
            64 << 10,
            64 << 10,
            [4; 32],
            record(6),
            record(9),
            0,
        )
        .unwrap();
        let digest = crate::integrity_observation::sha256::sha256(&publication.encode());
        let encoded = BlobChunkReuseClaimV2::new(base, publication, digest)
            .unwrap()
            .encode();
        let mut counters = crate::OfflineIntegrityObservationCounters::default();
        assert!(
            matches!(super::super::decode(&encoded, Some([1; 16]), &mut counters),
            Ok(BlobFact::ReuseClaim {
                source_witness: Some(ReuseSourceWitness { frame_digest, .. }), ..
            }) if frame_digest == digest)
        );
        let mut changed = encoded;
        changed[48 + 168] ^= 1;
        assert!(super::super::decode(&changed, Some([1; 16]), &mut counters).is_err());
    }
}
