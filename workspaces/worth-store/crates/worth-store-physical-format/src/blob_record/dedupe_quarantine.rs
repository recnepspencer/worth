use crate::PersistedRecordIdentity;

use super::envelope::{admitted_chunk_size, encode, nonzero_16, nonzero_32};
use super::{BlobRecordDenial, BlobRecordKind};

const PAYLOAD_BYTES: usize = 188;

/// A durable, selected denial of one scope-and-digest dedupe basis. The
/// referenced selected source and conflicting original chunk must be checked
/// through C.5 authority; the frame alone is not an equality proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobDedupeQuarantineV1 {
    store: [u8; 16],
    scope: [u8; 32],
    disputed_digest: [u8; 32],
    chunk_size: u32,
    source_publication: PersistedRecordIdentity,
    source_ordinal: u64,
    source_chunk: PersistedRecordIdentity,
    destination_session: [u8; 16],
    destination_ordinal: u64,
    conflicting_chunk: PersistedRecordIdentity,
}

impl BlobDedupeQuarantineV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        store: [u8; 16],
        scope: [u8; 32],
        disputed_digest: [u8; 32],
        chunk_size: u32,
        source_publication: PersistedRecordIdentity,
        source_ordinal: u64,
        source_chunk: PersistedRecordIdentity,
        destination_session: [u8; 16],
        destination_ordinal: u64,
        conflicting_chunk: PersistedRecordIdentity,
    ) -> Result<Self, BlobRecordDenial> {
        if source_chunk == conflicting_chunk {
            return Err(BlobRecordDenial::InvalidDedupeQuarantine);
        }
        Ok(Self {
            store: nonzero_16(store)?,
            scope: nonzero_32(scope)?,
            disputed_digest: nonzero_32(disputed_digest)?,
            chunk_size: admitted_chunk_size(chunk_size)?,
            source_publication,
            source_ordinal,
            source_chunk,
            destination_session: nonzero_16(destination_session)?,
            destination_ordinal,
            conflicting_chunk,
        })
    }

    pub fn encode(self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(PAYLOAD_BYTES);
        payload.extend_from_slice(&self.store);
        payload.extend_from_slice(&self.scope);
        payload.extend_from_slice(&self.disputed_digest);
        payload.extend_from_slice(&self.chunk_size.to_le_bytes());
        put_record(&mut payload, self.source_publication);
        payload.extend_from_slice(&self.source_ordinal.to_le_bytes());
        put_record(&mut payload, self.source_chunk);
        payload.extend_from_slice(&self.destination_session);
        payload.extend_from_slice(&self.destination_ordinal.to_le_bytes());
        put_record(&mut payload, self.conflicting_chunk);
        encode(BlobRecordKind::DedupeQuarantine, &payload)
            .expect("fixed-size dedupe quarantine fits control frame")
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, BlobRecordDenial> {
        let frame = super::envelope::decode(bytes)?;
        if frame.kind != BlobRecordKind::DedupeQuarantine {
            return Err(BlobRecordDenial::UnknownKind);
        }
        Self::decode_payload(frame.payload, frame.flags)
    }

    pub(super) fn decode_payload(payload: &[u8], flags: u16) -> Result<Self, BlobRecordDenial> {
        if flags != 1 || payload.len() != PAYLOAD_BYTES {
            return Err(BlobRecordDenial::InvalidDedupeQuarantine);
        }
        Self::new(
            payload[..16].try_into().expect("fixed store"),
            payload[16..48].try_into().expect("fixed scope"),
            payload[48..80].try_into().expect("fixed digest"),
            u32::from_le_bytes(payload[80..84].try_into().expect("fixed chunk size")),
            read_record(&payload[84..108])?,
            u64::from_le_bytes(payload[108..116].try_into().expect("fixed source ordinal")),
            read_record(&payload[116..140])?,
            payload[140..156]
                .try_into()
                .expect("fixed destination session"),
            u64::from_le_bytes(
                payload[156..164]
                    .try_into()
                    .expect("fixed destination ordinal"),
            ),
            read_record(&payload[164..188])?,
        )
    }

    pub const fn store(self) -> [u8; 16] {
        self.store
    }
    pub const fn scope(self) -> [u8; 32] {
        self.scope
    }
    pub const fn disputed_digest(self) -> [u8; 32] {
        self.disputed_digest
    }
    pub const fn chunk_size(self) -> u32 {
        self.chunk_size
    }
    pub const fn source_publication(self) -> PersistedRecordIdentity {
        self.source_publication
    }
    pub const fn source_ordinal(self) -> u64 {
        self.source_ordinal
    }
    pub const fn source_chunk(self) -> PersistedRecordIdentity {
        self.source_chunk
    }
    pub const fn destination_session(self) -> [u8; 16] {
        self.destination_session
    }
    pub const fn destination_ordinal(self) -> u64 {
        self.destination_ordinal
    }
    pub const fn conflicting_chunk(self) -> PersistedRecordIdentity {
        self.conflicting_chunk
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

    fn record(byte: u8) -> PersistedRecordIdentity {
        PersistedRecordIdentity::new([byte; 16], u64::from(byte)).unwrap()
    }

    #[test]
    fn roundtrip_preserves_both_selected_chunk_coordinates() {
        let claim = BlobDedupeQuarantineV1::new(
            [1; 16],
            [2; 32],
            [3; 32],
            64 << 10,
            record(4),
            5,
            record(6),
            [7; 16],
            8,
            record(9),
        )
        .unwrap();
        assert_eq!(claim.encode().len(), 236);
        assert_eq!(BlobDedupeQuarantineV1::decode(&claim.encode()), Ok(claim));
    }

    #[test]
    fn same_chunk_cannot_witness_conflict() {
        assert_eq!(
            BlobDedupeQuarantineV1::new(
                [1; 16],
                [2; 32],
                [3; 32],
                64 << 10,
                record(4),
                5,
                record(6),
                [7; 16],
                8,
                record(6),
            ),
            Err(BlobRecordDenial::InvalidDedupeQuarantine)
        );
    }
}
