use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::{
    blob_record_v1_validation_digest, BlobRecordKind, PersistedRecordIdentity,
};

use super::{PhysicalArtifactScope, PhysicalArtifactScopeIdentity};
use crate::localization::PhysicalByteRange;

impl PhysicalArtifactScope {
    pub const fn blob_chunk_frame(
        store: StableStoreIdentity,
        record: PersistedRecordIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::blob_record(store, record, BlobRecordKind::Chunk, range)
    }

    pub const fn blob_tree_node(
        store: StableStoreIdentity,
        record: PersistedRecordIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::blob_record(store, record, BlobRecordKind::TreeNode, range)
    }

    pub const fn blob_generation_publication(
        store: StableStoreIdentity,
        record: PersistedRecordIdentity,
        range: PhysicalByteRange,
    ) -> Self {
        Self::blob_record(store, record, BlobRecordKind::GenerationPublished, range)
    }

    const fn blob_record(
        store: StableStoreIdentity,
        record: PersistedRecordIdentity,
        kind: BlobRecordKind,
        range: PhysicalByteRange,
    ) -> Self {
        Self::new(
            store,
            PhysicalArtifactScopeIdentity::BlobRecord { record, kind },
            range,
        )
    }

    pub const fn blob_record_identity(self) -> Option<(PersistedRecordIdentity, BlobRecordKind)> {
        match self.identity {
            PhysicalArtifactScopeIdentity::BlobRecord { record, kind } => Some((record, kind)),
            _ => None,
        }
    }

    pub(crate) fn exact_blob_record_scope_digest(self) -> [u8; 32] {
        const DOMAIN: &[u8; 42] = b"worth-store-blob-record-integrity-scope-v1";
        let (record, kind) = self
            .blob_record_identity()
            .expect("blob scope digest requires a blob record");
        let mut preimage = [0_u8; 99];
        preimage[..42].copy_from_slice(DOMAIN);
        preimage[42..58].copy_from_slice(&self.store.bytes());
        preimage[58..74].copy_from_slice(&record.allocation_epoch());
        preimage[74..82].copy_from_slice(&record.ordinal().to_le_bytes());
        preimage[82] = kind as u8;
        preimage[83..91].copy_from_slice(&self.range.offset().to_le_bytes());
        preimage[91..99].copy_from_slice(&self.range.length().to_le_bytes());
        blob_record_v1_validation_digest(&preimage)
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    use worth_store_physical_format::store_namespace::{
        ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };

    use super::*;

    /// SHA-256 of the 99-byte preimage below; persisted scope identities
    /// depend on it, so any layout change must be a new versioned domain.
    const CHUNK_SCOPE_DIGEST_HEX: &str =
        "f2613326b102b9e4c6ed34f9ac1a7654bacf0efb4bca6c17f54e557b3c0014d8";

    #[test]
    fn blob_scope_digest_pins_the_99_byte_preimage_layout() {
        let store = StoreNamespaceIdentityRecord::new(
            StoreNamespaceVersion::CURRENT,
            ProposedStoreIdentity::from_nonzero_bytes([0x11; 16]).unwrap(),
        )
        .published_identity();
        let record = PersistedRecordIdentity::new([0x22; 16], 0x0102_0304_0506_0708).unwrap();
        let range = PhysicalByteRange::new(0x1000, 0x30).unwrap();
        let digest = PhysicalArtifactScope::blob_chunk_frame(store, record, range)
            .exact_blob_record_scope_digest();

        let mut preimage = Vec::with_capacity(99);
        preimage.extend_from_slice(b"worth-store-blob-record-integrity-scope-v1");
        preimage.extend_from_slice(&[0x11; 16]);
        preimage.extend_from_slice(&[0x22; 16]);
        preimage.extend_from_slice(&[8, 7, 6, 5, 4, 3, 2, 1]);
        preimage.push(BlobRecordKind::Chunk as u8);
        preimage.extend_from_slice(&0x1000_u64.to_le_bytes());
        preimage.extend_from_slice(&0x30_u64.to_le_bytes());
        assert_eq!(preimage.len(), 99);
        let independent: [u8; 32] = Sha256::digest(&preimage).into();
        assert_eq!(digest, independent);
        let hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        assert_eq!(hex, CHUNK_SCOPE_DIGEST_HEX);
    }
}
