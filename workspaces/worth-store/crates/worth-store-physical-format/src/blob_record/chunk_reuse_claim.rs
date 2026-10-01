use crate::PersistedRecordIdentity;

use super::envelope::{admitted_chunk_size, encode, nonzero_16, nonzero_32};
use super::{BlobRecordDenial, BlobRecordKind};

const PAYLOAD_BYTES: usize = 168;

/// A selected destination-session edge to an already selected chunk. Its
/// source publication and ordinal must still be verified from authority;
/// this frame does not promote a dedupe index hit into a content proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobChunkReuseClaimV1 {
    store: [u8; 16],
    destination_session: [u8; 16],
    destination_ordinal: u64,
    scope: [u8; 32],
    chunk_size: u32,
    chunk_length: u32,
    stored_digest: [u8; 32],
    selected_chunk: PersistedRecordIdentity,
    source_publication: PersistedRecordIdentity,
    source_ordinal: u64,
}

impl BlobChunkReuseClaimV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: [u8; 16],
        destination_session: [u8; 16],
        destination_ordinal: u64,
        scope: [u8; 32],
        chunk_size: u32,
        chunk_length: u32,
        stored_digest: [u8; 32],
        selected_chunk: PersistedRecordIdentity,
        source_publication: PersistedRecordIdentity,
        source_ordinal: u64,
    ) -> Result<Self, BlobRecordDenial> {
        let chunk_size = admitted_chunk_size(chunk_size)?;
        if chunk_length == 0 || chunk_length > chunk_size {
            return Err(BlobRecordDenial::InvalidReuseClaim);
        }
        Ok(Self {
            store: nonzero_16(store)?,
            destination_session: nonzero_16(destination_session)?,
            destination_ordinal,
            scope: nonzero_32(scope)?,
            chunk_size,
            chunk_length,
            stored_digest: nonzero_32(stored_digest)?,
            selected_chunk,
            source_publication,
            source_ordinal,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(PAYLOAD_BYTES);
        payload.extend_from_slice(&self.store);
        payload.extend_from_slice(&self.destination_session);
        payload.extend_from_slice(&self.destination_ordinal.to_le_bytes());
        payload.extend_from_slice(&self.scope);
        payload.extend_from_slice(&self.chunk_size.to_le_bytes());
        payload.extend_from_slice(&self.chunk_length.to_le_bytes());
        payload.extend_from_slice(&self.stored_digest);
        put_record(&mut payload, self.selected_chunk);
        put_record(&mut payload, self.source_publication);
        payload.extend_from_slice(&self.source_ordinal.to_le_bytes());
        encode(BlobRecordKind::ChunkReuseClaim, &payload)
            .expect("fixed-size reuse claim fits control frame")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::ChunkReuseClaim {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload, frame.flags)
    }

    pub(super) fn decode_payload(payload: &[u8], flags: u16) -> Result<Self, BlobRecordDenial> {
        if flags != 1 || payload.len() != PAYLOAD_BYTES {
            return Err(BlobRecordDenial::InvalidReuseClaim);
        }
        Self::new(
            payload[..16].try_into().expect("fixed store"),
            payload[16..32].try_into().expect("fixed session"),
            u64::from_le_bytes(payload[32..40].try_into().expect("fixed ordinal")),
            payload[40..72].try_into().expect("fixed scope"),
            u32::from_le_bytes(payload[72..76].try_into().expect("fixed chunk size")),
            u32::from_le_bytes(payload[76..80].try_into().expect("fixed chunk length")),
            payload[80..112].try_into().expect("fixed digest"),
            read_record(&payload[112..136])?,
            read_record(&payload[136..160])?,
            u64::from_le_bytes(payload[160..168].try_into().expect("fixed source ordinal")),
        )
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn destination_session(self) -> [u8; 16] {
        self.destination_session
    }
    pub const fn destination_ordinal(self) -> u64 {
        self.destination_ordinal
    }
    pub const fn scope(self) -> [u8; 32] {
        self.scope
    }
    pub const fn chunk_size(self) -> u32 {
        self.chunk_size
    }
    pub const fn chunk_length(self) -> u32 {
        self.chunk_length
    }
    pub const fn stored_digest(self) -> [u8; 32] {
        self.stored_digest
    }
    pub const fn selected_chunk(self) -> PersistedRecordIdentity {
        self.selected_chunk
    }
    pub const fn source_publication(self) -> PersistedRecordIdentity {
        self.source_publication
    }
    pub const fn source_ordinal(self) -> u64 {
        self.source_ordinal
    }
}

fn put_record(target: &mut Vec<u8>, record: PersistedRecordIdentity) {
    target.extend_from_slice(&record.allocation_epoch());
    target.extend_from_slice(&record.ordinal().to_le_bytes());
}

fn read_record(bytes: &[u8]) -> Result<PersistedRecordIdentity, BlobRecordDenial> {
    PersistedRecordIdentity::new(
        bytes[..16].try_into().expect("fixed record epoch"),
        u64::from_le_bytes(bytes[16..24].try_into().expect("fixed record ordinal")),
    )
    .ok_or(BlobRecordDenial::InvalidIdentity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_preserves_destination_and_source_claims() {
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
        assert_eq!(BlobChunkReuseClaimV1::decode(&claim.encode()), Ok(claim));
    }

    #[test]
    fn malformed_source_identity_and_chunk_length_are_denied() {
        let selected = PersistedRecordIdentity::new([5; 16], 6).unwrap();
        assert_eq!(
            BlobChunkReuseClaimV1::new(
                [1; 16],
                [2; 16],
                7,
                [3; 32],
                64 << 10,
                0,
                [4; 32],
                selected,
                selected,
                9
            ),
            Err(BlobRecordDenial::InvalidReuseClaim)
        );
    }
}
