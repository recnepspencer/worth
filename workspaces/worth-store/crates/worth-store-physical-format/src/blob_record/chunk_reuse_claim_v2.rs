use sha2::{Digest, Sha256};

use super::chunk_reuse_claim::BlobChunkReuseClaimV1;
use super::envelope::{encode, BLOB_RECORD_HEADER_BYTES};
use super::generation::GENERATION_PUBLICATION_FRAME_BYTES;
use super::{BlobGenerationPublicationV1, BlobRecordDenial, BlobRecordKind};

const V1_PAYLOAD_BYTES: usize = 168;
const PAYLOAD_BYTES: usize = V1_PAYLOAD_BYTES + 32 + GENERATION_PUBLICATION_FRAME_BYTES;

/// A destination occurrence whose source publication was authenticated when
/// the claim was issued. The selected claim retains this canonical witness
/// after a proved source release un-routes the publication record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobChunkReuseClaimV2 {
    claim: BlobChunkReuseClaimV1,
    source_publication: BlobGenerationPublicationV1,
    source_publication_frame_sha256: [u8; 32],
}

impl BlobChunkReuseClaimV2 {
    pub fn new(
        claim: BlobChunkReuseClaimV1,
        source_publication: BlobGenerationPublicationV1,
        source_publication_frame_sha256: [u8; 32],
    ) -> Result<Self, BlobRecordDenial> {
        let canonical = source_publication.encode();
        let observed: [u8; 32] = Sha256::digest(canonical).into();
        let start = claim
            .source_ordinal()
            .checked_mul(u64::from(claim.chunk_size()))
            .filter(|start| *start < source_publication.total_bytes())
            .ok_or(BlobRecordDenial::InvalidReuseClaim)?;
        let expected_length =
            (source_publication.total_bytes() - start).min(u64::from(claim.chunk_size()));
        if observed != source_publication_frame_sha256
            || claim.store() != source_publication.store()
            || claim.scope() != source_publication.key_scope()
            || claim.chunk_size() != source_publication.chunk_size()
            || u64::from(claim.chunk_length()) != expected_length
        {
            return Err(BlobRecordDenial::InvalidReuseClaim);
        }
        Ok(Self {
            claim,
            source_publication,
            source_publication_frame_sha256,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let v1 = self.claim.encode();
        let publication = self.source_publication.encode();
        let mut payload = Vec::with_capacity(PAYLOAD_BYTES);
        payload.extend_from_slice(&v1[BLOB_RECORD_HEADER_BYTES..]);
        payload.extend_from_slice(&self.source_publication_frame_sha256);
        payload.extend_from_slice(&publication);
        encode(BlobRecordKind::ChunkReuseClaimV2, &payload)
            .expect("bounded reuse witness fits control frame")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::ChunkReuseClaimV2 {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload, frame.flags)
    }

    pub(super) fn decode_payload(payload: &[u8], flags: u16) -> Result<Self, BlobRecordDenial> {
        if payload.len() != PAYLOAD_BYTES {
            return Err(BlobRecordDenial::InvalidReuseClaim);
        }
        Self::new(
            BlobChunkReuseClaimV1::decode_payload(&payload[..V1_PAYLOAD_BYTES], flags)?,
            BlobGenerationPublicationV1::decode(&payload[V1_PAYLOAD_BYTES + 32..])?,
            payload[V1_PAYLOAD_BYTES..V1_PAYLOAD_BYTES + 32]
                .try_into()
                .expect("fixed witness digest"),
        )
    }

    pub const fn claim(self) -> BlobChunkReuseClaimV1 {
        self.claim
    }

    pub const fn source_publication(self) -> BlobGenerationPublicationV1 {
        self.source_publication
    }

    pub const fn source_publication_frame_sha256(self) -> [u8; 32] {
        self.source_publication_frame_sha256
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{decode_blob_record, PersistedRecordIdentity};

    fn fixture(
        source_ordinal: u64,
        source_length: u32,
    ) -> (BlobChunkReuseClaimV1, BlobGenerationPublicationV1) {
        let record = |ordinal| PersistedRecordIdentity::new([9; 16], ordinal).unwrap();
        let publication = BlobGenerationPublicationV1::new(
            [1; 16],
            [2; 16],
            [3; 16],
            1,
            record(4),
            [5; 32],
            64 * 1024 + 17,
            [6; 32],
            64 * 1024,
            [7; 32],
        )
        .unwrap();
        let claim = BlobChunkReuseClaimV1::new(
            [1; 16],
            [8; 16],
            0,
            [7; 32],
            64 * 1024,
            source_length,
            [10; 32],
            record(11),
            record(12),
            source_ordinal,
        )
        .unwrap();
        (claim, publication)
    }

    #[test]
    fn v1_remains_explicitly_decoded_and_v2_roundtrips_canonical_witness() {
        let (claim, publication) = fixture(1, 17);
        assert!(
            matches!(decode_blob_record(&claim.encode()), Ok(super::super::BlobRecordV1::ChunkReuseClaim(value)) if value == claim)
        );
        let digest: [u8; 32] = Sha256::digest(publication.encode()).into();
        let v2 = BlobChunkReuseClaimV2::new(claim, publication, digest).unwrap();
        assert_eq!(BlobChunkReuseClaimV2::decode(&v2.encode()), Ok(v2));
        assert!(
            matches!(decode_blob_record(&v2.encode()), Ok(super::super::BlobRecordV1::ChunkReuseClaimV2(value)) if value == v2)
        );
    }

    #[test]
    fn malformed_ordinal_length_and_publication_digest_are_denied() {
        let (claim, publication) = fixture(1, 17);
        let digest: [u8; 32] = Sha256::digest(publication.encode()).into();
        assert_eq!(
            BlobChunkReuseClaimV2::new(claim, publication, [0; 32]),
            Err(BlobRecordDenial::InvalidReuseClaim)
        );
        let (wrong_length, _) = fixture(1, 64 * 1024);
        assert_eq!(
            BlobChunkReuseClaimV2::new(wrong_length, publication, digest),
            Err(BlobRecordDenial::InvalidReuseClaim)
        );
        let (wrong_ordinal, _) = fixture(2, 17);
        assert_eq!(
            BlobChunkReuseClaimV2::new(wrong_ordinal, publication, digest),
            Err(BlobRecordDenial::InvalidReuseClaim)
        );
        let mut encoded = BlobChunkReuseClaimV2::new(claim, publication, digest)
            .unwrap()
            .encode();
        let last = encoded.len() - 1;
        encoded[last] ^= 1;
        assert!(BlobChunkReuseClaimV2::decode(&encoded).is_err());
    }
}
