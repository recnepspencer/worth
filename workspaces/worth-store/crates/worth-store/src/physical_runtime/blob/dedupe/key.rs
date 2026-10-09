use sha2::{Digest, Sha256};
use worth_store_physical_format::PersistedRecordIdentity;

/// Scope precedes content digest in canonical lexicographic B-tree order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct DedupeIndexKey([u8; 64]);

impl DedupeIndexKey {
    pub(in crate::physical_runtime) fn for_chunk(
        scope: [u8; 32],
        chunk_size: u32,
        bytes: &[u8],
    ) -> Self {
        let mut content = Sha256::new();
        content.update([1, 0, 0, 0]);
        content.update(chunk_size.to_le_bytes());
        content.update((bytes.len() as u32).to_le_bytes());
        content.update(bytes);
        Self::from_digest(scope, content.finalize().into())
    }

    pub(in crate::physical_runtime) fn from_digest(scope: [u8; 32], digest: [u8; 32]) -> Self {
        let mut bytes = [0; 64];
        bytes[..32].copy_from_slice(&scope);
        bytes[32..].copy_from_slice(&digest);
        Self(bytes)
    }

    pub(in crate::physical_runtime) const fn bytes(self) -> [u8; 64] {
        self.0
    }

    pub(in crate::physical_runtime) fn digest(self) -> [u8; 32] {
        self.0[32..].try_into().expect("fixed digest width")
    }
}

/// A derived locator to a selected publication and one of its tree edges.
/// The Store rechecks the source publication, edge and chunk before reuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct DedupeIndexValue {
    publication: PersistedRecordIdentity,
    source_ordinal: u64,
    chunk: PersistedRecordIdentity,
}

impl DedupeIndexValue {
    pub(in crate::physical_runtime) const fn new(
        publication: PersistedRecordIdentity,
        source_ordinal: u64,
        chunk: PersistedRecordIdentity,
    ) -> Self {
        Self {
            publication,
            source_ordinal,
            chunk,
        }
    }

    pub(in crate::physical_runtime) fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 56 {
            return None;
        }
        Some(Self {
            publication: read_record(&bytes[..24])?,
            source_ordinal: u64::from_le_bytes(bytes[24..32].try_into().ok()?),
            chunk: read_record(&bytes[32..56])?,
        })
    }

    pub(in crate::physical_runtime) fn encode(self) -> [u8; 56] {
        let mut bytes = [0; 56];
        put_record(&mut bytes[..24], self.publication);
        bytes[24..32].copy_from_slice(&self.source_ordinal.to_le_bytes());
        put_record(&mut bytes[32..], self.chunk);
        bytes
    }

    pub(in crate::physical_runtime) const fn publication(self) -> PersistedRecordIdentity {
        self.publication
    }
    pub(in crate::physical_runtime) const fn source_ordinal(self) -> u64 {
        self.source_ordinal
    }
    pub(in crate::physical_runtime) const fn chunk(self) -> PersistedRecordIdentity {
        self.chunk
    }
}

fn read_record(bytes: &[u8]) -> Option<PersistedRecordIdentity> {
    PersistedRecordIdentity::new(
        bytes[..16].try_into().ok()?,
        u64::from_le_bytes(bytes[16..24].try_into().ok()?),
    )
}

fn put_record(bytes: &mut [u8], record: PersistedRecordIdentity) {
    bytes[..16].copy_from_slice(&record.allocation_epoch());
    bytes[16..24].copy_from_slice(&record.ordinal().to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_matches_versioned_canonical_chunk_digest() {
        let frame = worth_store_physical_format::BlobChunkFrameV1::encode(
            worth_store_physical_format::BlobChunkOccurrenceV1::new([1; 16], [2; 16], 0).unwrap(),
            64 << 10,
            &[3; 64 << 10],
        )
        .unwrap();
        let digest = worth_store_physical_format::DecodedBlobChunkFrameV1::decode(&frame)
            .unwrap()
            .stored_digest();
        assert_eq!(
            DedupeIndexKey::for_chunk([4; 32], 64 << 10, &[3; 64 << 10]).digest(),
            digest
        );
    }

    #[test]
    fn value_roundtrips_exact_source_coordinates() {
        let value = DedupeIndexValue::new(
            PersistedRecordIdentity::new([1; 16], 2).unwrap(),
            3,
            PersistedRecordIdentity::new([4; 16], 5).unwrap(),
        );
        assert_eq!(DedupeIndexValue::decode(&value.encode()), Some(value));
    }
}
